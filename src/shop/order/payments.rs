//! The payments against an order, which its page lists.

use rust_decimal::Decimal;
use tablo::prelude::*;
use topcoat::icon::iconify::iconify_icon;

use super::OrderResource;
use crate::access::{self, Domain};
use crate::shop::{CurrencyCode, Payment, PaymentMethod};

/// Payments.
pub struct PaymentResource;

impl Resource for PaymentResource {
    type Model = Payment;
    type Form = PaymentForm;

    fn declare() -> ResourceDef<Self> {
        let col = Payment::columns();
        let shown = Payment::columns();
        let c = PaymentForm::controls();
        ResourceDef::new()
            .icon(iconify_icon!("lucide:credit-card"))
            .without_navigation()
            .policy(access::policy(Domain::Shop))
            .without_global_search()
            .table(Table::new((
                RelationColumn::of::<OrderResource>(relation!(Payment.order)),
                col.reference.text().searchable(),
                col.amount.text().money(Currency::USD),
                col.provider.text().sortable(),
                col.method.text().sortable(),
            )))
            .detail(Detail::new(
                Section::new(tablo::text!("section-payment")).columns((
                    RelationColumn::of::<OrderResource>(relation!(Payment.order)),
                    shown.reference.text(),
                    shown.amount.text().money(Currency::USD),
                    shown.currency.text(),
                    shown.provider.text(),
                    shown.method.text(),
                )),
            ))
            .form(Schema::new(
                Section::new(tablo::text!("section-payment")).schema((
                    c.order_id.searchable(),
                    c.reference,
                    Grid::new(2).schema((c.amount, c.currency.searchable())),
                    c.provider,
                    c.method,
                )),
            ))
            .record_title(lens!(Payment.reference))
    }
}

/// The payment's record form.
#[derive(tablo::RecordForm)]
#[form(model = Payment)]
pub struct PaymentForm {
    #[form(relationship = OrderResource)]
    pub order_id: u64,
    pub reference: String,
    pub amount: Decimal,
    #[form(options)]
    pub currency: CurrencyCode,
    #[form(options = Providers)]
    pub provider: String,
    #[form(options)]
    pub method: PaymentMethod,
}

/// The two payment providers the demo offers.
pub struct Providers;

impl tablo::Options for Providers {
    fn options() -> Vec<(String, tablo::Text)> {
        vec![
            ("stripe".to_string(), "Stripe".into()),
            ("paypal".to_string(), "PayPal".into()),
        ]
    }
}
