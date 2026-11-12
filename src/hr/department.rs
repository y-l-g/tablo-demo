//! Departments, with their budgets and the employees they hold.

use rust_decimal::Decimal;
use tablo::prelude::*;
use topcoat::{Result, context::Cx, icon::iconify::iconify_icon};

use super::{Department, Employee, EmployeeResource};
use crate::access::{self, Domain};
use crate::shared::{ColorInput, replica_suffix};

/// Departments.
pub struct DepartmentResource;

impl Resource for DepartmentResource {
    type Model = Department;
    type Form = DepartmentForm;

    fn declare() -> ResourceDef<Self> {
        let col = Department::columns();
        let shown = Department::columns();
        let c = DepartmentForm::controls();
        ResourceDef::new()
            .icon(iconify_icon!("lucide:building"))
            .navigation_group(tablo::text!("navigation-hr"))
            .navigation_order(1)
            .policy(access::policy(Domain::Hr))
            .table(
                Table::new((
                    col.name.text().searchable().sortable(),
                    RelationColumn::of::<DepartmentResource>(relation!(Department.parent))
                        .toggleable(),
                    CountColumn::of::<EmployeeResource>(relation!(Department.employees))
                        .label(tablo::text!("label-headcount")),
                    col.budget.text().money(Currency::USD),
                    col.color.color().toggleable(),
                    col.is_active.boolean(),
                ))
                .default_sort(lens!(Department.name))
                .action_layout(ActionLayout::Menu),
            )
            .detail(Detail::new(
                Section::new(tablo::text!("section-department")).columns((
                    shown.name.text(),
                    shown.slug.text(),
                    RelationColumn::of::<DepartmentResource>(relation!(Department.parent)),
                    shown.description.text(),
                    shown.budget.text().money(Currency::USD),
                    shown.headcount_limit.text().numeric(0),
                    shown.color.color(),
                    shown.is_active.boolean(),
                )),
            ))
            .form(Schema::new(
                Section::new(tablo::text!("section-department")).schema((
                    Grid::new(2).schema((c.name, c.slug.disabled_on_edit())),
                    c.parent_id.label(tablo::text!("label-parent-department")),
                    c.description.multiline(3),
                    Grid::new(2).schema((c.budget.prefix("$"), c.headcount_limit)),
                    c.color.custom(ColorInput),
                    c.is_active.switch(),
                )),
            ))
            .record_title(lens!(Department.name))
            .relation(Relation::has_many::<EmployeeResource>(
                Employee::fields().department_id(),
            ))
            .action::<AdjustDepartmentBudget>()
            .action::<ToggleDepartmentActive>()
            .action::<ReplicateDepartment>()
    }
}

/// The department's record form.
#[derive(tablo::RecordForm)]
#[form(model = Department)]
pub struct DepartmentForm {
    pub name: String,
    pub slug: String,
    #[form(relationship = DepartmentResource)]
    pub parent_id: Option<u64>,
    pub description: Option<String>,
    pub budget: Decimal,
    pub headcount_limit: i64,
    pub color: Option<String>,
    pub is_active: bool,
}

/// What `AdjustDepartmentBudget` asks for.
#[derive(FormInput)]
pub struct BudgetChange {
    pub budget: Decimal,
}

/// Sets a department's budget.
pub struct AdjustDepartmentBudget;

impl Action<DepartmentResource> for AdjustDepartmentBudget {
    type Input = BudgetChange;
    const NAME: &'static str = "adjust-budget";

    fn declare() -> ActionDef<DepartmentResource> {
        ActionDef::new()
            .places(Places::ROW)
            .success(|_| tablo::text!("flash-budget-saved"))
    }

    async fn run(
        _cx: &Cx,
        departments: &[Department],
        change: BudgetChange,
        ex: &mut dyn toasty::Executor,
    ) -> Result<Outcome, WriteError<BudgetChange>> {
        Department::filter(filter_keys(departments))
            .update()
            .budget(change.budget)
            .exec(ex)
            .await?;
        Ok(Outcome::done())
    }
}

/// Copies a department under a fresh slug, confirming first.
pub struct ReplicateDepartment;

impl Action<DepartmentResource> for ReplicateDepartment {
    type Input = ();
    const NAME: &'static str = "replicate";

    fn declare() -> ActionDef<DepartmentResource> {
        ActionDef::new()
            .places(Places::ROW)
            .confirm()
            .success(|_| tablo::text!("flash-department-replicated"))
    }

    async fn run(
        _cx: &Cx,
        departments: &[Department],
        (): (),
        ex: &mut dyn toasty::Executor,
    ) -> Result<Outcome, WriteError<()>> {
        for department in departments {
            toasty::create!(Department {
                parent_id: department.parent_id,
                name: department.name.clone(),
                slug: format!("{}-{}", department.slug, replica_suffix().to_lowercase()),
                description: department.description.clone(),
                budget: department.budget,
                headcount_limit: department.headcount_limit,
                color: department.color.clone(),
                is_active: department.is_active,
            })
            .exec(&mut *ex)
            .await?;
        }
        Ok(Outcome::done())
    }
}

/// Activates or deactivates a department.
pub struct ToggleDepartmentActive;

impl Action<DepartmentResource> for ToggleDepartmentActive {
    type Input = ();
    const NAME: &'static str = "toggle-active";

    fn declare() -> ActionDef<DepartmentResource> {
        ActionDef::new().places(Places::ROW)
    }

    async fn run(
        _cx: &Cx,
        departments: &[Department],
        (): (),
        ex: &mut dyn toasty::Executor,
    ) -> Result<Outcome, WriteError<()>> {
        for department in departments {
            Department::filter_by_id(department.id)
                .update()
                .is_active(!department.is_active)
                .exec(&mut *ex)
                .await?;
        }
        Ok(Outcome::done())
    }
}
