//! HR's tables: departments, employees, their leave and their expense claims, with the closed
//! value sets their columns store.
//!
//! Toasty stores each enum variant under its `snake_case` name. Its label is the
//! `option-{enum}.{variant}` message, which the catalogs under `locales/` translate and which
//! falls back to the variant in sentence case.

use jiff::{
    Timestamp,
    civil::{Date, Time},
};
use rust_decimal::Decimal;
use toasty::Deferred;

use crate::projects::{Project, Task, Timesheet};

/// A part of the company, which may sit under another one.
#[derive(Debug, Clone, toasty::Model, tablo::PanelModel)]
pub struct Department {
    #[key]
    #[auto]
    pub id: u64,
    #[index]
    pub parent_id: Option<u64>,
    #[belongs_to(key = parent_id, references = id)]
    pub parent: Deferred<Option<Department>>,
    pub name: String,
    #[unique]
    pub slug: String,
    pub description: Option<String>,
    #[default(Decimal::ZERO)]
    pub budget: Decimal,
    #[default(0)]
    pub headcount_limit: i64,
    pub color: Option<String>,
    #[default(true)]
    pub is_active: bool,
    #[default(Timestamp::now())]
    pub created_at: Timestamp,
    #[update(Timestamp::now())]
    pub updated_at: Timestamp,
    #[has_many(pair = parent)]
    pub children: Deferred<Vec<Department>>,
    #[has_many]
    pub employees: Deferred<Vec<Employee>>,
    #[has_many]
    pub projects: Deferred<Vec<Project>>,
}

/// One key and value of an employee's metadata.
#[derive(Debug, Clone, toasty::Embed, tablo::FormInput)]
pub struct MetadataEntry {
    pub key: String,
    pub value: String,
}

/// Someone the company employs.
#[derive(Debug, Clone, toasty::Model, tablo::PanelModel)]
pub struct Employee {
    #[key]
    #[auto]
    pub id: u64,
    #[index]
    pub department_id: Option<u64>,
    #[belongs_to(key = department_id, references = id)]
    pub department: Deferred<Option<Department>>,
    pub name: String,
    #[unique]
    pub email: String,
    pub phone: Option<String>,
    pub date_of_birth: Option<Date>,
    pub hire_date: Date,
    pub job_title: String,
    #[default(EmploymentType::FullTime)]
    pub employment_type: EmploymentType,
    pub salary: Option<Decimal>,
    pub hourly_rate: Option<Decimal>,
    pub team_color: Option<String>,
    /// The skills the employee claims.
    #[default(Vec::<String>::new())]
    pub skills: Vec<String>,
    /// The employee's key-value details, such as a certification or an office.
    #[document]
    #[default(Vec::<MetadataEntry>::new())]
    pub metadata: Vec<MetadataEntry>,
    pub is_active: bool,
    /// Set by a delete, which hides the row instead of removing it.
    pub deleted_at: Option<Timestamp>,
    #[default(Timestamp::now())]
    pub created_at: Timestamp,
    #[update(Timestamp::now())]
    pub updated_at: Timestamp,
    #[has_many(pair = employee)]
    pub leave_requests: Deferred<Vec<LeaveRequest>>,
    #[has_many(pair = approver)]
    pub approved_leave_requests: Deferred<Vec<LeaveRequest>>,
    #[has_many(pair = assignee)]
    pub tasks: Deferred<Vec<Task>>,
    #[has_many]
    pub timesheets: Deferred<Vec<Timesheet>>,
    #[has_many(pair = employee)]
    pub expenses: Deferred<Vec<Expense>>,
}

/// A request for time off.
#[derive(Debug, Clone, toasty::Model, tablo::PanelModel)]
pub struct LeaveRequest {
    #[key]
    #[auto]
    pub id: u64,
    #[index]
    pub employee_id: u64,
    #[belongs_to(key = employee_id, references = id)]
    pub employee: Deferred<Employee>,
    #[index]
    pub approver_id: Option<u64>,
    #[belongs_to(key = approver_id, references = id)]
    pub approver: Deferred<Option<Employee>>,
    #[column("type")]
    pub kind: LeaveType,
    #[default(LeaveStatus::Pending)]
    pub status: LeaveStatus,
    pub start_date: Date,
    pub end_date: Date,
    pub start_time: Option<Time>,
    pub end_time: Option<Time>,
    pub days_requested: f64,
    pub reason: String,
    pub reviewer_notes: Option<String>,
    pub reviewed_at: Option<Timestamp>,
    #[default(Timestamp::now())]
    pub created_at: Timestamp,
    #[update(Timestamp::now())]
    pub updated_at: Timestamp,
}

impl LeaveRequest {
    /// The days a request from `start` to `end` covers, counting both ends.
    pub fn days_between(start: Date, end: Date) -> f64 {
        let days = end.since(start).map(|span| span.get_days()).unwrap_or(0) + 1;
        f64::from(days.max(1))
    }
}

/// A line of an expense claim.
#[derive(Debug, Clone, toasty::Model, tablo::PanelModel)]
pub struct ExpenseLine {
    #[key]
    #[auto]
    pub id: u64,
    #[index]
    pub expense_id: u64,
    #[belongs_to(key = expense_id, references = id)]
    pub expense: Deferred<Expense>,
    pub description: String,
    pub amount: Decimal,
    #[default(1)]
    pub quantity: i64,
    pub unit_price: Decimal,
    pub date: Date,
    #[default(Timestamp::now())]
    pub created_at: Timestamp,
    #[update(Timestamp::now())]
    pub updated_at: Timestamp,
}

impl ExpenseLine {
    /// What `quantity` at `unit_price` comes to, rounded to the cent.
    pub fn amount_of(quantity: i64, unit_price: Decimal) -> Decimal {
        (Decimal::from(quantity) * unit_price).round_dp(2)
    }
}

/// An employee's claim for money they spent.
#[derive(Debug, Clone, toasty::Model, tablo::PanelModel)]
pub struct Expense {
    #[key]
    #[auto]
    pub id: u64,
    #[index]
    pub employee_id: u64,
    #[belongs_to(key = employee_id, references = id)]
    pub employee: Deferred<Employee>,
    #[index]
    pub project_id: Option<u64>,
    #[belongs_to(key = project_id, references = id)]
    pub project: Deferred<Option<Project>>,
    #[unique]
    pub expense_number: String,
    #[default(ExpenseStatus::Draft)]
    pub status: ExpenseStatus,
    pub category: ExpenseCategory,
    pub description: String,
    pub total_amount: Decimal,
    #[default("USD".to_string())]
    pub currency: String,
    pub submitted_at: Option<Timestamp>,
    pub approved_at: Option<Timestamp>,
    #[index]
    pub approved_by: Option<u64>,
    #[belongs_to(key = approved_by, references = id)]
    pub approved_by_employee: Deferred<Option<Employee>>,
    #[default(String::new())]
    pub receipt_path: String,
    pub notes: Option<String>,
    #[default(Timestamp::now())]
    pub created_at: Timestamp,
    #[update(Timestamp::now())]
    pub updated_at: Timestamp,
    #[has_many]
    pub expense_lines: Deferred<Vec<ExpenseLine>>,
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
