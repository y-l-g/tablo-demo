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
        let col = OrderAddress::columns();
        let shown = OrderAddress::columns();
        let c = OrderAddressForm::controls();
        ResourceDef::new()
            .icon(iconify_icon!("lucide:truck"))
            .without_navigation()
            .policy(access::policy(Domain::Shop))
            .without_global_search()
            .table(Table::new((
                RelationColumn::of::<OrderResource>(relation!(OrderAddress.order)),
                col.street.text(),
                col.city.text(),
                col.zip.text(),
                col.country.text().width(ColumnWidth::Auto),
            )))
            .detail(Detail::new(
                Section::new(tablo::text!("section-order-address")).columns((
                    RelationColumn::of::<OrderResource>(relation!(OrderAddress.order)),
                    shown.street.text(),
                    shown.city.text(),
                    shown.state.text(),
                    shown.zip.text(),
                    shown.country.text(),
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
