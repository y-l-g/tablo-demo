//! Tasks: the list grouped by status, and the actions that move a task through its workflow.

use jiff::Timestamp;
use tablo::ListColumn;
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
                    TextColumn::new(lens!(Task.title)).searchable().sortable(),
                    RelationColumn::of::<ProjectResource>(relation!(Task.project)),
                    RelationColumn::of::<EmployeeResource>(relation!(Task.assignee)).toggleable(),
                    TextColumn::new(lens!(Task.status)).badge(),
                    TextColumn::new(lens!(Task.priority)).badge(),
                    ListColumn::new(lens!(Task.labels)).toggleable(),
                    TextColumn::new(lens!(Task.estimated_hours))
                        .numeric(1)
                        .sortable()
                        .toggleable(),
                    DateColumn::new(lens!(Task.due_date))
                        .sortable()
                        .toggleable(),
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
            .detail(Detail::new(
                Section::new(tablo::text!("section-task")).columns((
                    Grid::new(2).columns((
                        TextColumn::new(lens!(Task.title)),
                        RelationColumn::of::<ProjectResource>(relation!(Task.project)),
                        RelationColumn::of::<EmployeeResource>(relation!(Task.assignee)),
                        TextColumn::new(lens!(Task.status)).badge(),
                        TextColumn::new(lens!(Task.priority)).badge(),
                        TextColumn::new(lens!(Task.estimated_hours)),
                        TextColumn::new(lens!(Task.actual_hours)),
                        DateColumn::new(lens!(Task.due_date)),
                        DateColumn::new(lens!(Task.completed_at)),
                        ListColumn::new(lens!(Task.labels)),
                    )),
                    TextColumn::new(lens!(Task.description)),
                )),
            ))
            .form(Schema::new(
                Section::new(tablo::text!("section-task-details")).schema(Grid::new(2).schema((
                    c.title,
                    c.project_id.searchable(),
                    c.assigned_to.searchable(),
                    c.status.buttons(),
                    c.priority.radio(),
                    c.estimated_hours.suffix(tablo::text!("unit-hours")),
                    c.due_date,
                    c.labels,
                    c.description.multiline(4),
                    completed_at,
                ))),
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
    const PLACES: Places = Places::ROW;

    fn can_run(_cx: &Cx, task: &Task) -> bool {
        matches!(task.status, TaskStatus::Backlog | TaskStatus::Todo)
    }

    async fn run(
        _cx: &Cx,
        tasks: &[Task],
        (): (),
        ex: &mut dyn toasty::Executor,
    ) -> Result<Outcome, WriteError<std::convert::Infallible>> {
        set_task_status(tasks, TaskStatus::InProgress, ex).await
    }

    fn success(_count: usize) -> Text {
        tablo::text!("flash-task-started")
    }
}

/// Sends a started task to review.
pub struct SendTaskToReview;

impl Action<TaskResource> for SendTaskToReview {
    type Input = ();
    const NAME: &'static str = "send-to-review";
    const PLACES: Places = Places::ROW;

    fn can_run(_cx: &Cx, task: &Task) -> bool {
        task.status == TaskStatus::InProgress
    }

    async fn run(
        _cx: &Cx,
        tasks: &[Task],
        (): (),
        ex: &mut dyn toasty::Executor,
    ) -> Result<Outcome, WriteError<std::convert::Infallible>> {
        set_task_status(tasks, TaskStatus::InReview, ex).await
    }

    fn success(_count: usize) -> Text {
        tablo::text!("flash-task-sent-to-review")
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
    const PLACES: Places = Places::ROW;

    fn can_run(_cx: &Cx, task: &Task) -> bool {
        matches!(task.status, TaskStatus::InProgress | TaskStatus::InReview)
    }

    async fn run(
        _cx: &Cx,
        tasks: &[Task],
        completion: TaskCompletion,
        ex: &mut dyn toasty::Executor,
    ) -> Result<Outcome, WriteError<TaskCompletionField>> {
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

    fn success(_count: usize) -> Text {
        tablo::text!("flash-task-completed")
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
    const PLACES: Places = Places::ROW.with(Places::BULK);

    async fn run(
        _cx: &Cx,
        tasks: &[Task],
        choice: AssigneeChoice,
        ex: &mut dyn toasty::Executor,
    ) -> Result<Outcome, WriteError<AssigneeChoiceField>> {
        for task in tasks {
            Task::filter_by_id(task.id)
                .update()
                .assigned_to(choice.assigned_to)
                .exec(&mut *ex)
                .await?;
        }
        Ok(Outcome::done())
    }

    fn success(count: usize) -> Text {
        tablo::text!("flash-tasks-assigned", count = count)
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
    const PLACES: Places = Places::ROW;

    async fn run(
        _cx: &Cx,
        tasks: &[Task],
        change: PriorityChange,
        ex: &mut dyn toasty::Executor,
    ) -> Result<Outcome, WriteError<PriorityChangeField>> {
        for task in tasks {
            Task::filter_by_id(task.id)
                .update()
                .priority(change.priority)
                .exec(&mut *ex)
                .await?;
        }
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
    const PLACES: Places = Places::BULK;

    fn label() -> Text {
        tablo::text!("action-bulk-set-status")
    }

    async fn run(
        _cx: &Cx,
        tasks: &[Task],
        change: TaskStatusChange,
        ex: &mut dyn toasty::Executor,
    ) -> Result<Outcome, WriteError<TaskStatusChangeField>> {
        set_task_status(tasks, change.status, ex).await
    }

    fn success(count: usize) -> Text {
        tablo::text!("flash-tasks-updated", count = count)
    }
}

/// Copies a task, leaving out the hours booked and the completion, confirming first.
pub struct ReplicateTask;

impl Action<TaskResource> for ReplicateTask {
    type Input = ();
    const NAME: &'static str = "replicate";
    const PLACES: Places = Places::ROW;
    const CONFIRM: bool = true;

    async fn run(
        _cx: &Cx,
        tasks: &[Task],
        (): (),
        ex: &mut dyn toasty::Executor,
    ) -> Result<Outcome, WriteError<std::convert::Infallible>> {
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

    fn success(_count: usize) -> Text {
        tablo::text!("flash-task-replicated")
    }
}

/// Moves every task of `tasks` to `status`.
async fn set_task_status<K>(
    tasks: &[Task],
    status: TaskStatus,
    ex: &mut dyn toasty::Executor,
) -> Result<Outcome, WriteError<K>> {
    for task in tasks {
        Task::filter_by_id(task.id)
            .update()
            .status(status)
            .exec(&mut *ex)
            .await?;
    }
    Ok(Outcome::done())
}
