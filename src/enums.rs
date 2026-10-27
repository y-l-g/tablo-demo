//! The closed value sets the demo's columns store, one `Options` enum each.
//!
//! Toasty stores a variant under its `snake_case` name.
//!
//! Each label is a `Text` message, `option-{enum}.{variant}`, which the
//! catalogs under `locales/` translate and which falls back to the variant in
//! sentence case.

/// Where a customer or an address is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, toasty::Embed, tablo::Options)]
pub enum CountryCode {
    Us,
    Gb,
    De,
    Fr,
    Ca,
    Au,
    Nl,
    Br,
    Jp,
    In,
}

/// The currency a payment or a reimbursement is in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, toasty::Embed, tablo::Options)]
pub enum CurrencyCode {
    Usd,
    Eur,
    Gbp,
    Cad,
    Aud,
    Jpy,
    Brl,
    Inr,
}

/// How an employee is engaged.
#[derive(Debug, Clone, Copy, PartialEq, Eq, toasty::Embed, tablo::Options)]
pub enum EmploymentType {
    #[option(color = "success")]
    FullTime,
    #[option(color = "info")]
    PartTime,
    #[option(color = "warning")]
    Contractor,
    #[option(color = "gray")]
    Intern,
}

/// What an expense was for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, toasty::Embed, tablo::Options)]
pub enum ExpenseCategory {
    #[option(color = "info")]
    Travel,
    #[option(color = "success")]
    Meals,
    #[option(color = "warning")]
    Supplies,
    #[option(color = "primary")]
    Equipment,
    #[option(color = "gray")]
    Software,
    #[option(color = "gray")]
    Other,
}

/// Where an expense sits in its approval flow.
#[derive(Debug, Clone, Copy, PartialEq, Eq, toasty::Embed, tablo::Options)]
pub enum ExpenseStatus {
    #[option(color = "gray")]
    Draft,
    #[option(color = "info")]
    Submitted,
    #[option(color = "success")]
    Approved,
    #[option(color = "danger")]
    Rejected,
    #[option(color = "primary")]
    Reimbursed,
}

/// Where a leave request sits in its approval flow.
#[derive(Debug, Clone, Copy, PartialEq, Eq, toasty::Embed, tablo::Options)]
pub enum LeaveStatus {
    #[option(color = "warning")]
    Pending,
    #[option(color = "success")]
    Approved,
    #[option(color = "danger")]
    Rejected,
    #[option(color = "info")]
    Taken,
    #[option(color = "gray")]
    Cancelled,
}

/// What a leave request is for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, toasty::Embed, tablo::Options)]
pub enum LeaveType {
    #[option(color = "success")]
    Annual,
    #[option(color = "danger")]
    Sick,
    #[option(color = "info")]
    Personal,
    #[option(color = "warning")]
    Unpaid,
    #[option(color = "primary")]
    Parental,
}

/// Where an order sits in its fulfilment flow.
#[derive(Debug, Clone, Copy, PartialEq, Eq, toasty::Embed, tablo::Options)]
pub enum OrderStatus {
    #[option(color = "info")]
    New,
    #[option(color = "warning")]
    Processing,
    #[option(color = "success")]
    Shipped,
    #[option(color = "success")]
    Delivered,
    #[option(color = "danger")]
    Cancelled,
}

/// How a payment was made.
#[derive(Debug, Clone, Copy, PartialEq, Eq, toasty::Embed, tablo::Options)]
pub enum PaymentMethod {
    CreditCard,
    BankTransfer,
    Paypal,
    ApplePay,
    GooglePay,
}

/// Where a project sits in its lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, toasty::Embed, tablo::Options)]
pub enum ProjectStatus {
    #[option(color = "gray")]
    Planning,
    #[option(color = "success")]
    Active,
    #[option(color = "warning")]
    OnHold,
    #[option(color = "info")]
    Completed,
    #[option(color = "danger")]
    Cancelled,
}

/// How urgent a task or a project is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, toasty::Embed, tablo::Options)]
pub enum TaskPriority {
    #[option(color = "gray")]
    Low,
    #[option(color = "info")]
    Medium,
    #[option(color = "warning")]
    High,
    #[option(color = "danger")]
    Critical,
}

/// Where a task sits in its workflow.
#[derive(Debug, Clone, Copy, PartialEq, Eq, toasty::Embed, tablo::Options)]
pub enum TaskStatus {
    #[option(color = "gray")]
    Backlog,
    #[option(color = "info")]
    Todo,
    #[option(color = "warning")]
    InProgress,
    #[option(color = "primary")]
    InReview,
    #[option(color = "success")]
    Completed,
    #[option(color = "danger")]
    Cancelled,
}

/// How a product is delivered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, toasty::Embed, tablo::Options)]
pub enum ProductType {
    Deliverable,
    Downloadable,
}

/// What a member of staff does, and so what the `admin` panel lets them do. An account without
/// one, such as a sign-up, is a customer of the `app` panel alone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, toasty::Embed, tablo::Options)]
pub enum StaffRole {
    #[option(color = "danger")]
    Administrator,
    #[option(color = "info")]
    ShopManager,
    #[option(color = "success")]
    HrManager,
    #[option(color = "gray")]
    Viewer,
}
