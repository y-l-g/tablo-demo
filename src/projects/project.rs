//! Projects: their budgets and hours, and the actions that move one through its lifecycle.

use jiff::Zoned;
use rust_decimal::Decimal;
use tablo::prelude::*;
use topcoat::{Result, context::Cx, icon::iconify::iconify_icon};

use super::{
    Project, ProjectStatus, Task, TaskPriority, TaskResource, Timesheet, TimesheetResource,
};
use crate::access::{self, Domain};
use crate::hr::DepartmentResource;
use crate::shared::ColorInput;

/// Projects.
pub struct ProjectResource;

impl Resource for ProjectResource {
    type Model = Project;
    type Form = ProjectForm;

    fn declare() -> ResourceDef<Self> {
        let c = ProjectForm::controls();
        let start_date = c.start_date;
        let end_date = c.end_date.gte(&start_date);
        ResourceDef::new()
            .icon(iconify_icon!("lucide:folder"))
            .navigation_group(tablo::text!("navigation-projects"))
            .navigation_order(0)
            .navigation_badge(Project::fields().status().eq(ProjectStatus::Active))
            .policy(access::policy(Domain::Projects))
            .table(
                Table::new((
                    TextColumn::new(lens!(Project.name)).searchable().sortable(),
                    TextColumn::new(lens!(Project.status)).badge(),
                    TextColumn::new(lens!(Project.priority)).badge(),
                    RelationColumn::of::<DepartmentResource>(relation!(Project.department))
                        .toggleable(),
                    TextColumn::new(lens!(Project.budget)).money(Currency::USD),
                    TextColumn::new(lens!(Project.spent)).money(Currency::USD),
                    ComputedColumn::new(tablo::text!("label-progress"), |project: &Project| {
                        format!("{:.0}%", project.progress())
                    }),
                    DateColumn::new(lens!(Project.start_date))
                        .sortable()
                        .toggleable(),
                    DateColumn::new(lens!(Project.end_date))
                        .sortable()
                        .toggleable(),
                ))
                .default_sort_desc(lens!(Project.start_date))
                .action_layout(ActionLayout::Menu)
                .filters((
                    SelectFilter::of(lens!(Project.status)),
                    SelectFilter::of(lens!(Project.priority)),
                    SelectFilter::relationship::<DepartmentResource>(lens!(Project.department_id)),
                )),
            )
            .detail(Detail::new(Tabs::new([
                Tab::new(tablo::text!("tab-overview")).columns(Grid::new(2).columns((
                    TextColumn::new(lens!(Project.name)),
                    TextColumn::new(lens!(Project.slug)),
                    RelationColumn::of::<DepartmentResource>(relation!(Project.department)),
                    TextColumn::new(lens!(Project.status)).badge(),
                    TextColumn::new(lens!(Project.priority)).badge(),
                    ColorColumn::new(lens!(Project.color)),
                    DateColumn::new(lens!(Project.start_date)),
                    DateColumn::new(lens!(Project.end_date)),
                    TextColumn::new(lens!(Project.description)),
                ))),
                Tab::new(tablo::text!("tab-budget")).columns(Grid::new(2).columns(
                    (
                        TextColumn::new(lens!(Project.budget)).money(Currency::USD),
                        TextColumn::new(lens!(Project.spent)).money(Currency::USD),
                        TextColumn::new(lens!(Project.estimated_hours)).format(|hours| {
                            tablo::text!("value-hours", value = format!("{hours:.1}"))
                        }),
                        TextColumn::new(lens!(Project.actual_hours)).format(|hours| {
                            tablo::text!("value-hours", value = format!("{hours:.1}"))
                        }),
                    ),
                )),
            ])))
            .form(Schema::new(Tabs::new([
                Tab::new(tablo::text!("tab-overview")).schema(Grid::new(2).schema((
                    c.name,
                    c.slug.disabled_on_edit(),
                    c.description.multiline(4),
                    c.department_id.searchable(),
                    c.status.buttons(),
                    c.priority.radio(),
                    c.color.custom(ColorInput),
                    start_date,
                    end_date,
                ))),
                Tab::new(tablo::text!("tab-budget")).schema(Grid::new(2).schema((
                    c.budget.prefix("$"),
                    c.spent.prefix("$"),
                    c.estimated_hours.suffix(tablo::text!("unit-hours")),
                    c.actual_hours.suffix(tablo::text!("unit-hours")),
                ))),
            ])))
            .record_title(lens!(Project.name))
            .relation(Relation::has_many::<TaskResource>(
                Task::fields().project_id(),
            ))
            .relation(Relation::has_many::<TimesheetResource>(
                Timesheet::fields().project_id(),
            ))
            .action::<ChangeProjectStatus>()
            .action::<HoldProject>()
            .action::<ResumeProject>()
            .action::<CompleteProject>()
            .soft_delete(lens!(Project.deleted_at))
    }
}

/// The project's record form.
#[derive(tablo::RecordForm)]
#[form(model = Project)]
pub struct ProjectForm {
    pub name: String,
    pub slug: String,
    #[form(relationship = DepartmentResource)]
    pub department_id: Option<u64>,
    pub description: Option<String>,
    pub color: Option<String>,
    #[form(options)]
    pub status: ProjectStatus,
    #[form(options)]
    pub priority: TaskPriority,
    pub budget: Decimal,
    pub spent: Decimal,
    pub estimated_hours: f64,
    pub actual_hours: f64,
    pub start_date: jiff::civil::Date,
    pub end_date: Option<jiff::civil::Date>,
}

/// What a status change asks for.
#[derive(FormInput)]
pub struct StatusChange {
    #[form(options)]
    pub status: ProjectStatus,
}

/// Sets a project's status.
pub struct ChangeProjectStatus;

impl Action<ProjectResource> for ChangeProjectStatus {
    type Input = StatusChange;
    const NAME: &'static str = "change-status";
    const PLACES: Places = Places::ROW.with(Places::DETAIL);

    async fn run(
        _cx: &Cx,
        projects: &[Project],
        change: StatusChange,
        ex: &mut dyn toasty::Executor,
    ) -> Result<Outcome, WriteError<StatusChangeField>> {
        set_project_status(projects, change.status, ex).await
    }

    fn success(_count: usize) -> Text {
        tablo::text!("flash-project-saved")
    }
}

/// Pauses work on an active project, confirming first.
pub struct HoldProject;

impl Action<ProjectResource> for HoldProject {
    type Input = ();
    const NAME: &'static str = "put-on-hold";
    const PLACES: Places = Places::ROW.with(Places::DETAIL);
    const CONFIRM: bool = true;

    fn can_run(_cx: &Cx, project: &Project) -> bool {
        project.status == ProjectStatus::Active
    }

    async fn run(
        _cx: &Cx,
        projects: &[Project],
        (): (),
        ex: &mut dyn toasty::Executor,
    ) -> Result<Outcome, WriteError<std::convert::Infallible>> {
        set_project_status(projects, ProjectStatus::OnHold, ex).await
    }

    fn success(_count: usize) -> Text {
        tablo::text!("flash-project-on-hold")
    }
}

/// Resumes a project that is on hold.
pub struct ResumeProject;

impl Action<ProjectResource> for ResumeProject {
    type Input = ();
    const NAME: &'static str = "resume";
    const PLACES: Places = Places::ROW.with(Places::DETAIL);

    fn can_run(_cx: &Cx, project: &Project) -> bool {
        project.status == ProjectStatus::OnHold
    }

    async fn run(
        _cx: &Cx,
        projects: &[Project],
        (): (),
        ex: &mut dyn toasty::Executor,
    ) -> Result<Outcome, WriteError<std::convert::Infallible>> {
        set_project_status(projects, ProjectStatus::Active, ex).await
    }

    fn success(_count: usize) -> Text {
        tablo::text!("flash-project-resumed")
    }
}

/// Completes an active or paused project, confirming first.
pub struct CompleteProject;

impl Action<ProjectResource> for CompleteProject {
    type Input = ();
    const NAME: &'static str = "complete";
    const PLACES: Places = Places::ROW.with(Places::DETAIL);
    const CONFIRM: bool = true;

    fn can_run(_cx: &Cx, project: &Project) -> bool {
        matches!(
            project.status,
            ProjectStatus::Active | ProjectStatus::OnHold
        )
    }

    async fn run(
        _cx: &Cx,
        projects: &[Project],
        (): (),
        ex: &mut dyn toasty::Executor,
    ) -> Result<Outcome, WriteError<std::convert::Infallible>> {
        for project in projects {
            Project::filter_by_id(project.id)
                .update()
                .status(ProjectStatus::Completed)
                .end_date(Some(Zoned::now().date()))
                .exec(&mut *ex)
                .await?;
        }
        Ok(Outcome::done())
    }

    fn success(_count: usize) -> Text {
        tablo::text!("flash-project-completed")
    }
}

/// Moves every project of `projects` to `status`.
async fn set_project_status<K>(
    projects: &[Project],
    status: ProjectStatus,
    ex: &mut dyn toasty::Executor,
) -> Result<Outcome, WriteError<K>> {
    for project in projects {
        Project::filter_by_id(project.id)
            .update()
            .status(status)
            .exec(&mut *ex)
            .await?;
    }
    Ok(Outcome::done())
}
