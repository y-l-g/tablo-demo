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
    const PLACES: Places = Places::ROW;

    fn can_run(_cx: &Cx, order: &Order) -> bool {
        order.status == OrderStatus::New
    }

    async fn run(
        _cx: &Cx,
        orders: &[Order],
        (): (),
        ex: &mut dyn toasty::Executor,
    ) -> Result<Outcome, WriteError<std::convert::Infallible>> {
        set_status(orders, OrderStatus::Processing, ex).await
    }

    fn success(_count: usize) -> Text {
        tablo::text!("flash-order-processing")
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
    const PLACES: Places = Places::ROW;

    fn can_run(_cx: &Cx, order: &Order) -> bool {
        order.status == OrderStatus::Processing
    }

    async fn run(
        _cx: &Cx,
        orders: &[Order],
        shipping: ShippingNotes,
        ex: &mut dyn toasty::Executor,
    ) -> Result<Outcome, WriteError<ShippingNotesField>> {
        Order::filter(filter_keys(orders))
            .update()
            .status(OrderStatus::Shipped)
            .notes(shipping.shipping_notes)
            .exec(ex)
            .await?;
        count_sold(orders, ex).await?;
        Ok(Outcome::done())
    }

    fn success(_count: usize) -> Text {
        tablo::text!("flash-order-shipped")
    }
}

/// Marks a shipped order delivered, confirming first.
pub struct DeliverOrder;

impl Action<OrderResource> for DeliverOrder {
    type Input = ();
    const NAME: &'static str = "deliver";
    const PLACES: Places = Places::ROW;
    const CONFIRM: bool = true;

    fn can_run(_cx: &Cx, order: &Order) -> bool {
        order.status == OrderStatus::Shipped
    }

    async fn run(
        _cx: &Cx,
        orders: &[Order],
        (): (),
        ex: &mut dyn toasty::Executor,
    ) -> Result<Outcome, WriteError<std::convert::Infallible>> {
        set_status(orders, OrderStatus::Delivered, ex).await
    }

    fn success(_count: usize) -> Text {
        tablo::text!("flash-order-delivered")
    }
}

/// Copies an order as a new one under a fresh number, confirming first.
pub struct ReplicateOrder;

impl Action<OrderResource> for ReplicateOrder {
    type Input = ();
    const NAME: &'static str = "replicate";
    const PLACES: Places = Places::EDIT;
    const CONFIRM: bool = true;

    async fn run(
        _cx: &Cx,
        orders: &[Order],
        (): (),
        ex: &mut dyn toasty::Executor,
    ) -> Result<Outcome, WriteError<std::convert::Infallible>> {
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

    fn success(_count: usize) -> Text {
        tablo::text!("flash-order-replicated")
    }
}

/// Cancels an order that is neither delivered nor already cancelled.
pub struct CancelOrder;

impl Action<OrderResource> for CancelOrder {
    type Input = ();
    const NAME: &'static str = "cancel";
    const PLACES: Places = Places::ROW;
    const CONFIRM: bool = true;

    fn can_run(_cx: &Cx, order: &Order) -> bool {
        !matches!(
            order.status,
            OrderStatus::Delivered | OrderStatus::Cancelled
        )
    }

    async fn run(
        _cx: &Cx,
        orders: &[Order],
        (): (),
        ex: &mut dyn toasty::Executor,
    ) -> Result<Outcome, WriteError<std::convert::Infallible>> {
        set_status(orders, OrderStatus::Cancelled, ex).await
    }

    fn success(_count: usize) -> Text {
        tablo::text!("flash-order-cancelled")
    }
}

/// Moves every order of `orders` to `status`.
async fn set_status<K>(
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
