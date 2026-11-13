//! Tasks: the list grouped by status, and the actions that move a task through its workflow.

use jiff::Timestamp;
use tablo::prelude::*;
use topcoat::{Result, context::Cx, icon::iconify::iconify_icon};

use super::{ProjectResource, Task, TaskPriority, TaskStatus};
use crate::access::{self, Domain};
use crate::hr::EmployeeResource;
use crate::shared::suggestions;

/// The labels the task form suggests, which the seeded tasks carry.
pub const TASK_LABELS: &[&str] = &[
    "bug",
    "feature",
    "enhancement",
    "documentation",
    "design",
    "testing",
    "refactor",
    "urgent",
];

/// The labels the task form suggests.
pub struct TaskLabels;

impl tablo::Options for TaskLabels {
    fn options() -> Vec<(String, tablo::Text)> {
        suggestions(TASK_LABELS)
    }
}

/// Tasks.
pub struct TaskResource;

impl Resource for TaskResource {
    type Model = Task;
    type Form = TaskForm;

    fn declare() -> ResourceDef<Self> {
        let col = Task::columns();
        let shown = Task::columns();
        let c = TaskForm::controls();
        let completed_at = c.completed_at.visible_when(&c.status, ["completed"]);
        ResourceDef::new()
            .icon(iconify_icon!("lucide:list-checks"))
            .navigation_group(tablo::text!("navigation-projects"))
            .navigation_order(1)
            .navigation_badge(Task::fields().status().eq(TaskStatus::InProgress))
            .policy(access::policy(Domain::Projects))
            .table(
                Table::new((
                    col.title.text().searchable().sortable().limit(40),
                    RelationColumn::of::<ProjectResource>(relation!(Task.project)),
                    RelationColumn::of::<EmployeeResource>(relation!(Task.assignee)).toggleable(),
                    col.status.text().badge(),
                    col.priority.text().badge(),
                    col.labels.list().toggleable(),
                    col.estimated_hours
                        .text()
                        .numeric(1)
                        .sortable()
                        .toggleable(),
                    col.due_date.date().sortable().toggleable(),
                ))
                .action_layout(ActionLayout::Menu)
                .filters((
                    SelectFilter::of(lens!(Task.status)),
                    SelectFilter::of(lens!(Task.priority)),
                    SelectFilter::relationship::<ProjectResource>(lens!(Task.project_id)),
                    SelectFilter::relationship::<EmployeeResource>(lens!(Task.assigned_to)),
                ))
                .group_by(lens!(Task.status)),
            )
            // The task itself beside where it stands and when it is due, as its form lays them.
            .detail(Detail::new(Grid::new(3).columns((
                Section::new(tablo::text!("section-task")).span(2).columns((
                    shown.title.text(),
                    Grid::new(2).columns((
                        RelationColumn::of::<ProjectResource>(relation!(Task.project)),
                        RelationColumn::of::<EmployeeResource>(relation!(Task.assignee)),
                    )),
                    shown.description.text(),
                    shown.labels.list(),
                )),
                Group::new().columns((
                    Section::new(tablo::text!("section-status")).columns((
                        shown.status.text().badge(),
                        shown.priority.text().badge(),
                        shown.completed_at.date(),
                    )),
                    Section::new(tablo::text!("section-schedule")).columns((
                        shown.due_date.date(),
                        shown.estimated_hours.text(),
                        shown.actual_hours.text(),
                    )),
                )),
            ))))
            .form(Schema::new(
                Grid::new(3).schema((
                    Section::new(tablo::text!("section-task-details"))
                        .span(2)
                        .schema((
                            c.title,
                            Grid::new(2)
                                .schema((c.project_id.searchable(), c.assigned_to.searchable())),
                            c.description.multiline(6),
                            c.labels,
                        )),
                    Group::new().schema((
                        Section::new(tablo::text!("section-status")).schema((
                            c.status.buttons(),
                            c.priority.buttons(),
                            completed_at,
                        )),
                        Section::new(tablo::text!("section-schedule")).schema((
                            c.due_date,
                            c.estimated_hours.suffix(tablo::text!("unit-hours")),
                        )),
                    )),
                )),
            ))
            .record_title(lens!(Task.title))
            .action::<StartTask>()
            .action::<SendTaskToReview>()
            .action::<CompleteTask>()
            .action::<AssignTask>()
            .action::<SetTaskPriority>()
            .action::<BulkSetTaskStatus>()
            .action::<ReplicateTask>()
    }
}

/// The task's record form.
#[derive(tablo::RecordForm)]
#[form(model = Task)]
pub struct TaskForm {
    pub title: String,
    #[form(relationship = ProjectResource)]
    pub project_id: u64,
    #[form(relationship = EmployeeResource)]
    pub assigned_to: Option<u64>,
    #[form(options)]
    pub status: TaskStatus,
    #[form(options)]
    pub priority: TaskPriority,
    pub estimated_hours: Option<f64>,
    pub due_date: Option<jiff::civil::Date>,
    #[form(tags, options = TaskLabels)]
    pub labels: Vec<String>,
    pub description: Option<String>,
    pub completed_at: Option<Timestamp>,
}

/// Starts a task that is in the backlog or to do.
pub struct StartTask;

impl Action<TaskResource> for StartTask {
    type Input = ();
    const NAME: &'static str = "start";

    fn declare() -> ActionDef<TaskResource> {
        ActionDef::new()
            .places(Places::ROW)
            .can_run(|_, task: &Task| matches!(task.status, TaskStatus::Backlog | TaskStatus::Todo))
            .success(|_| tablo::text!("flash-task-started"))
    }

    async fn run(
        _cx: &Cx,
        tasks: &[Task],
        (): (),
        ex: &mut dyn toasty::Executor,
    ) -> Result<Outcome, WriteError<()>> {
        set_task_status(tasks, TaskStatus::InProgress, ex).await
    }
}

/// Sends a started task to review.
pub struct SendTaskToReview;

impl Action<TaskResource> for SendTaskToReview {
    type Input = ();
    const NAME: &'static str = "send-to-review";

    fn declare() -> ActionDef<TaskResource> {
        ActionDef::new()
            .places(Places::ROW)
            .can_run(|_, task: &Task| task.status == TaskStatus::InProgress)
            .success(|_| tablo::text!("flash-task-sent-to-review"))
    }

    async fn run(
        _cx: &Cx,
        tasks: &[Task],
        (): (),
        ex: &mut dyn toasty::Executor,
    ) -> Result<Outcome, WriteError<()>> {
        set_task_status(tasks, TaskStatus::InReview, ex).await
    }
}

/// What completing a task asks for.
#[derive(FormInput)]
pub struct TaskCompletion {
    pub actual_hours: Option<f64>,
}

/// Completes a task in progress or in review, recording the hours it took.
pub struct CompleteTask;

impl Action<TaskResource> for CompleteTask {
    type Input = TaskCompletion;
    const NAME: &'static str = "complete";

    fn declare() -> ActionDef<TaskResource> {
        ActionDef::new()
            .places(Places::ROW)
            .can_run(|_, task: &Task| {
                matches!(task.status, TaskStatus::InProgress | TaskStatus::InReview)
            })
            .success(|_| tablo::text!("flash-task-completed"))
    }

    async fn run(
        _cx: &Cx,
        tasks: &[Task],
        completion: TaskCompletion,
        ex: &mut dyn toasty::Executor,
    ) -> Result<Outcome, WriteError<TaskCompletion>> {
        for task in tasks {
            Task::filter_by_id(task.id)
                .update()
                .status(TaskStatus::Completed)
                .completed_at(Some(Timestamp::now()))
                .actual_hours(completion.actual_hours.unwrap_or(task.actual_hours))
                .exec(&mut *ex)
                .await?;
        }
        Ok(Outcome::done())
    }
}

/// What assigning a task asks for.
#[derive(FormInput)]
pub struct AssigneeChoice {
    pub assigned_to: Option<u64>,
}

/// Assigns a task, or every task of the selection, to an employee.
pub struct AssignTask;

impl Action<TaskResource> for AssignTask {
    type Input = AssigneeChoice;
    const NAME: &'static str = "assign";

    fn declare() -> ActionDef<TaskResource> {
        ActionDef::new()
            .places(Places::ROW.with(Places::BULK))
            .success(|count| tablo::text!("flash-tasks-assigned", count = count))
    }

    async fn run(
        _cx: &Cx,
        tasks: &[Task],
        choice: AssigneeChoice,
        ex: &mut dyn toasty::Executor,
    ) -> Result<Outcome, WriteError<AssigneeChoice>> {
        Task::filter(filter_keys(tasks))
            .update()
            .assigned_to(choice.assigned_to)
            .exec(ex)
            .await?;
        Ok(Outcome::done())
    }
}

/// What setting a priority asks for.
#[derive(FormInput)]
pub struct PriorityChange {
    #[form(options)]
    pub priority: TaskPriority,
}

/// Sets a task's priority.
pub struct SetTaskPriority;

impl Action<TaskResource> for SetTaskPriority {
    type Input = PriorityChange;
    const NAME: &'static str = "set-priority";

    fn declare() -> ActionDef<TaskResource> {
        ActionDef::new().places(Places::ROW)
    }

    async fn run(
        _cx: &Cx,
        tasks: &[Task],
        change: PriorityChange,
        ex: &mut dyn toasty::Executor,
    ) -> Result<Outcome, WriteError<PriorityChange>> {
        Task::filter(filter_keys(tasks))
            .update()
            .priority(change.priority)
            .exec(ex)
            .await?;
        Ok(Outcome::done())
    }
}

/// What setting a status asks for.
#[derive(FormInput)]
pub struct TaskStatusChange {
    #[form(options)]
    pub status: TaskStatus,
}

/// Sets the status of every task in the selection.
pub struct BulkSetTaskStatus;

impl Action<TaskResource> for BulkSetTaskStatus {
    type Input = TaskStatusChange;
    const NAME: &'static str = "bulk-set-status";

    fn declare() -> ActionDef<TaskResource> {
        ActionDef::new()
            .places(Places::BULK)
            .label(tablo::text!("action-bulk-set-status"))
            .success(|count| tablo::text!("flash-tasks-updated", count = count))
    }

    async fn run(
        _cx: &Cx,
        tasks: &[Task],
        change: TaskStatusChange,
        ex: &mut dyn toasty::Executor,
    ) -> Result<Outcome, WriteError<TaskStatusChange>> {
        set_task_status(tasks, change.status, ex).await
    }
}

/// Copies a task, leaving out the hours booked and the completion, confirming first.
pub struct ReplicateTask;

impl Action<TaskResource> for ReplicateTask {
    type Input = ();
    const NAME: &'static str = "replicate";

    fn declare() -> ActionDef<TaskResource> {
        ActionDef::new()
            .places(Places::ROW)
            .confirm()
            .success(|_| tablo::text!("flash-task-replicated"))
    }

    async fn run(
        _cx: &Cx,
        tasks: &[Task],
        (): (),
        ex: &mut dyn toasty::Executor,
    ) -> Result<Outcome, WriteError<()>> {
        for task in tasks {
            toasty::create!(Task {
                project_id: task.project_id,
                assigned_to: task.assigned_to,
                title: task.title.clone(),
                description: task.description.clone(),
                status: task.status,
                priority: task.priority,
                estimated_hours: task.estimated_hours,
                due_date: task.due_date,
                labels: task.labels.clone(),
            })
            .exec(&mut *ex)
            .await?;
        }
        Ok(Outcome::done())
    }
}

/// Moves every task of `tasks` to `status`.
async fn set_task_status<K: tablo::FormFields>(
    tasks: &[Task],
    status: TaskStatus,
    ex: &mut dyn toasty::Executor,
) -> Result<Outcome, WriteError<K>> {
    Task::filter(filter_keys(tasks))
        .update()
        .status(status)
        .exec(ex)
        .await?;
    Ok(Outcome::done())
}
