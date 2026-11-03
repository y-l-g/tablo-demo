//! The lines of an expense claim, whose amounts add up to its total.

use rust_decimal::Decimal;
use tablo::prelude::*;
use topcoat::{Result, context::Cx, icon::iconify::iconify_icon};

use super::ExpenseResource;
use crate::access::{self, Domain};
use crate::hr::{Expense, ExpenseLine};

/// One line of an expense claim.
pub struct ExpenseLineResource;

impl Resource for ExpenseLineResource {
    type Model = ExpenseLine;
    type Form = ExpenseLineForm;

    fn declare() -> ResourceDef<Self> {
        let c = ExpenseLineForm::controls();
        ResourceDef::new()
            .icon(iconify_icon!("lucide:receipt"))
            .without_navigation()
            .policy(access::policy(Domain::Hr))
            .without_global_search()
            .table(Table::new((
                RelationColumn::of::<ExpenseResource>(relation!(ExpenseLine.expense)),
                TextColumn::new(lens!(ExpenseLine.description)).searchable(),
                TextColumn::new(lens!(ExpenseLine.quantity)).numeric(0),
                TextColumn::new(lens!(ExpenseLine.unit_price)).money(Currency::USD),
                TextColumn::new(lens!(ExpenseLine.amount)).money(Currency::USD),
                TextColumn::new(lens!(ExpenseLine.date)),
            )))
            .detail(Detail::new(
                Section::new(tablo::text!("section-expense-line")).columns((
                    RelationColumn::of::<ExpenseResource>(relation!(ExpenseLine.expense)),
                    TextColumn::new(lens!(ExpenseLine.description)),
                    TextColumn::new(lens!(ExpenseLine.quantity)).numeric(0),
                    TextColumn::new(lens!(ExpenseLine.unit_price)).money(Currency::USD),
                    TextColumn::new(lens!(ExpenseLine.amount)).money(Currency::USD),
                    TextColumn::new(lens!(ExpenseLine.date)),
                )),
            ))
            .form(Schema::new(
                Section::new(tablo::text!("section-expense-line")).schema(Grid::new(2).schema((
                    c.expense_id.searchable(),
                    c.description,
                    c.quantity,
                    c.unit_price.prefix("$"),
                    c.date,
                ))),
            ))
            .create_column(lens!(ExpenseLine.amount))
    }

    /// Stores the amount the quantity and the unit price come to.
    async fn create_record(
        _cx: &Cx,
        form: ExpenseLineForm,
        ex: &mut dyn toasty::Executor,
    ) -> Result<ExpenseLine, WriteError<ExpenseLineFormField>> {
        let amount = ExpenseLine::amount_of(form.quantity, form.unit_price);
        let line = form.into_create().amount(amount).exec(&mut *ex).await?;
        refresh_expense_total(line.expense_id, ex).await?;
        Ok(line)
    }

    /// Recomputes the amount from the stored quantity and price.
    async fn update_record(
        cx: &Cx,
        record: ExpenseLine,
        posted: tablo::Posted<ExpenseLineForm>,
        ex: &mut dyn toasty::Executor,
    ) -> Result<ExpenseLine, WriteError<ExpenseLineFormField>> {
        let previous = record.expense_id;
        let mut record = tablo::write_update::<Self>(cx, record, posted, ex).await?;
        let amount = ExpenseLine::amount_of(record.quantity, record.unit_price);
        if record.amount != amount {
            record.update().amount(amount).exec(&mut *ex).await?;
            record.amount = amount;
        }
        refresh_expense_total(previous, ex).await?;
        if record.expense_id != previous {
            refresh_expense_total(record.expense_id, ex).await?;
        }
        Ok(record)
    }

    /// Removes the line and takes it off the claim it belonged to.
    async fn delete_record(
        _cx: &Cx,
        record: &ExpenseLine,
        ex: &mut dyn toasty::Executor,
    ) -> Result<(), WriteError> {
        ExpenseLine::filter_by_id(record.id)
            .delete()
            .exec(&mut *ex)
            .await?;
        Ok(refresh_expense_total(record.expense_id, ex).await?)
    }
}

/// Sets a claim's total to what its lines now come to.
async fn refresh_expense_total(expense_id: u64, ex: &mut dyn toasty::Executor) -> Result<()> {
    let lines = ExpenseLine::filter(ExpenseLine::fields().expense_id().eq(expense_id))
        .exec(&mut *ex)
        .await?;
    let total: Decimal = lines.iter().map(|line| line.amount).sum();
    Expense::filter_by_id(expense_id)
        .update()
        .total_amount(total)
        .exec(&mut *ex)
        .await?;
    Ok(())
}

/// The expense line's record form.
#[derive(tablo::RecordForm)]
#[form(model = ExpenseLine)]
pub struct ExpenseLineForm {
    #[form(relationship = ExpenseResource)]
    pub expense_id: u64,
    pub description: String,
    pub quantity: i64,
    pub unit_price: Decimal,
    pub date: jiff::civil::Date,
}
