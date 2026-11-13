//! Projects: their budgets and hours, and the actions that move one through its lifecycle.

use jiff::Zoned;
use rust_decimal::Decimal;
use tablo::Number;
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
        let col = Project::columns();
        let shown = Project::columns();
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
                    col.name.text().searchable().sortable(),
                    col.status.text().badge(),
                    col.priority.text().badge(),
                    RelationColumn::of::<DepartmentResource>(relation!(Project.department))
                        .toggleable(),
                    col.budget.text().money(Currency::USD),
                    col.spent.text().money(Currency::USD),
                    // The reader's locale spells it: `45%`, or `45 %` in French.
                    ComputedColumn::new(tablo::text!("label-progress"), |cx, project: &Project| {
                        Number::percent_in(cx, project.progress(), 0)
                    }),
                    col.start_date.date().sortable().toggleable(),
                    col.end_date.date().sortable().toggleable(),
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
                Tab::new(tablo::text!("tab-overview"))
                    .icon(iconify_icon!("lucide:folder"))
                    .columns(Grid::new(2).columns((
                        shown.name.text(),
                        shown.slug.text(),
                        RelationColumn::of::<DepartmentResource>(relation!(Project.department)),
                        shown.status.text().badge(),
                        shown.priority.text().badge(),
                        shown.color.color(),
                        shown.start_date.date(),
                        shown.end_date.date(),
                        shown.description.text(),
                    ))),
                Tab::new(tablo::text!("tab-budget"))
                    .icon(iconify_icon!("lucide:banknote"))
                    .badge("USD")
                    .columns(Grid::new(2).columns((
                        shown.budget.text().money(Currency::USD),
                        shown.spent.text().money(Currency::USD),
                        shown.estimated_hours.text().format(|_, hours| {
                            tablo::text!("value-hours", value = format!("{hours:.1}"))
                        }),
                        shown.actual_hours.text().format(|_, hours| {
                            tablo::text!("value-hours", value = format!("{hours:.1}"))
                        }),
                    ))),
            ])))
            .form(Schema::new(Tabs::new([
                Tab::new(tablo::text!("tab-overview"))
                    .icon(iconify_icon!("lucide:folder"))
                    .schema(Grid::new(2).schema((
                        c.name,
                        c.slug.disabled_on_edit(),
                        c.description.multiline(4),
                        c.department_id.searchable(),
                        c.status.buttons(),
                        c.priority.buttons(),
                        c.color.custom(ColorInput),
                        start_date,
                        end_date,
                    ))),
                Tab::new(tablo::text!("tab-budget"))
                    .icon(iconify_icon!("lucide:banknote"))
                    .badge("USD")
                    .schema(Grid::new(2).schema((
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

    fn declare() -> ActionDef<ProjectResource> {
        ActionDef::new()
            .places(Places::ROW.with(Places::DETAIL))
            .success(|_| tablo::text!("flash-project-saved"))
    }

    async fn run(
        _cx: &Cx,
        projects: &[Project],
        change: StatusChange,
        ex: &mut dyn toasty::Executor,
    ) -> Result<Outcome, WriteError<StatusChange>> {
        set_project_status(projects, change.status, ex).await
    }
}

/// Pauses work on an active project, confirming first.
pub struct HoldProject;

impl Action<ProjectResource> for HoldProject {
    type Input = ();
    const NAME: &'static str = "put-on-hold";

    fn declare() -> ActionDef<ProjectResource> {
        ActionDef::new()
            .places(Places::ROW.with(Places::DETAIL))
            .confirm()
            .can_run(|_, project: &Project| project.status == ProjectStatus::Active)
            .success(|_| tablo::text!("flash-project-on-hold"))
    }

    async fn run(
        _cx: &Cx,
        projects: &[Project],
        (): (),
        ex: &mut dyn toasty::Executor,
    ) -> Result<Outcome, WriteError<()>> {
        set_project_status(projects, ProjectStatus::OnHold, ex).await
    }
}

/// Resumes a project that is on hold.
pub struct ResumeProject;

impl Action<ProjectResource> for ResumeProject {
    type Input = ();
    const NAME: &'static str = "resume";

    fn declare() -> ActionDef<ProjectResource> {
        ActionDef::new()
            .places(Places::ROW.with(Places::DETAIL))
            .can_run(|_, project: &Project| project.status == ProjectStatus::OnHold)
            .success(|_| tablo::text!("flash-project-resumed"))
    }

    async fn run(
        _cx: &Cx,
        projects: &[Project],
        (): (),
        ex: &mut dyn toasty::Executor,
    ) -> Result<Outcome, WriteError<()>> {
        set_project_status(projects, ProjectStatus::Active, ex).await
    }
}

/// Completes an active or paused project, confirming first.
pub struct CompleteProject;

impl Action<ProjectResource> for CompleteProject {
    type Input = ();
    const NAME: &'static str = "complete";

    fn declare() -> ActionDef<ProjectResource> {
        ActionDef::new()
            .places(Places::ROW.with(Places::DETAIL))
            .confirm()
            .can_run(|_, project: &Project| {
                matches!(
                    project.status,
                    ProjectStatus::Active | ProjectStatus::OnHold
                )
            })
            .success(|_| tablo::text!("flash-project-completed"))
    }

    async fn run(
        _cx: &Cx,
        projects: &[Project],
        (): (),
        ex: &mut dyn toasty::Executor,
    ) -> Result<Outcome, WriteError<()>> {
        Project::filter(filter_keys(projects))
            .update()
            .status(ProjectStatus::Completed)
            .end_date(Some(Zoned::now().date()))
            .exec(ex)
            .await?;
        Ok(Outcome::done())
    }
}

/// Moves every project of `projects` to `status`.
async fn set_project_status<K: tablo::FormFields>(
    projects: &[Project],
    status: ProjectStatus,
    ex: &mut dyn toasty::Executor,
) -> Result<Outcome, WriteError<K>> {
    Project::filter(filter_keys(projects))
        .update()
        .status(status)
        .exec(ex)
        .await?;
    Ok(Outcome::done())
}
