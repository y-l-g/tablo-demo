//! Employees: who they are, where they work, and what they are paid, which few may see.

use rust_decimal::Decimal;
use tablo::ListColumn;
use tablo::prelude::*;
use topcoat::{Result, context::Cx, icon::iconify::iconify_icon};

use super::{
    DepartmentResource, Employee, EmploymentType, LeaveRequest, LeaveRequestResource, MetadataEntry,
};
use crate::access::{self, Domain};
use crate::projects::{Timesheet, TimesheetResource};
use crate::shared::{ColorInput, suggestions};

/// The job titles the employee form suggests, which the seeded employees hold.
pub const JOB_TITLES: &[&str] = &[
    "Software Engineer",
    "Senior Engineer",
    "Designer",
    "Product Manager",
    "Data Analyst",
    "Support Specialist",
    "Account Manager",
    "Technical Writer",
    "QA Engineer",
    "DevOps Engineer",
];

/// The skills the employee form suggests, which the seeded employees claim.
pub const SKILLS: &[&str] = &[
    "Rust",
    "TypeScript",
    "React",
    "Python",
    "SQL",
    "Docker",
    "Kubernetes",
    "AWS",
    "Figma",
    "Go",
];

/// The skills the employee form suggests.
pub struct Skills;

impl tablo::Options for Skills {
    fn options() -> Vec<(String, tablo::Text)> {
        suggestions(SKILLS)
    }
}

/// Employees.
pub struct EmployeeResource;

impl Resource for EmployeeResource {
    type Model = Employee;
    type Form = EmployeeForm;

    fn declare() -> ResourceDef<Self> {
        let c = EmployeeForm::controls();
        let salary = c
            .salary
            .prefix("$")
            .suffix(tablo::text!("unit-per-year"))
            .visible_when(&c.employment_type, ["full_time", "part_time"]);
        let hourly_rate = c
            .hourly_rate
            .prefix("$")
            .suffix(tablo::text!("unit-per-hour"))
            .visible_when(&c.employment_type, ["contractor", "intern"]);
        let job_title = c.job_title.suggestions(JOB_TITLES.iter().copied());
        ResourceDef::new()
            .icon(iconify_icon!("lucide:id-card"))
            .navigation_group(tablo::text!("navigation-hr"))
            .navigation_order(0)
            .policy(access::policy(Domain::Hr))
            // What an employee is paid shows to administrators and HR managers alone, wherever
            // the panel shows employees: the list, the detail, the form, the search, the export.
            .field_access(
                lens!(Employee.salary),
                FieldAccess::new().view(access::sees_pay),
            )
            .field_access(
                lens!(Employee.hourly_rate),
                FieldAccess::new().view(access::sees_pay),
            )
            .table(employee_table())
            .detail(Detail::new(Tabs::new([
                Tab::new(tablo::text!("tab-personal"))
                    .icon(iconify_icon!("lucide:id-card"))
                    .columns(Grid::new(2).columns((
                        TextColumn::new(lens!(Employee.name)),
                        TextColumn::new(lens!(Employee.email)),
                        TextColumn::new(lens!(Employee.phone)),
                        DateColumn::new(lens!(Employee.date_of_birth)),
                        ColorColumn::new(lens!(Employee.team_color)),
                    ))),
                Tab::new(tablo::text!("tab-employment"))
                    .icon(iconify_icon!("lucide:briefcase"))
                    .columns(Grid::new(2).columns((
                        RelationColumn::of::<DepartmentResource>(relation!(Employee.department)),
                        TextColumn::new(lens!(Employee.job_title)),
                        TextColumn::new(lens!(Employee.employment_type)).badge(),
                        DateColumn::new(lens!(Employee.hire_date)),
                        TextColumn::new(lens!(Employee.salary)).money(Currency::USD),
                        ListColumn::new(lens!(Employee.skills)),
                        RepeaterColumn::new(lens!(Employee.metadata)),
                        BooleanColumn::new(lens!(Employee.is_active)),
                    ))),
            ])))
            .form(Schema::new(Tabs::new([
                Tab::new(tablo::text!("tab-personal"))
                    .icon(iconify_icon!("lucide:id-card"))
                    .schema(Grid::new(2).schema((
                        c.name,
                        c.email.email().unique(),
                        c.phone.tel(),
                        c.date_of_birth,
                        c.team_color.custom(ColorInput),
                    ))),
                Tab::new(tablo::text!("tab-employment"))
                    .icon(iconify_icon!("lucide:briefcase"))
                    .schema((
                        Grid::new(2).schema((
                            c.department_id.searchable(),
                            job_title,
                            c.employment_type.radio(),
                            c.hire_date,
                            c.skills,
                            salary,
                            hourly_rate,
                            c.is_active.switch(),
                        )),
                        c.metadata.add_label(tablo::text!("action-add-metadata")),
                    )),
            ])))
            .record_title(lens!(Employee.name))
            .global_search(
                GlobalSearch::new()
                    .detail(tablo::text!("label-job-title"), lens!(Employee.job_title))
                    .detail(tablo::text!("label-email"), lens!(Employee.email)),
            )
            .relation(Relation::has_many::<TimesheetResource>(
                Timesheet::fields().employee_id(),
            ))
            .relation(Relation::has_many::<LeaveRequestResource>(
                LeaveRequest::fields().employee_id(),
            ))
            .action::<ToggleEmployeeActive>()
            .action::<ChangeEmployeeDepartment>()
            .action::<BulkSetEmployeeActive>()
            .soft_delete(lens!(Employee.deleted_at))
    }
}

/// The employee list.
fn employee_table() -> Table<Employee> {
    Table::new((
        TextColumn::new(lens!(Employee.name))
            .searchable()
            .sortable(),
        TextColumn::new(lens!(Employee.email))
            .searchable()
            .sortable(),
        RelationColumn::of::<DepartmentResource>(relation!(Employee.department)).toggleable(),
        TextColumn::new(lens!(Employee.job_title))
            .searchable()
            .sortable()
            .toggleable(),
        TextColumn::new(lens!(Employee.employment_type)).badge(),
        TextColumn::new(lens!(Employee.salary))
            .money(Currency::USD)
            .toggleable(),
        ColorColumn::new(lens!(Employee.team_color))
            .toggleable()
            .hidden_by_default(),
        DateColumn::new(lens!(Employee.hire_date))
            .sortable()
            .toggleable(),
        BooleanColumn::new(lens!(Employee.is_active)).sortable(),
    ))
    .default_sort(lens!(Employee.name))
    .action_layout(ActionLayout::Menu)
    .option_tabs(lens!(Employee.employment_type))
    .filters((
        SelectFilter::relationship::<DepartmentResource>(lens!(Employee.department_id)),
        TernaryFilter::new(lens!(Employee.is_active)),
    ))
}

/// The employee's record form.
#[derive(tablo::RecordForm)]
#[form(model = Employee)]
pub struct EmployeeForm {
    pub name: String,
    pub email: String,
    pub phone: Option<String>,
    pub date_of_birth: Option<jiff::civil::Date>,
    pub team_color: Option<String>,
    #[form(relationship = DepartmentResource)]
    pub department_id: Option<u64>,
    pub job_title: String,
    #[form(options)]
    pub employment_type: EmploymentType,
    pub hire_date: jiff::civil::Date,
    pub salary: Option<Decimal>,
    pub hourly_rate: Option<Decimal>,
    #[form(tags, options = Skills)]
    pub skills: Vec<String>,
    #[form(repeat)]
    pub metadata: Vec<MetadataEntry>,
    pub is_active: bool,
}

/// Activates or deactivates an employee, confirming first.
pub struct ToggleEmployeeActive;

impl Action<EmployeeResource> for ToggleEmployeeActive {
    type Input = ();
    const NAME: &'static str = "toggle-active";
    const PLACES: Places = Places::ROW;
    const CONFIRM: bool = true;

    async fn run(
        _cx: &Cx,
        employees: &[Employee],
        (): (),
        ex: &mut dyn toasty::Executor,
    ) -> Result<Outcome, WriteError<std::convert::Infallible>> {
        for employee in employees {
            Employee::filter_by_id(employee.id)
                .update()
                .is_active(!employee.is_active)
                .exec(&mut *ex)
                .await?;
        }
        Ok(Outcome::done())
    }
}

/// What a department choice asks for.
#[derive(FormInput)]
pub struct DepartmentChoice {
    pub department_id: Option<u64>,
}

/// Moves an employee, or the selection, to another department.
pub struct ChangeEmployeeDepartment;

impl Action<EmployeeResource> for ChangeEmployeeDepartment {
    type Input = DepartmentChoice;
    const NAME: &'static str = "change-department";
    const PLACES: Places = Places::ROW.with(Places::BULK);

    async fn run(
        _cx: &Cx,
        employees: &[Employee],
        choice: DepartmentChoice,
        ex: &mut dyn toasty::Executor,
    ) -> Result<Outcome, WriteError<DepartmentChoiceField>> {
        Employee::filter(filter_keys(employees))
            .update()
            .department_id(choice.department_id)
            .exec(ex)
            .await?;
        Ok(Outcome::done())
    }

    fn success(count: usize) -> Text {
        tablo::text!("flash-employees-moved", count = count)
    }
}

/// What a yes or no choice asks for.
#[derive(FormInput)]
pub struct ActiveChoice {
    #[form(options = ActiveOptions)]
    pub is_active: bool,
}

/// The two answers a boolean choice offers.
pub struct ActiveOptions;

impl tablo::Options for ActiveOptions {
    fn options() -> Vec<(String, tablo::Text)> {
        vec![
            ("true".to_string(), tablo::text!("label-active")),
            ("false".to_string(), tablo::text!("label-inactive")),
        ]
    }
}

/// Activates or deactivates the selection.
pub struct BulkSetEmployeeActive;

impl Action<EmployeeResource> for BulkSetEmployeeActive {
    type Input = ActiveChoice;
    const NAME: &'static str = "bulk-set-active";
    const PLACES: Places = Places::BULK;

    fn label() -> Text {
        tablo::text!("action-bulk-set-active")
    }

    async fn run(
        _cx: &Cx,
        employees: &[Employee],
        choice: ActiveChoice,
        ex: &mut dyn toasty::Executor,
    ) -> Result<Outcome, WriteError<ActiveChoiceField>> {
        Employee::filter(filter_keys(employees))
            .update()
            .is_active(choice.is_active)
            .exec(ex)
            .await?;
        Ok(Outcome::done())
    }

    fn success(count: usize) -> Text {
        tablo::text!("flash-employees-updated", count = count)
    }
}
