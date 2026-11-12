//! The lines of an order, which its page lists.

use rust_decimal::Decimal;
use tablo::prelude::*;
use topcoat::icon::iconify::iconify_icon;

use super::OrderResource;
use crate::access::{self, Domain};
use crate::shop::{OrderItem, ProductResource};

/// One line of an order, which the order page lists.
pub struct OrderItemResource;

impl Resource for OrderItemResource {
    type Model = OrderItem;
    type Form = OrderItemForm;

    fn declare() -> ResourceDef<Self> {
        let col = OrderItem::columns();
        let shown = OrderItem::columns();
        let c = OrderItemForm::controls();
        ResourceDef::new()
            .icon(iconify_icon!("lucide:clipboard-list"))
            .without_navigation()
            .policy(access::policy(Domain::Shop))
            .without_global_search()
            .table(Table::new((
                RelationColumn::of::<ProductResource>(relation!(OrderItem.product)),
                col.qty.text().numeric(0),
                col.unit_price.text().money(Currency::USD),
            )))
            .detail(Detail::new(
                Section::new(tablo::text!("section-order-item")).columns((
                    RelationColumn::of::<OrderResource>(relation!(OrderItem.order)),
                    RelationColumn::of::<ProductResource>(relation!(OrderItem.product)),
                    shown.qty.text().numeric(0),
                    shown.unit_price.text().money(Currency::USD),
                )),
            ))
            .form(Schema::new(
                Section::new(tablo::text!("section-order-item")).schema((
                    c.order_id.searchable(),
                    c.product_id.searchable(),
                    Grid::new(2).schema((c.qty, c.unit_price)),
                )),
            ))
    }
}

/// The order item's record form.
#[derive(tablo::RecordForm)]
#[form(model = OrderItem)]
pub struct OrderItemForm {
    #[form(relationship = OrderResource)]
    pub order_id: Option<u64>,
    #[form(relationship = ProductResource)]
    pub product_id: Option<u64>,
    pub qty: i64,
    pub unit_price: Decimal,
}
