//! What moves an order along its fulfilment flow, and what copies one.

use tablo::prelude::*;
use topcoat::context::Cx;

use super::OrderResource;
use crate::shared::replica_suffix;
use crate::shop::{Order, OrderItem, OrderStatus, Product};

/// Moves a new order to processing.
pub struct ProcessOrder;

impl Action<OrderResource> for ProcessOrder {
    type Input = ();
    const NAME: &'static str = "process";

    fn declare() -> ActionDef<OrderResource> {
        ActionDef::new()
            .places(Places::ROW)
            .can_run(|_, order: &Order| order.status == OrderStatus::New)
            .success(|_| tablo::text!("flash-order-processing"))
    }

    async fn run(
        _cx: &Cx,
        orders: &[Order],
        (): (),
        ex: &mut dyn toasty::Executor,
    ) -> Result<Outcome, WriteError<()>> {
        set_status(orders, OrderStatus::Processing, ex).await
    }
}

/// What `ShipOrder` asks for.
#[derive(FormInput)]
pub struct ShippingNotes {
    #[form(multiline = 3)]
    pub shipping_notes: Option<String>,
}

/// Ships a processing order.
pub struct ShipOrder;

impl Action<OrderResource> for ShipOrder {
    type Input = ShippingNotes;
    const NAME: &'static str = "ship";

    fn declare() -> ActionDef<OrderResource> {
        ActionDef::new()
            .places(Places::ROW)
            .can_run(|_, order: &Order| order.status == OrderStatus::Processing)
            .success(|_| tablo::text!("flash-order-shipped"))
    }

    async fn run(
        _cx: &Cx,
        orders: &[Order],
        shipping: ShippingNotes,
        ex: &mut dyn toasty::Executor,
    ) -> Result<Outcome, WriteError<ShippingNotes>> {
        Order::filter(filter_keys(orders))
            .update()
            .status(OrderStatus::Shipped)
            .notes(shipping.shipping_notes)
            .exec(ex)
            .await?;
        count_sold(orders, ex).await?;
        Ok(Outcome::done())
    }
}

/// Marks a shipped order delivered, confirming first.
pub struct DeliverOrder;

impl Action<OrderResource> for DeliverOrder {
    type Input = ();
    const NAME: &'static str = "deliver";

    fn declare() -> ActionDef<OrderResource> {
        ActionDef::new()
            .places(Places::ROW)
            .confirm()
            .can_run(|_, order: &Order| order.status == OrderStatus::Shipped)
            .success(|_| tablo::text!("flash-order-delivered"))
    }

    async fn run(
        _cx: &Cx,
        orders: &[Order],
        (): (),
        ex: &mut dyn toasty::Executor,
    ) -> Result<Outcome, WriteError<()>> {
        set_status(orders, OrderStatus::Delivered, ex).await
    }
}

/// Copies an order as a new one under a fresh number, confirming first.
pub struct ReplicateOrder;

impl Action<OrderResource> for ReplicateOrder {
    type Input = ();
    const NAME: &'static str = "replicate";

    fn declare() -> ActionDef<OrderResource> {
        ActionDef::new()
            .places(Places::EDIT)
            .confirm()
            .success(|_| tablo::text!("flash-order-replicated"))
    }

    async fn run(
        _cx: &Cx,
        orders: &[Order],
        (): (),
        ex: &mut dyn toasty::Executor,
    ) -> Result<Outcome, WriteError<()>> {
        let date = jiff::Zoned::now().strftime("%Y%m%d").to_string();
        for order in orders {
            toasty::create!(Order {
                customer_id: order.customer_id,
                number: format!("OR-{date}-{}", replica_suffix()),
                total_price: order.total_price,
                shipping_price: order.shipping_price,
                shipping_method: order.shipping_method.clone(),
                notes: order.notes.clone(),
            })
            .exec(&mut *ex)
            .await?;
        }
        Ok(Outcome::done())
    }
}

/// Cancels an order that is neither delivered nor already cancelled.
pub struct CancelOrder;

impl Action<OrderResource> for CancelOrder {
    type Input = ();
    const NAME: &'static str = "cancel";

    fn declare() -> ActionDef<OrderResource> {
        ActionDef::new()
            .places(Places::ROW)
            .confirm()
            .can_run(|_, order: &Order| {
                !matches!(
                    order.status,
                    OrderStatus::Delivered | OrderStatus::Cancelled
                )
            })
            .success(|_| tablo::text!("flash-order-cancelled"))
    }

    async fn run(
        _cx: &Cx,
        orders: &[Order],
        (): (),
        ex: &mut dyn toasty::Executor,
    ) -> Result<Outcome, WriteError<()>> {
        set_status(orders, OrderStatus::Cancelled, ex).await
    }
}

/// Moves every order of `orders` to `status`.
async fn set_status<K: tablo::FormFields>(
    orders: &[Order],
    status: OrderStatus,
    ex: &mut dyn toasty::Executor,
) -> Result<Outcome, WriteError<K>> {
    Order::filter(filter_keys(orders))
        .update()
        .status(status)
        .exec(ex)
        .await?;
    Ok(Outcome::done())
}

/// Adds each line of the shipped `orders` to its product's units sold.
async fn count_sold(orders: &[Order], ex: &mut dyn toasty::Executor) -> toasty::Result<()> {
    for order in orders {
        let items = OrderItem::filter(OrderItem::fields().order_id().eq(Some(order.id)))
            .exec(&mut *ex)
            .await?;
        for item in items {
            let Some(product_id) = item.product_id else {
                continue;
            };
            let product = Product::filter_by_id(product_id)
                .first()
                .exec(&mut *ex)
                .await?;
            let Some(product) = product else {
                continue;
            };
            Product::filter_by_id(product_id)
                .update()
                .units_sold(product.units_sold + item.qty.max(0) as u64)
                .exec(&mut *ex)
                .await?;
        }
    }
    Ok(())
}
