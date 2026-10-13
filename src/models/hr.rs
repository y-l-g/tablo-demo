//! The HR domain: departments, employees, leave, projects, tasks, timesheets
//! and expenses.

use jiff::{Timestamp, civil::{Date, Time}};
use serde_json::Value;
use toasty::{Deferred, Json};

use crate::enums::{
    EmploymentType, ExpenseCategory, ExpenseStatus, LeaveStatus, LeaveType, ProjectStatus,
    TaskPriority, TaskStatus,
};

/// A part of the company, which may sit under another one.
#[derive(Debug, Clone, toasty::Model, tablo::ModelColumns)]
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
    pub budget: f64,
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

/// Someone the company employs.
#[derive(Debug, Clone, toasty::Model, tablo::ModelColumns)]
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
    pub employment_type: EmploymentType,
    pub salary: Option<f64>,
    pub hourly_rate: Option<f64>,
    pub team_color: Option<String>,
    /// The skills the employee claims, which the Filament form picks from a list.
    #[default(Vec::<String>::new())]
    pub skills: Vec<String>,
    #[column(type = text)]
    pub metadata: Option<Json<Value>>,
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
    #[has_many]
    pub expenses: Deferred<Vec<Expense>>,
}

/// A request for time off.
#[derive(Debug, Clone, toasty::Model, tablo::ModelColumns)]
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

/// A body of work.
#[derive(Debug, Clone, toasty::Model, tablo::ModelColumns)]
pub struct Project {
    #[key]
    #[auto]
    pub id: u64,
    #[index]
    pub department_id: Option<u64>,
    #[belongs_to(key = department_id, references = id)]
    pub department: Deferred<Option<Department>>,
    pub name: String,
    #[unique]
    pub slug: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub status: ProjectStatus,
    pub priority: TaskPriority,
    pub budget: f64,
    pub spent: f64,
    pub estimated_hours: f64,
    pub actual_hours: f64,
    pub start_date: Date,
    pub end_date: Option<Date>,
    /// The project's plan, which the Filament form edits as builder blocks.
    #[column(type = text)]
    pub plan: Option<Json<Value>>,
    /// Set by a delete, which hides the row instead of removing it.
    pub deleted_at: Option<Timestamp>,
    #[default(Timestamp::now())]
    pub created_at: Timestamp,
    #[update(Timestamp::now())]
    pub updated_at: Timestamp,
    #[has_many]
    pub tasks: Deferred<Vec<Task>>,
    #[has_many]
    pub timesheets: Deferred<Vec<Timesheet>>,
    #[has_many]
    pub expenses: Deferred<Vec<Expense>>,
}

/// One piece of a project.
#[derive(Debug, Clone, toasty::Model, tablo::ModelColumns)]
pub struct Task {
    #[key]
    #[auto]
    pub id: u64,
    #[index]
    pub project_id: u64,
    #[belongs_to(key = project_id, references = id)]
    pub project: Deferred<Project>,
    #[index]
    pub assigned_to: Option<u64>,
    #[belongs_to(key = assigned_to, references = id)]
    pub assignee: Deferred<Option<Employee>>,
    pub title: String,
    pub description: Option<String>,
    pub status: TaskStatus,
    pub priority: TaskPriority,
    pub estimated_hours: Option<f64>,
    #[default(0.0)]
    pub actual_hours: f64,
    pub due_date: Option<Date>,
    pub completed_at: Option<Timestamp>,
    /// The labels the task carries, which the Filament form picks from a list.
    #[default(Vec::<String>::new())]
    pub labels: Vec<String>,
    #[default(0)]
    pub sort: i64,
    #[default(Timestamp::now())]
    pub created_at: Timestamp,
    #[update(Timestamp::now())]
    pub updated_at: Timestamp,
    #[has_many]
    pub timesheets: Deferred<Vec<Timesheet>>,
}

/// Time an employee booked to a project.
#[derive(Debug, Clone, toasty::Model, tablo::ModelColumns)]
pub struct Timesheet {
    #[key]
    #[auto]
    pub id: u64,
    #[index]
    pub employee_id: u64,
    #[belongs_to(key = employee_id, references = id)]
    pub employee: Deferred<Employee>,
    #[index]
    pub task_id: Option<u64>,
    #[belongs_to(key = task_id, references = id)]
    pub task: Deferred<Option<Task>>,
    #[index]
    pub project_id: u64,
    #[belongs_to(key = project_id, references = id)]
    pub project: Deferred<Project>,
    pub date: Date,
    pub hours: f64,
    pub minutes: i64,
    pub description: Option<String>,
    pub is_billable: bool,
    pub hourly_rate: f64,
    pub total_cost: f64,
    #[default(Timestamp::now())]
    pub created_at: Timestamp,
    #[update(Timestamp::now())]
    pub updated_at: Timestamp,
}

/// A line of an expense claim.
#[derive(Debug, Clone, toasty::Model, tablo::ModelColumns)]
pub struct ExpenseLine {
    #[key]
    #[auto]
    pub id: u64,
    #[index]
    pub expense_id: u64,
    #[belongs_to(key = expense_id, references = id)]
    pub expense: Deferred<Expense>,
    pub description: String,
    pub amount: f64,
    pub quantity: i64,
    pub unit_price: f64,
    pub date: Date,
    #[default(Timestamp::now())]
    pub created_at: Timestamp,
    #[update(Timestamp::now())]
    pub updated_at: Timestamp,
}

/// An employee's claim for money they spent.
#[derive(Debug, Clone, toasty::Model, tablo::ModelColumns)]
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
    pub status: ExpenseStatus,
    pub category: ExpenseCategory,
    pub description: String,
    pub total_amount: f64,
    pub currency: String,
    pub submitted_at: Option<Timestamp>,
    pub approved_at: Option<Timestamp>,
    #[index]
    pub approved_by: Option<u64>,
    #[belongs_to(key = approved_by, references = id)]
    pub approved_by_employee: Deferred<Option<Employee>>,
    pub receipt_path: Option<String>,
    pub notes: Option<String>,
    #[default(Timestamp::now())]
    pub created_at: Timestamp,
    #[update(Timestamp::now())]
    pub updated_at: Timestamp,
    #[has_many]
    pub expense_lines: Deferred<Vec<ExpenseLine>>,
}
