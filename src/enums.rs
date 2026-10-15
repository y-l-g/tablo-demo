//! The closed value sets the demo's columns store, one `Options` enum each.
//!
//! Toasty stores a variant under its `snake_case` name, which is the value the
//! Filament demo stores in the same column.

/// Where a customer or an address is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, toasty::Embed, tablo::Options)]
pub enum CountryCode {
    #[option(label = "United States")]
    Us,
    #[option(label = "United Kingdom")]
    Gb,
    #[option(label = "Germany")]
    De,
    #[option(label = "France")]
    Fr,
    #[option(label = "Canada")]
    Ca,
    #[option(label = "Australia")]
    Au,
    #[option(label = "Netherlands")]
    Nl,
    #[option(label = "Brazil")]
    Br,
    #[option(label = "Japan")]
    Jp,
    #[option(label = "India")]
    In,
}

/// The currency a payment or a reimbursement is in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, toasty::Embed, tablo::Options)]
pub enum CurrencyCode {
    #[option(label = "US Dollar")]
    Usd,
    #[option(label = "Euro")]
    Eur,
    #[option(label = "British Pound")]
    Gbp,
    #[option(label = "Canadian Dollar")]
    Cad,
    #[option(label = "Australian Dollar")]
    Aud,
    #[option(label = "Japanese Yen")]
    Jpy,
    #[option(label = "Brazilian Real")]
    Brl,
    #[option(label = "Indian Rupee")]
    Inr,
}

/// How an employee is engaged.
#[derive(Debug, Clone, Copy, PartialEq, Eq, toasty::Embed, tablo::Options)]
pub enum EmploymentType {
    #[option(label = "Full Time", color = "success")]
    FullTime,
    #[option(label = "Part Time", color = "info")]
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
    #[option(label = "Annual Leave", color = "success")]
    Annual,
    #[option(label = "Sick Leave", color = "danger")]
    Sick,
    #[option(label = "Personal Leave", color = "info")]
    Personal,
    #[option(label = "Unpaid Leave", color = "warning")]
    Unpaid,
    #[option(label = "Parental Leave", color = "primary")]
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
    #[option(label = "Credit Card")]
    CreditCard,
    #[option(label = "Bank Transfer")]
    BankTransfer,
    Paypal,
    #[option(label = "Apple Pay")]
    ApplePay,
    #[option(label = "Google Pay")]
    GooglePay,
}

/// Where a project sits in its lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, toasty::Embed, tablo::Options)]
pub enum ProjectStatus {
    #[option(color = "gray")]
    Planning,
    #[option(color = "success")]
    Active,
    #[option(label = "On Hold", color = "warning")]
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
    #[option(label = "To Do", color = "info")]
    Todo,
    #[option(label = "In Progress", color = "warning")]
    InProgress,
    #[option(label = "In Review", color = "primary")]
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
