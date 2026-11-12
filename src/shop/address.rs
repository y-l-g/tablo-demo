//! Addresses, which a customer's or a brand's page lists, each opening on a map of another site.

use tablo::prelude::*;
use topcoat::{context::Cx, icon::iconify::iconify_icon};

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
        let col = Address::columns();
        let shown = Address::columns();
        let c = AddressForm::controls();
        ResourceDef::new()
            .icon(iconify_icon!("lucide:map-pin"))
            .without_navigation()
            .policy(access::policy(Domain::Shop))
            .without_global_search()
            .table(
                Table::new((
                    col.street.text(),
                    col.zip.text(),
                    col.city.text().sortable(),
                    col.country.text().width(ColumnWidth::Auto),
                ))
                .default_sort(lens!(Address.city)),
            )
            .detail(Detail::new(
                Section::new(tablo::text!("section-address")).columns((
                    shown.street.text(),
                    shown.city.text(),
                    shown.state.text(),
                    shown.zip.text(),
                    shown.country.text(),
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
            .action::<ShowOnMap>()
            .record_title(lens!(Address.street))
    }
}

/// Where [`ShowOnMap`] lands, which the admin panel's `form-action` allows.
pub const MAP_ORIGIN: &str = "https://www.openstreetmap.org";

/// Opens the address on OpenStreetMap: an action that lands on another site, as a payment
/// provider's checkout would once the action opened a session for it.
pub struct ShowOnMap;

impl Action<AddressResource> for ShowOnMap {
    type Input = ();
    const NAME: &'static str = "show-on-map";

    fn declare() -> ActionDef<AddressResource> {
        ActionDef::new()
            .places(Places::RECORD)
            .icon(iconify_icon!("lucide:map-pin"))
            .can_run(|_, address: &Address| address.street.is_some() || address.city.is_some())
    }

    async fn run(
        _cx: &Cx,
        addresses: &[Address],
        (): (),
        _ex: &mut dyn toasty::Executor,
    ) -> Result<Outcome, WriteError<()>> {
        let Some(address) = addresses.first() else {
            return Ok(Outcome::done());
        };
        let country = address
            .country
            .map(|country| format!("{country:?}").to_uppercase());
        let query = [&address.street, &address.zip, &address.city, &country]
            .into_iter()
            .flatten()
            .map(String::as_str)
            .collect::<Vec<_>>()
            .join(", ");
        let query: String = form_urlencoded::byte_serialize(query.as_bytes()).collect();
        Ok(Outcome::done().redirect_off_site(format!("{MAP_ORIGIN}/search?query={query}")))
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
