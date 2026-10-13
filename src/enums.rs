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
    #[option(label = "Full Time")]
    FullTime,
    #[option(label = "Part Time")]
    PartTime,
    Contractor,
    Intern,
}

/// What an expense was for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, toasty::Embed, tablo::Options)]
pub enum ExpenseCategory {
    Travel,
    Meals,
    Supplies,
    Equipment,
    Software,
    Other,
}

/// Where an expense sits in its approval flow.
#[derive(Debug, Clone, Copy, PartialEq, Eq, toasty::Embed, tablo::Options)]
pub enum ExpenseStatus {
    Draft,
    Submitted,
    Approved,
    Rejected,
    Reimbursed,
}

/// Where a leave request sits in its approval flow.
#[derive(Debug, Clone, Copy, PartialEq, Eq, toasty::Embed, tablo::Options)]
pub enum LeaveStatus {
    Pending,
    Approved,
    Rejected,
    Taken,
    Cancelled,
}

/// What a leave request is for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, toasty::Embed, tablo::Options)]
pub enum LeaveType {
    #[option(label = "Annual Leave")]
    Annual,
    #[option(label = "Sick Leave")]
    Sick,
    #[option(label = "Personal Leave")]
    Personal,
    #[option(label = "Unpaid Leave")]
    Unpaid,
    #[option(label = "Parental Leave")]
    Parental,
}

/// Where an order sits in its fulfilment flow.
#[derive(Debug, Clone, Copy, PartialEq, Eq, toasty::Embed, tablo::Options)]
pub enum OrderStatus {
    New,
    Processing,
    Shipped,
    Delivered,
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
    Planning,
    Active,
    #[option(label = "On Hold")]
    OnHold,
    Completed,
    Cancelled,
}

/// How urgent a task or a project is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, toasty::Embed, tablo::Options)]
pub enum TaskPriority {
    Low,
    Medium,
    High,
    Critical,
}

/// Where a task sits in its workflow.
#[derive(Debug, Clone, Copy, PartialEq, Eq, toasty::Embed, tablo::Options)]
pub enum TaskStatus {
    Backlog,
    #[option(label = "To Do")]
    Todo,
    #[option(label = "In Progress")]
    InProgress,
    #[option(label = "In Review")]
    InReview,
    Completed,
    Cancelled,
}

/// How a product is delivered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, toasty::Embed, tablo::Options)]
pub enum ProductType {
    Deliverable,
    Downloadable,
}
