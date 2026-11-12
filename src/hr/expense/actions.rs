//! What moves an expense claim along its approval flow.

use jiff::Timestamp;
use rust_decimal::Decimal;
use tablo::prelude::*;
use topcoat::context::Cx;

use super::ExpenseResource;
use crate::hr::{Expense, ExpenseStatus};

/// Submits a draft expense, confirming first.
pub struct SubmitExpense;

impl Action<ExpenseResource> for SubmitExpense {
    type Input = ();
    const NAME: &'static str = "submit";

    fn declare() -> ActionDef<ExpenseResource> {
        ActionDef::new()
            .places(Places::ROW.with(Places::DETAIL))
            .confirm()
            // A draft with an amount; a draft without one shows the button disabled, saying why.
            .can_run(|_, expense: &Expense| {
                expense.status == ExpenseStatus::Draft && expense.total_amount > Decimal::ZERO
            })
            .disabled_reason(|_, expense: &Expense| {
                (expense.status == ExpenseStatus::Draft && expense.total_amount <= Decimal::ZERO)
                    .then(|| tablo::text!("reason-expense-no-amount"))
            })
            .success(|count| tablo::text!("flash-expenses-submitted", count = count))
    }

    async fn run(
        _cx: &Cx,
        expenses: &[Expense],
        (): (),
        ex: &mut dyn toasty::Executor,
    ) -> Result<Outcome, WriteError<()>> {
        Expense::filter(filter_keys(expenses))
            .update()
            .status(ExpenseStatus::Submitted)
            .submitted_at(Some(Timestamp::now()))
            .exec(ex)
            .await?;
        Ok(Outcome::done())
    }
}

/// Approves a submitted expense, confirming first.
pub struct ApproveExpense;

impl Action<ExpenseResource> for ApproveExpense {
    type Input = ();
    const NAME: &'static str = "approve";

    fn declare() -> ActionDef<ExpenseResource> {
        ActionDef::new()
            .places(Places::ROW.with(Places::DETAIL))
            .confirm()
            .can_run(|_, expense: &Expense| expense.status == ExpenseStatus::Submitted)
            .success(|_| tablo::text!("flash-expense-approved"))
    }

    async fn run(
        _cx: &Cx,
        expenses: &[Expense],
        (): (),
        ex: &mut dyn toasty::Executor,
    ) -> Result<Outcome, WriteError<()>> {
        Expense::filter(filter_keys(expenses))
            .update()
            .status(ExpenseStatus::Approved)
            .approved_at(Some(Timestamp::now()))
            .exec(ex)
            .await?;
        Ok(Outcome::done())
    }
}

/// What a rejection asks for.
#[derive(FormInput)]
pub struct RejectionReason {
    #[form(multiline = 3)]
    pub reason: String,
}

/// Rejects a submitted expense, storing the reason.
pub struct RejectExpense;

impl Action<ExpenseResource> for RejectExpense {
    type Input = RejectionReason;
    const NAME: &'static str = "reject";

    fn declare() -> ActionDef<ExpenseResource> {
        ActionDef::new()
            .places(Places::ROW.with(Places::DETAIL))
            .can_run(|_, expense: &Expense| expense.status == ExpenseStatus::Submitted)
            .success(|_| tablo::text!("flash-expense-rejected"))
    }

    async fn run(
        _cx: &Cx,
        expenses: &[Expense],
        rejection: RejectionReason,
        ex: &mut dyn toasty::Executor,
    ) -> Result<Outcome, WriteError<RejectionReason>> {
        Expense::filter(filter_keys(expenses))
            .update()
            .status(ExpenseStatus::Rejected)
            .notes(Some(rejection.reason))
            .exec(ex)
            .await?;
        Ok(Outcome::done())
    }
}

/// Marks an approved expense reimbursed, confirming first.
pub struct ReimburseExpense;

impl Action<ExpenseResource> for ReimburseExpense {
    type Input = ();
    const NAME: &'static str = "reimburse";

    fn declare() -> ActionDef<ExpenseResource> {
        ActionDef::new()
            .places(Places::ROW.with(Places::DETAIL))
            .confirm()
            .can_run(|_, expense: &Expense| expense.status == ExpenseStatus::Approved)
            .success(|_| tablo::text!("flash-expense-reimbursed"))
    }

    async fn run(
        _cx: &Cx,
        expenses: &[Expense],
        (): (),
        ex: &mut dyn toasty::Executor,
    ) -> Result<Outcome, WriteError<()>> {
        Expense::filter(filter_keys(expenses))
            .update()
            .status(ExpenseStatus::Reimbursed)
            .exec(ex)
            .await?;
        Ok(Outcome::done())
    }
}

/// Returns an expense to draft, with the reason it was flagged.
pub struct FlagExpense;

impl Action<ExpenseResource> for FlagExpense {
    type Input = RejectionReason;
    const NAME: &'static str = "flag";

    fn declare() -> ActionDef<ExpenseResource> {
        ActionDef::new()
            .places(Places::ROW.with(Places::DETAIL))
            .success(|_| tablo::text!("flash-expense-flagged"))
    }

    async fn run(
        _cx: &Cx,
        expenses: &[Expense],
        flag: RejectionReason,
        ex: &mut dyn toasty::Executor,
    ) -> Result<Outcome, WriteError<RejectionReason>> {
        Expense::filter(filter_keys(expenses))
            .update()
            .status(ExpenseStatus::Draft)
            .notes(Some(flag.reason))
            .exec(ex)
            .await?;
        Ok(Outcome::done())
    }
}
