//! Expense claims: the list, the form, the approval flow, and the lines a claim adds up.

mod actions;
mod lines;

pub use lines::ExpenseLineResource;

use rust_decimal::Decimal;
use tablo::prelude::*;
use topcoat::icon::iconify::iconify_icon;

use self::actions::{ApproveExpense, FlagExpense, ReimburseExpense, RejectExpense, SubmitExpense};
use super::{EmployeeResource, Expense, ExpenseCategory, ExpenseLine, ExpenseStatus};
use crate::access::{self, Domain};
use crate::projects::ProjectResource;
use crate::shared::uploads::{RECEIPT_EXTENSIONS, RECEIPT_MAX_BYTES};

/// Expenses.
pub struct ExpenseResource;

impl Resource for ExpenseResource {
    type Model = Expense;
    type Form = ExpenseForm;

    fn declare() -> ResourceDef<Self> {
        let col = Expense::columns();
        let shown = Expense::columns();
        let c = ExpenseForm::controls();
        ResourceDef::new()
            .icon(iconify_icon!("lucide:banknote"))
            .navigation_group(tablo::text!("navigation-hr"))
            .navigation_order(3)
            .navigation_badge(Expense::fields().status().eq(ExpenseStatus::Submitted))
            .policy(access::policy(Domain::Hr))
            .table(
                Table::new((
                    col.expense_number.text().searchable().sortable(),
                    RelationColumn::of::<EmployeeResource>(relation!(Expense.employee)),
                    col.category.text().badge(),
                    col.status.text().badge(),
                    col.total_amount.text().money(Currency::USD),
                    RelationColumn::of::<ProjectResource>(relation!(Expense.project)).toggleable(),
                    col.submitted_at.date().sortable().toggleable(),
                ))
                .action_layout(ActionLayout::Menu)
                .row_color(|_, expense: &Expense| match expense.status {
                    ExpenseStatus::Submitted => Some(Color::Warning),
                    ExpenseStatus::Rejected => Some(Color::Danger),
                    _ => None,
                })
                .row_color_reading(Reads::of(lens!(Expense.status)))
                .option_tabs(lens!(Expense.status))
                .filters((
                    SelectFilter::of(lens!(Expense.category)),
                    SelectFilter::relationship::<EmployeeResource>(lens!(Expense.employee_id)),
                )),
            )
            .detail(Detail::new((
                Section::new(tablo::text!("section-expense-details")).columns(
                    Grid::new(3).columns((
                        shown.expense_number.text(),
                        RelationColumn::of::<EmployeeResource>(relation!(Expense.employee)),
                        shown.category.text().badge(),
                        shown.status.text().badge(),
                        shown.total_amount.text().money(Currency::USD),
                        shown.currency.text(),
                        RelationColumn::of::<ProjectResource>(relation!(Expense.project)),
                        shown.submitted_at.date(),
                        shown.approved_at.date(),
                        shown.description.text(),
                        shown.notes.text(),
                    )),
                ),
                Section::new(tablo::text!("section-receipt")).columns(shown.receipt_path.file()),
            )))
            // Who spent what on what first, then the amount, the receipt and the status.
            .form(Schema::new(Wizard::new([
                Step::new(tablo::text!("step-details")).schema(Grid::new(2).schema((
                    c.expense_number.disabled_on_edit(),
                    c.employee_id.searchable(),
                    c.project_id.searchable(),
                    c.category.radio().span_full(),
                    c.description.multiline(3).span_full(),
                ))),
                Step::new(tablo::text!("step-review")).schema(
                    Grid::new(2).schema((
                        c.total_amount.prefix("$"),
                        c.currency.choice().options(["USD"]),
                        c.status.buttons().span_full(),
                        c.receipt_path
                            .extensions(RECEIPT_EXTENSIONS)
                            .max_bytes(RECEIPT_MAX_BYTES),
                        c.notes.multiline(3),
                    )),
                ),
            ])))
            .record_title(lens!(Expense.expense_number))
            .relation(
                Relation::has_many::<ExpenseLineResource>(ExpenseLine::fields().expense_id()).tab(),
            )
            .action::<SubmitExpense>()
            .action::<ApproveExpense>()
            .action::<RejectExpense>()
            .action::<ReimburseExpense>()
            .action::<FlagExpense>()
    }
}

/// The expense's record form.
#[derive(tablo::RecordForm)]
#[form(model = Expense)]
pub struct ExpenseForm {
    pub expense_number: String,
    #[form(relationship = EmployeeResource)]
    pub employee_id: u64,
    #[form(relationship = ProjectResource)]
    pub project_id: Option<u64>,
    #[form(options)]
    pub category: ExpenseCategory,
    #[form(options)]
    pub status: ExpenseStatus,
    pub description: String,
    pub total_amount: Decimal,
    pub currency: String,
    #[form(file, optional)]
    pub receipt_path: String,
    pub notes: Option<String>,
}
