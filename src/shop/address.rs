//! Addresses, which a customer's or a brand's page lists.

use tablo::prelude::*;
use topcoat::icon::iconify::iconify_icon;

use super::{Address, BrandResource, CountryCode, CustomerResource};
use crate::access::{self, Domain};

/// The cities the address forms suggest, which the seeded addresses are in.
pub const CITIES: &[&str] = &[
    "Portland", "Austin", "Denver", "Boston", "Seattle", "Chicago", "Toronto", "Dublin",
];

/// Addresses, which the customer and brand pages list.
pub struct AddressResource;

impl Resource for AddressResource {
    type Model = Address;
    type Form = AddressForm;

    fn declare() -> ResourceDef<Self> {
        let c = AddressForm::controls();
        ResourceDef::new()
            .icon(iconify_icon!("lucide:map-pin"))
            .without_navigation()
            .policy(access::policy(Domain::Shop))
            .without_global_search()
            .table(
                Table::new((
                    TextColumn::new(lens!(Address.street)),
                    TextColumn::new(lens!(Address.zip)),
                    TextColumn::new(lens!(Address.city)).sortable(),
                    TextColumn::new(lens!(Address.country)).width(ColumnWidth::Auto),
                ))
                .default_sort(lens!(Address.city)),
            )
            .detail(Detail::new(
                Section::new(tablo::text!("section-address")).columns((
                    TextColumn::new(lens!(Address.street)),
                    TextColumn::new(lens!(Address.city)),
                    TextColumn::new(lens!(Address.state)),
                    TextColumn::new(lens!(Address.zip)),
                    TextColumn::new(lens!(Address.country)),
                )),
            ))
            .form(Schema::new(
                Section::new(tablo::text!("section-address")).schema((
                    c.customer_id.searchable(),
                    c.brand_id.searchable(),
                    c.country.searchable(),
                    c.street,
                    Grid::new(3).schema((
                        c.city.suggestions(CITIES.iter().copied()),
                        c.state,
                        c.zip,
                    )),
                )),
            ))
            .record_title(lens!(Address.street))
    }
}

/// The address's record form.
#[derive(tablo::RecordForm)]
#[form(model = Address)]
pub struct AddressForm {
    #[form(relationship = CustomerResource)]
    pub customer_id: Option<u64>,
    #[form(relationship = BrandResource)]
    pub brand_id: Option<u64>,
    #[form(options)]
    pub country: Option<CountryCode>,
    pub street: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub zip: Option<String>,
}
