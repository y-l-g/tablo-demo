//! Customers, with their addresses and the email the demo pretends to send them.

use tablo::ColumnToggle;
use tablo::extend::{Column, Includes};
use tablo::prelude::*;
use topcoat::{Result, context::Cx, icon::iconify::iconify_icon};

use super::{Address, AddressResource, CountryCode, Customer};
use crate::access::{self, Domain};

/// Customers.
pub struct CustomerResource;

impl Resource for CustomerResource {
    type Model = Customer;
    type Form = CustomerForm;

    fn declare() -> ResourceDef<Self> {
        let c = CustomerForm::controls();
        ResourceDef::new()
            .icon(iconify_icon!("lucide:users"))
            .navigation_group(tablo::text!("navigation-shop"))
            .navigation_order(2)
            .policy(access::policy(Domain::Shop))
            .table(
                Table::new((
                    TextColumn::new(lens!(Customer.name))
                        .searchable()
                        .sortable(),
                    TextColumn::new(lens!(Customer.email))
                        .searchable()
                        .sortable(),
                    CustomerCountryColumn::new(),
                    TextColumn::new(lens!(Customer.phone))
                        .searchable()
                        .sortable()
                        .toggleable(),
                ))
                .default_sort(lens!(Customer.name))
                .action_layout(ActionLayout::Menu),
            )
            .detail(Detail::new(
                Section::new(tablo::text!("section-customer")).columns((
                    TextColumn::new(lens!(Customer.name)),
                    TextColumn::new(lens!(Customer.email)),
                    TextColumn::new(lens!(Customer.phone)),
                    TextColumn::new(lens!(Customer.birthday)),
                )),
            ))
            .form(Schema::new(
                Section::new(tablo::text!("section-customer")).schema((
                    Grid::new(2).schema((c.name, c.email.email().unique())),
                    Grid::new(2).schema((c.phone.tel(), c.birthday)),
                )),
            ))
            .record_title(lens!(Customer.name))
            .global_search(
                GlobalSearch::new()
                    .detail(tablo::text!("label-email"), lens!(Customer.email))
                    .detail(tablo::text!("label-phone"), lens!(Customer.phone)),
            )
            .relation(Relation::has_many::<AddressResource>(Address::fields().customer_id()).tab())
            .action::<SendCustomerEmail>()
            .soft_delete(lens!(Customer.deleted_at))
    }
}

/// The customer's country, the first address's.
fn customer_country(customer: &Customer) -> Option<CountryCode> {
    if customer.addresses.is_unloaded() {
        return None;
    }
    customer
        .addresses
        .get()
        .first()
        .and_then(|address| address.country)
}

/// The customer list's country column: the first address's country, labelled in
/// the request's locale.
struct CustomerCountryColumn {
    includes: Includes<Customer>,
}

impl CustomerCountryColumn {
    fn new() -> Self {
        Self {
            includes: Includes::new().with(Customer::fields().addresses()),
        }
    }
}

impl Column<Customer> for CustomerCountryColumn {
    fn name(&self) -> &str {
        "country"
    }

    fn label(&self) -> Text {
        tablo::text!("field-country")
    }

    fn text(&self, cx: &Cx, customer: &Customer) -> String {
        customer_country(customer)
            .map(|country| country.label().localize(cx))
            .unwrap_or_default()
    }

    fn toggle(&self) -> ColumnToggle {
        ColumnToggle::ShownByDefault
    }

    fn includes(&self) -> Includes<Customer> {
        self.includes.clone()
    }
}

/// The customer's record form.
#[derive(tablo::RecordForm)]
#[form(model = Customer)]
pub struct CustomerForm {
    pub name: String,
    pub email: String,
    pub phone: Option<String>,
    pub birthday: Option<jiff::civil::Date>,
}

/// What `SendCustomerEmail` asks for.
#[derive(FormInput)]
pub struct EmailDraft {
    pub subject: String,
    #[form(multiline = 5)]
    pub body: String,
}

/// Reports an email to the customer, as the demo's action does without sending one.
pub struct SendCustomerEmail;

impl Action<CustomerResource> for SendCustomerEmail {
    type Input = EmailDraft;
    const NAME: &'static str = "send-email";
    const PLACES: Places = Places::ROW.with(Places::DETAIL).with(Places::EDIT);

    async fn run(
        _cx: &Cx,
        _customers: &[Customer],
        _draft: EmailDraft,
        _ex: &mut dyn toasty::Executor,
    ) -> Result<Outcome, WriteError<EmailDraftField>> {
        Ok(Outcome::done())
    }

    fn success(count: usize) -> Text {
        tablo::text!("flash-email-sent", count = count)
    }
}

/// The name of the customer `id` names, read after the write committed.
pub async fn customer_name(cx: &Cx, id: Option<u64>) -> Option<String> {
    let id = id?;
    let mut db = tablo::db::db(cx).ok()?;
    Customer::filter_by_id(id)
        .first()
        .exec(&mut db)
        .await
        .ok()
        .flatten()
        .map(|customer| customer.name)
}
