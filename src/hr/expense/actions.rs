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
    const PLACES: Places = Places::ROW.with(Places::DETAIL);
    const CONFIRM: bool = true;

    /// A draft with an amount; a draft without one shows the button disabled, saying why.
    fn can_run(_cx: &Cx, expense: &Expense) -> bool {
        expense.status == ExpenseStatus::Draft && expense.total_amount > Decimal::ZERO
    }

    fn disabled_reason(_cx: &Cx, expense: &Expense) -> Option<Text> {
        (expense.status == ExpenseStatus::Draft && expense.total_amount <= Decimal::ZERO)
            .then(|| tablo::text!("reason-expense-no-amount"))
    }

    async fn run(
        _cx: &Cx,
        expenses: &[Expense],
        (): (),
        ex: &mut dyn toasty::Executor,
    ) -> Result<Outcome, WriteError<std::convert::Infallible>> {
        for expense in expenses {
            Expense::filter_by_id(expense.id)
                .update()
                .status(ExpenseStatus::Submitted)
                .submitted_at(Some(Timestamp::now()))
                .exec(&mut *ex)
                .await?;
        }
        Ok(Outcome::done())
    }

    fn success(count: usize) -> Text {
        tablo::text!("flash-expenses-submitted", count = count)
    }
}

/// Approves a submitted expense, confirming first.
pub struct ApproveExpense;

impl Action<ExpenseResource> for ApproveExpense {
    type Input = ();
    const NAME: &'static str = "approve";
    const PLACES: Places = Places::ROW.with(Places::DETAIL);
    const CONFIRM: bool = true;

    fn can_run(_cx: &Cx, expense: &Expense) -> bool {
        expense.status == ExpenseStatus::Submitted
    }

    async fn run(
        _cx: &Cx,
        expenses: &[Expense],
        (): (),
        ex: &mut dyn toasty::Executor,
    ) -> Result<Outcome, WriteError<std::convert::Infallible>> {
        for expense in expenses {
            Expense::filter_by_id(expense.id)
                .update()
                .status(ExpenseStatus::Approved)
                .approved_at(Some(Timestamp::now()))
                .exec(&mut *ex)
                .await?;
        }
        Ok(Outcome::done())
    }

    fn success(_count: usize) -> Text {
        tablo::text!("flash-expense-approved")
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
    const PLACES: Places = Places::ROW.with(Places::DETAIL);

    fn can_run(_cx: &Cx, expense: &Expense) -> bool {
        expense.status == ExpenseStatus::Submitted
    }

    async fn run(
        _cx: &Cx,
        expenses: &[Expense],
        rejection: RejectionReason,
        ex: &mut dyn toasty::Executor,
    ) -> Result<Outcome, WriteError<RejectionReasonField>> {
        for expense in expenses {
            Expense::filter_by_id(expense.id)
                .update()
                .status(ExpenseStatus::Rejected)
                .notes(Some(rejection.reason.clone()))
                .exec(&mut *ex)
                .await?;
        }
        Ok(Outcome::done())
    }

    fn success(_count: usize) -> Text {
        tablo::text!("flash-expense-rejected")
    }
}

/// Marks an approved expense reimbursed, confirming first.
pub struct ReimburseExpense;

impl Action<ExpenseResource> for ReimburseExpense {
    type Input = ();
    const NAME: &'static str = "reimburse";
    const PLACES: Places = Places::ROW.with(Places::DETAIL);
    const CONFIRM: bool = true;

    fn can_run(_cx: &Cx, expense: &Expense) -> bool {
        expense.status == ExpenseStatus::Approved
    }

    async fn run(
        _cx: &Cx,
        expenses: &[Expense],
        (): (),
        ex: &mut dyn toasty::Executor,
    ) -> Result<Outcome, WriteError<std::convert::Infallible>> {
        for expense in expenses {
            Expense::filter_by_id(expense.id)
                .update()
                .status(ExpenseStatus::Reimbursed)
                .exec(&mut *ex)
                .await?;
        }
        Ok(Outcome::done())
    }

    fn success(_count: usize) -> Text {
        tablo::text!("flash-expense-reimbursed")
    }
}

/// Returns an expense to draft, with the reason it was flagged.
pub struct FlagExpense;

impl Action<ExpenseResource> for FlagExpense {
    type Input = RejectionReason;
    const NAME: &'static str = "flag";
    const PLACES: Places = Places::ROW.with(Places::DETAIL);

    async fn run(
        _cx: &Cx,
        expenses: &[Expense],
        flag: RejectionReason,
        ex: &mut dyn toasty::Executor,
    ) -> Result<Outcome, WriteError<RejectionReasonField>> {
        for expense in expenses {
            Expense::filter_by_id(expense.id)
                .update()
                .status(ExpenseStatus::Draft)
                .notes(Some(flag.reason.clone()))
                .exec(&mut *ex)
                .await?;
        }
        Ok(Outcome::done())
    }

    fn success(_count: usize) -> Text {
        tablo::text!("flash-expense-flagged")
    }
}
