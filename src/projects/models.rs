//! The projects' tables: projects, their tasks, and the time booked to them, with the closed
//! value sets their columns store.
//!
//! Toasty stores each enum variant under its `snake_case` name. Its label is the
//! `option-{enum}.{variant}` message, which the catalogs under `locales/` translate and which
//! falls back to the variant in sentence case.

use jiff::{Span, Timestamp, Zoned, civil::Date};
use rust_decimal::Decimal;
use serde_json::Value;
use toasty::{Deferred, Json};

use crate::hr::{Department, Employee, Expense};

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
    #[default(ProjectStatus::Planning)]
    pub status: ProjectStatus,
    #[default(TaskPriority::Medium)]
    pub priority: TaskPriority,
    #[default(Decimal::ZERO)]
    pub budget: Decimal,
    #[default(Decimal::ZERO)]
    pub spent: Decimal,
    #[default(0.0)]
    pub estimated_hours: f64,
    #[default(0.0)]
    pub actual_hours: f64,
    pub start_date: Date,
    pub end_date: Option<Date>,
    /// The project's plan, stored as a JSON document.
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

impl Project {
    /// How far along the project is: the hours booked against the hours estimated, as a
    /// percentage.
    pub fn progress(&self) -> f64 {
        if self.estimated_hours > 0.0 {
            self.actual_hours / self.estimated_hours * 100.0
        } else {
            0.0
        }
    }
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
    #[default(TaskStatus::Backlog)]
    pub status: TaskStatus,
    #[default(TaskPriority::Medium)]
    pub priority: TaskPriority,
    pub estimated_hours: Option<f64>,
    #[default(0.0)]
    pub actual_hours: f64,
    pub due_date: Option<Date>,
    pub completed_at: Option<Timestamp>,
    /// The labels the task carries.
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
    #[default(0)]
    pub minutes: i64,
    pub description: Option<String>,
    #[default(true)]
    pub is_billable: bool,
    #[default(Decimal::ZERO)]
    pub hourly_rate: Decimal,
    pub total_cost: Decimal,
    #[default(Timestamp::now())]
    pub created_at: Timestamp,
    #[update(Timestamp::now())]
    pub updated_at: Timestamp,
}

impl Timesheet {
    /// What `hours` of work at `rate` an hour cost, rounded to the cent.
    pub fn cost(hours: f64, rate: Decimal) -> Decimal {
        (Decimal::from_f64_retain(hours).unwrap_or_default() * rate).round_dp(2)
    }

    /// Whether the entry is older than the last week, so it can no longer change.
    pub fn is_locked(&self) -> bool {
        let week_ago = Zoned::now()
            .date()
            .checked_sub(Span::new().days(7))
            .unwrap_or(self.date);
        self.date < week_ago
    }
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
