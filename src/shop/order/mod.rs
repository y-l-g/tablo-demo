//! Orders: the list, the form and the fulfilment actions, with the lines, payments and shipping
//! address an order's page holds, and the pipeline above the list.

mod actions;
mod items;
mod payments;
mod pipeline;
mod shipping_address;

pub use items::OrderItemResource;
pub use payments::PaymentResource;
pub use pipeline::PipelineWidget;
pub use shipping_address::OrderAddressResource;

use tablo::{Notification, prelude::*};
use topcoat::{Result, context::Cx, icon::iconify::iconify_icon};

use self::actions::{CancelOrder, DeliverOrder, ProcessOrder, ReplicateOrder, ShipOrder};
use super::{
    CustomerResource, Order, OrderAddress, OrderItem, OrderStatus, Payment, customer_name,
};
use crate::access::{self, Domain};
use crate::shared::{OptionTabs, notify};

/// Orders.
pub struct OrderResource;

impl Resource for OrderResource {
    type Model = Order;
    type Form = OrderForm;

    fn declare() -> ResourceDef<Self> {
        let c = OrderForm::controls();
        ResourceDef::new()
            .icon(iconify_icon!("lucide:shopping-cart"))
            .navigation_group(tablo::text!("navigation-shop"))
            .navigation_order(1)
            .navigation_badge(Order::fields().status().eq(OrderStatus::New))
            .policy(access::policy(Domain::Shop))
            .table(order_table())
            .detail(Detail::new(
                Section::new(tablo::text!("section-order")).columns((
                    Grid::new(2).columns((
                        TextColumn::new(lens!(Order.number)),
                        RelationColumn::of::<CustomerResource>(relation!(Order.customer)),
                        TextColumn::new(lens!(Order.status)).badge(),
                        TextColumn::new(lens!(Order.total_price)).money(Currency::USD),
                        TextColumn::new(lens!(Order.shipping_price)).money(Currency::USD),
                        DateColumn::new(lens!(Order.created_at))
                            .label(tablo::text!("label-order-date")),
                        RelationColumn::of::<OrderAddressResource>(relation!(Order.address))
                            .label(tablo::text!("label-shipping-address")),
                    )),
                    RichTextColumn::new(lens!(Order.notes)),
                )),
            ))
            .form(Schema::new(
                Section::new(tablo::text!("section-order")).schema(Grid::new(2).schema((
                    c.number,
                    c.customer_id.searchable().creatable(),
                    c.status.buttons(),
                    c.notes.rich_text(),
                ))),
            ))
            .record_title(lens!(Order.number))
            .global_search(
                GlobalSearch::new()
                    .detail(tablo::text!("label-status"), lens!(Order.status))
                    .detail(tablo::text!("label-total"), lens!(Order.total_price)),
            )
            .relation(Relation::has_many::<OrderItemResource>(OrderItem::fields().order_id()).tab())
            .relation(Relation::has_one::<OrderAddressResource>(
                OrderAddress::fields().order_id(),
            ))
            .relation(Relation::has_many::<PaymentResource>(Payment::fields().order_id()).tab())
            .action::<ProcessOrder>()
            .action::<ShipOrder>()
            .action::<DeliverOrder>()
            .action::<ReplicateOrder>()
            .action::<CancelOrder>()
            .soft_delete(lens!(Order.deleted_at))
            .list_widget::<PipelineWidget>()
    }

    /// A paid order stays: a delete declines while it holds payments, and the message flashes.
    async fn delete_record(
        _cx: &Cx,
        order: &Order,
        ex: &mut dyn toasty::Executor,
    ) -> Result<(), WriteError> {
        let payments = Payment::filter(Payment::fields().order_id().eq(order.id))
            .exec(&mut *ex)
            .await?;
        if !payments.is_empty() {
            return Err(WriteError::declined(tablo::text!(
                "error-order-has-payments"
            )));
        }
        Order::filter_by_id(order.id)
            .update()
            .deleted_at(Some(jiff::Timestamp::now()))
            .exec(&mut *ex)
            .await?;
        Ok(())
    }

    /// Tells the signed-in user about a new order, once the write committed.
    async fn after_commit(cx: &Cx, change: Change<Order>) -> Result<()> {
        if let Change::Create(order) = &change {
            notify_new_order(cx, order).await?;
        }
        Ok(())
    }
}

/// The order list.
fn order_table() -> Table<Order> {
    Table::new((
        TextColumn::new(lens!(Order.number))
            .weight(FontWeight::Medium)
            .searchable()
            .sortable(),
        RelationColumn::of::<CustomerResource>(relation!(Order.customer)).toggleable(),
        TextColumn::new(lens!(Order.status)).badge(),
        TextColumn::new(lens!(Order.total_price)).money(Currency::USD),
        TextColumn::new(lens!(Order.shipping_price))
            .money(Currency::USD)
            .toggleable(),
        DateColumn::new(lens!(Order.created_at))
            .label(tablo::text!("label-order-date"))
            .sortable()
            .toggleable(),
    ))
    .default_sort_desc(lens!(Order.created_at))
    .action_layout(ActionLayout::Menu)
    .row_color(|order: &Order| (order.status == OrderStatus::Cancelled).then_some(Color::Danger))
    .row_color_reading(Reads::of(lens!(Order.status)))
    .option_tabs(|status: OrderStatus| Order::fields().status().eq(status))
    .filters((DateFilter::new(lens!(Order.created_at))
        .label(tablo::text!("label-order-date"))
        .range(),))
    // Counted under the current tab, search and filters, across every page.
    .summary(Summary::count(tablo::text!("summary-orders")))
    .summary(
        Summary::count(tablo::text!("summary-cancelled"))
            .query(Order::fields().status().eq(OrderStatus::Cancelled)),
    )
}

/// The order's record form.
#[derive(tablo::RecordForm)]
#[form(model = Order)]
pub struct OrderForm {
    pub number: String,
    #[form(relationship = CustomerResource)]
    pub customer_id: Option<u64>,
    #[form(options)]
    pub status: OrderStatus,
    pub notes: Option<String>,
}

/// Tells the signed-in user that an order was placed.
async fn notify_new_order(cx: &Cx, order: &Order) -> Result<()> {
    let description = match customer_name(cx, order.customer_id).await {
        Some(name) => tablo::t!(cx, "notice-order-placed", customer = name),
        None => tablo::t!(cx, "notice-order-placed-unknown"),
    };
    notify(
        cx,
        Notification::success(tablo::t!(cx, "notice-new-order")).description(description),
        tablo::url::edit::<OrderResource>(cx, order),
    )
    .await
}
