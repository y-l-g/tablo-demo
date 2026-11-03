//! Projects: the work the departments take on, its tasks, and the time employees book to it.
//!
//! Its resources form the sidebar's Projects group.

mod models;
mod project;
mod task;
mod timesheet;

pub use models::{Project, ProjectStatus, Task, TaskPriority, TaskStatus, Timesheet};
pub use project::ProjectResource;
pub use task::{TASK_LABELS, TaskResource};
pub use timesheet::TimesheetResource;
