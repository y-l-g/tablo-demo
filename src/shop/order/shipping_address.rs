//! Where an order ships, which the order's own form edits.

use tablo::prelude::*;
use topcoat::icon::iconify::iconify_icon;

use super::OrderResource;
use crate::access::{self, Domain};
use crate::shop::{CITIES, CountryCode, OrderAddress};

/// Where an order ships: the order's own form edits it, as its one child.
pub struct OrderAddressResource;

impl Resource for OrderAddressResource {
    type Model = OrderAddress;
    type Form = OrderAddressForm;

    fn declare() -> ResourceDef<Self> {
        let c = OrderAddressForm::controls();
        ResourceDef::new()
            .icon(iconify_icon!("lucide:truck"))
            .without_navigation()
            .policy(access::policy(Domain::Shop))
            .without_global_search()
            .table(Table::new((
                RelationColumn::of::<OrderResource>(relation!(OrderAddress.order)),
                TextColumn::new(lens!(OrderAddress.street)),
                TextColumn::new(lens!(OrderAddress.city)),
                TextColumn::new(lens!(OrderAddress.zip)),
                TextColumn::new(lens!(OrderAddress.country)).width(ColumnWidth::Auto),
            )))
            .detail(Detail::new(
                Section::new(tablo::text!("section-order-address")).columns((
                    RelationColumn::of::<OrderResource>(relation!(OrderAddress.order)),
                    TextColumn::new(lens!(OrderAddress.street)),
                    TextColumn::new(lens!(OrderAddress.city)),
                    TextColumn::new(lens!(OrderAddress.state)),
                    TextColumn::new(lens!(OrderAddress.zip)),
                    TextColumn::new(lens!(OrderAddress.country)),
                )),
            ))
            .form(Schema::new(
                Section::new(tablo::text!("section-order-address")).schema((
                    c.country,
                    c.street,
                    Grid::new(3).schema((
                        c.city.suggestions(CITIES.iter().copied()),
                        c.state,
                        c.zip,
                    )),
                )),
            ))
            .record_title(lens!(OrderAddress.street))
    }
}

/// The order address's record form.
#[derive(tablo::RecordForm)]
#[form(model = OrderAddress)]
pub struct OrderAddressForm {
    #[form(relationship = OrderResource)]
    pub order_id: Option<u64>,
    #[form(options)]
    pub country: Option<CountryCode>,
    pub street: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub zip: Option<String>,
}
