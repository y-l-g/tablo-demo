//! HR: departments, the employees they hold, their leave and their expense claims.
//!
//! Its resources form the sidebar's HR group. An expense claim's lines are a tab on its page, so
//! their resource has no sidebar entry of its own.

mod dashboard;
mod department;
mod employee;
mod expense;
mod leave;
mod models;

pub use dashboard::{HoursWidget, HrDashboard};
pub use department::DepartmentResource;
pub use employee::{EmployeeResource, JOB_TITLES, SKILLS};
pub use expense::{ExpenseLineResource, ExpenseResource};
pub use leave::LeaveRequestResource;
pub use models::{
    Department, Employee, EmploymentType, Expense, ExpenseCategory, ExpenseLine, ExpenseStatus,
    LeaveRequest, LeaveStatus, LeaveType, MetadataEntry,
};
