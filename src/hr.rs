//! The HR and project resources.

use jiff::{Span, Timestamp, Zoned};
use tablo::{
    Action, ActionLayout, BooleanColumn, Color, ColorColumn, ComputedColumn, CountColumn, Currency,
    DateColumn, DateFilter, Detail, FieldAccess, FileColumn, FormInput, GlobalSearch, Grid,
    Outcome, Places, Reads, RecordForm, Relation, RelationColumn, Resource, ResourceDef, Schema,
    Section, SelectFilter, Tab, Table, Tabs, TernaryFilter, Text, TextColumn, WriteError, lens,
    relation,
};
use topcoat::{Result, context::Cx};

use crate::dashboards::LeaveStatusWidget;
use crate::enums::{
    EmploymentType, ExpenseCategory, ExpenseStatus, LeaveStatus, LeaveType, ProjectStatus,
    TaskPriority, TaskStatus,
};
use crate::models::{
    Department, Employee, Expense, ExpenseLine, LeaveRequest, Project, Task, Timesheet,
    replica_suffix, round,
};
use crate::panel::{RECEIPT_EXTENSIONS, RECEIPT_MAX_BYTES};
use crate::roles::{self, Domain};
use crate::tabs::OptionTabs;

/// Departments.
pub struct DepartmentResource;

impl Resource for DepartmentResource {
    type Model = Department;
    type Form = DepartmentForm;

    fn declare() -> ResourceDef<Self> {
        let c = DepartmentForm::controls();
        ResourceDef::new()
            .icon(crate::icons::DEPARTMENTS)
            .navigation_group(tablo::text!("navigation-hr"))
            .navigation_order(1)
            .policy(roles::policy(Domain::Hr))
            .table(
                Table::new((
                    TextColumn::new(lens!(Department.name))
                        .searchable()
                        .sortable(),
                    RelationColumn::of::<DepartmentResource>(relation!(Department.parent))
                        .label(tablo::text!("label-parent"))
                        .toggleable(),
                    CountColumn::of::<EmployeeResource>(relation!(Department.employees))
                        .label(tablo::text!("label-headcount")),
                    TextColumn::new(lens!(Department.budget))
                        .money(Currency::USD)
                        .sortable(),
                    ColorColumn::new(lens!(Department.color)).toggleable(),
                    BooleanColumn::new(lens!(Department.is_active))
                        .label(tablo::text!("label-active")),
                ))
                .default_sort(lens!(Department.name))
                .action_layout(ActionLayout::Menu),
            )
            .detail(Detail::new(
                Section::new(tablo::text!("section-department")).columns((
                    TextColumn::new(lens!(Department.name)),
                    TextColumn::new(lens!(Department.slug)),
                    RelationColumn::of::<DepartmentResource>(relation!(Department.parent))
                        .label(tablo::text!("label-parent")),
                    TextColumn::new(lens!(Department.description)),
                    TextColumn::new(lens!(Department.budget)).money(Currency::USD),
                    TextColumn::new(lens!(Department.headcount_limit)).numeric(0),
                    ColorColumn::new(lens!(Department.color)),
                    BooleanColumn::new(lens!(Department.is_active))
                        .label(tablo::text!("label-active")),
                )),
            ))
            .form(Schema::new(
                Section::new(tablo::text!("section-department")).schema((
                    Grid::new(2).schema((c.name, c.slug.disabled_on_edit())),
                    c.parent_id.label(tablo::text!("label-parent-department")),
                    c.description.multiline(3),
                    Grid::new(2).schema((c.budget.prefix("$"), c.headcount_limit)),
                    c.color,
                    c.is_active.switch().label(tablo::text!("label-active")),
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

    crate::audit!();
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
    pub budget: f64,
    pub headcount_limit: i64,
    pub color: Option<String>,
    pub is_active: bool,
}

/// What `AdjustDepartmentBudget` asks for.
#[derive(FormInput)]
pub struct BudgetChange {
    pub budget: f64,
}

/// Sets a department's budget.
pub struct AdjustDepartmentBudget;

impl Action<DepartmentResource> for AdjustDepartmentBudget {
    type Input = BudgetChange;
    const NAME: &'static str = "adjust-budget";
    const PLACES: Places = Places::ROW;

    async fn run(
        _cx: &Cx,
        departments: &[Department],
        change: BudgetChange,
        ex: &mut dyn toasty::Executor,
    ) -> Result<Outcome, WriteError> {
        for department in departments {
            Department::filter(Department::fields().id().eq(department.id))
                .update()
                .budget(change.budget)
                .exec(&mut *ex)
                .await?;
        }
        Ok(Outcome::done())
    }

    fn success(_count: usize) -> Text {
        tablo::text!("flash-budget-saved")
    }
}

/// Copies a department under a fresh slug, confirming first.
pub struct ReplicateDepartment;

impl Action<DepartmentResource> for ReplicateDepartment {
    type Input = ();
    const NAME: &'static str = "replicate";
    const PLACES: Places = Places::ROW;
    const CONFIRM: bool = true;

    async fn run(
        _cx: &Cx,
        departments: &[Department],
        (): (),
        ex: &mut dyn toasty::Executor,
    ) -> Result<Outcome, WriteError> {
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

    fn success(_count: usize) -> Text {
        tablo::text!("flash-department-replicated")
    }
}

/// Activates or deactivates a department.
pub struct ToggleDepartmentActive;

impl Action<DepartmentResource> for ToggleDepartmentActive {
    type Input = ();
    const NAME: &'static str = "toggle-active";
    const PLACES: Places = Places::ROW;

    async fn run(
        _cx: &Cx,
        departments: &[Department],
        (): (),
        ex: &mut dyn toasty::Executor,
    ) -> Result<Outcome, WriteError> {
        for department in departments {
            Department::filter(Department::fields().id().eq(department.id))
                .update()
                .is_active(!department.is_active)
                .exec(&mut *ex)
                .await?;
        }
        Ok(Outcome::done())
    }
}

/// Employees.
pub struct EmployeeResource;

impl Resource for EmployeeResource {
    type Model = Employee;
    type Form = EmployeeForm;

    fn declare() -> ResourceDef<Self> {
        let c = EmployeeForm::controls();
        // `visible_when` borrows the field it watches, so the conditioned
        // controls are built before the schema takes them.
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
        let job_title = c
            .job_title
            .suggestions(crate::seed::JOB_TITLES.iter().copied());
        ResourceDef::new()
            .icon(crate::icons::EMPLOYEES)
            .navigation_group(tablo::text!("navigation-hr"))
            .navigation_order(0)
            .policy(roles::policy(Domain::Hr))
            // What an employee is paid shows to administrators and HR managers alone, wherever
            // the panel shows employees: the list, the detail, the form, the search, the export.
            .field_access(
                lens!(Employee.salary),
                FieldAccess::new().view(roles::sees_pay),
            )
            .field_access(
                lens!(Employee.hourly_rate),
                FieldAccess::new().view(roles::sees_pay),
            )
            .table(employee_table())
            .detail(Detail::new(Tabs::new([
                Tab::new(tablo::text!("tab-personal")).columns(Grid::new(2).columns((
                    TextColumn::new(lens!(Employee.name)),
                    TextColumn::new(lens!(Employee.email)),
                    TextColumn::new(lens!(Employee.phone)),
                    DateColumn::new(lens!(Employee.date_of_birth)),
                    ColorColumn::new(lens!(Employee.team_color)),
                ))),
                Tab::new(tablo::text!("tab-employment")).columns(
                    Grid::new(2).columns((
                        RelationColumn::of::<DepartmentResource>(relation!(Employee.department))
                            .label(tablo::text!("label-department")),
                        TextColumn::new(lens!(Employee.job_title)),
                        TextColumn::new(lens!(Employee.employment_type)).badge(),
                        DateColumn::new(lens!(Employee.hire_date)),
                        TextColumn::new(lens!(Employee.salary)).money(Currency::USD),
                        BooleanColumn::new(lens!(Employee.is_active))
                            .label(tablo::text!("label-active")),
                    )),
                ),
            ])))
            .form(Schema::new(Tabs::new([
                Tab::new(tablo::text!("tab-personal")).schema(Grid::new(2).schema((
                    c.name,
                    c.email.email().unique(),
                    c.phone.tel(),
                    c.date_of_birth,
                    c.team_color,
                ))),
                Tab::new(tablo::text!("tab-employment")).schema(
                    Grid::new(2).schema((
                        c.department_id
                            .searchable()
                            .label(tablo::text!("label-department")),
                        job_title,
                        c.employment_type.radio(),
                        c.hire_date,
                        salary,
                        hourly_rate,
                        c.is_active.switch().label(tablo::text!("label-active")),
                    )),
                ),
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
            .action::<BulkChangeEmployeeDepartment>()
            .action::<BulkSetEmployeeActive>()
            .soft_delete(lens!(Employee.deleted_at))
    }

    crate::audit!();
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
        RelationColumn::of::<DepartmentResource>(relation!(Employee.department))
            .label(tablo::text!("label-department"))
            .toggleable(),
        TextColumn::new(lens!(Employee.job_title))
            .searchable()
            .sortable()
            .toggleable(),
        TextColumn::new(lens!(Employee.employment_type)).badge(),
        TextColumn::new(lens!(Employee.salary))
            .money(Currency::USD)
            .sortable()
            .toggleable(),
        ColorColumn::new(lens!(Employee.team_color))
            .toggleable()
            .hidden_by_default(),
        DateColumn::new(lens!(Employee.hire_date))
            .sortable()
            .toggleable(),
        BooleanColumn::new(lens!(Employee.is_active))
            .label(tablo::text!("label-active"))
            .sortable(),
    ))
    .default_sort(lens!(Employee.name))
    .action_layout(ActionLayout::Menu)
    .option_tabs(|employment_type: EmploymentType| {
        Employee::fields().employment_type().eq(employment_type)
    })
    .filters((
        SelectFilter::relationship::<DepartmentResource>(lens!(Employee.department_id))
            .label(tablo::text!("label-department")),
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
    pub salary: Option<f64>,
    pub hourly_rate: Option<f64>,
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
    ) -> Result<Outcome, WriteError> {
        for employee in employees {
            Employee::filter(Employee::fields().id().eq(employee.id))
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

/// Moves one employee to another department.
pub struct ChangeEmployeeDepartment;

impl Action<EmployeeResource> for ChangeEmployeeDepartment {
    type Input = DepartmentChoice;
    const NAME: &'static str = "change-department";
    const PLACES: Places = Places::ROW;

    async fn run(
        _cx: &Cx,
        employees: &[Employee],
        choice: DepartmentChoice,
        ex: &mut dyn toasty::Executor,
    ) -> Result<Outcome, WriteError> {
        move_department(employees, choice.department_id, ex).await
    }
}

/// Moves the selection to another department.
pub struct BulkChangeEmployeeDepartment;

impl Action<EmployeeResource> for BulkChangeEmployeeDepartment {
    type Input = DepartmentChoice;
    const NAME: &'static str = "bulk-change-department";
    const PLACES: Places = Places::BULK;

    fn label() -> Text {
        tablo::text!("action-bulk-change-department")
    }

    async fn run(
        _cx: &Cx,
        employees: &[Employee],
        choice: DepartmentChoice,
        ex: &mut dyn toasty::Executor,
    ) -> Result<Outcome, WriteError> {
        move_department(employees, choice.department_id, ex).await
    }

    fn success(count: usize) -> Text {
        tablo::text!("flash-employees-moved", count = count)
    }
}

/// Moves every employee of `employees` to `department_id`.
async fn move_department(
    employees: &[Employee],
    department_id: Option<u64>,
    ex: &mut dyn toasty::Executor,
) -> Result<Outcome, WriteError> {
    for employee in employees {
        Employee::filter(Employee::fields().id().eq(employee.id))
            .update()
            .department_id(department_id)
            .exec(&mut *ex)
            .await?;
    }
    Ok(Outcome::done())
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
    ) -> Result<Outcome, WriteError> {
        for employee in employees {
            Employee::filter(Employee::fields().id().eq(employee.id))
                .update()
                .is_active(choice.is_active)
                .exec(&mut *ex)
                .await?;
        }
        Ok(Outcome::done())
    }

    fn success(count: usize) -> Text {
        tablo::text!("flash-employees-updated", count = count)
    }
}

/// Leave requests.
pub struct LeaveRequestResource;

impl Resource for LeaveRequestResource {
    type Model = LeaveRequest;
    type Form = LeaveRequestForm;

    fn declare() -> ResourceDef<Self> {
        let c = LeaveRequestForm::controls();
        // `gte` borrows the start date it compares with, so the start date is built first and
        // the schema takes it after the comparison.
        let start_date = c.start_date;
        let end_date = c.end_date.gte(&start_date);
        ResourceDef::new()
            .icon(crate::icons::LEAVE_REQUESTS)
            .navigation_group(tablo::text!("navigation-hr"))
            .navigation_order(2)
            .navigation_badge(LeaveRequest::fields().status().eq(LeaveStatus::Pending))
            .policy(roles::policy(Domain::Hr))
            .table(
                Table::new((
                    RelationColumn::of::<EmployeeResource>(relation!(LeaveRequest.employee))
                        .label(tablo::text!("label-employee")),
                    TextColumn::new(lens!(LeaveRequest.kind))
                        .badge()
                        .label(tablo::text!("label-type")),
                    TextColumn::new(lens!(LeaveRequest.status)).badge(),
                    DateColumn::new(lens!(LeaveRequest.start_date)).sortable(),
                    DateColumn::new(lens!(LeaveRequest.end_date)).sortable(),
                    TextColumn::new(lens!(LeaveRequest.days_requested))
                        .numeric(1)
                        .sortable(),
                    RelationColumn::of::<EmployeeResource>(relation!(LeaveRequest.approver))
                        .label(tablo::text!("label-approver"))
                        .toggleable()
                        .hidden_by_default(),
                ))
                .default_sort_desc(lens!(LeaveRequest.start_date))
                .action_layout(ActionLayout::Menu)
                .row_color(|request: &LeaveRequest| {
                    (request.status == LeaveStatus::Rejected).then_some(Color::Danger)
                })
                .row_color_reading(Reads::of(lens!(LeaveRequest.status)))
                .option_tabs(|status: LeaveStatus| LeaveRequest::fields().status().eq(status))
                .filters((
                    SelectFilter::of(lens!(LeaveRequest.kind)).label(tablo::text!("label-type")),
                )),
            )
            .detail(Detail::new((
                Section::new(tablo::text!("section-leave-details")).columns(
                    Grid::new(3).columns((
                        RelationColumn::of::<EmployeeResource>(relation!(LeaveRequest.employee))
                            .label(tablo::text!("label-employee")),
                        TextColumn::new(lens!(LeaveRequest.kind))
                            .badge()
                            .label(tablo::text!("label-type")),
                        TextColumn::new(lens!(LeaveRequest.status)).badge(),
                        DateColumn::new(lens!(LeaveRequest.start_date)),
                        DateColumn::new(lens!(LeaveRequest.end_date)),
                        TextColumn::new(lens!(LeaveRequest.days_requested)).format(|days| {
                            tablo::text!("value-days", value = format!("{days:.1}"))
                        }),
                        TextColumn::new(lens!(LeaveRequest.start_time)),
                        TextColumn::new(lens!(LeaveRequest.end_time)),
                        TextColumn::new(lens!(LeaveRequest.reason)),
                    )),
                ),
                Section::new(tablo::text!("section-review")).columns((
                    RelationColumn::of::<EmployeeResource>(relation!(LeaveRequest.approver))
                        .label(tablo::text!("label-approver")),
                    DateColumn::new(lens!(LeaveRequest.reviewed_at))
                        .label(tablo::text!("label-reviewed-at")),
                    TextColumn::new(lens!(LeaveRequest.reviewer_notes)),
                )),
            )))
            .form(Schema::new((
                Section::new(tablo::text!("section-leave-details")).schema(
                    Grid::new(2).schema((
                        c.employee_id
                            .searchable()
                            .label(tablo::text!("label-employee")),
                        c.kind.label(tablo::text!("label-type")).radio(),
                        start_date,
                        end_date,
                        c.start_time
                            .label(tablo::text!("label-start-time-half-days")),
                        c.end_time.label(tablo::text!("label-end-time-half-days")),
                        c.days_requested,
                        c.reason.multiline(4),
                    )),
                ),
                Section::new(tablo::text!("section-review")).schema(
                    Grid::new(2).schema((
                        c.status.buttons(),
                        c.approver_id
                            .searchable()
                            .label(tablo::text!("label-approver")),
                        c.reviewer_notes.multiline(3),
                    )),
                ),
            )))
            .record_title(lens!(LeaveRequest.reason))
            .action::<ApproveLeaveRequest>()
            .action::<RejectLeaveRequest>()
            .action::<BulkApproveLeaveRequests>()
            .action::<BulkRejectLeaveRequests>()
            // The requests by status, above the list.
            .list_widget::<LeaveStatusWidget>()
    }

    /// Refuses dates that overlap another request of the employee, then stores the day count
    /// the submitted dates cover.
    async fn create_record(
        _cx: &Cx,
        form: LeaveRequestForm,
        ex: &mut dyn toasty::Executor,
    ) -> Result<LeaveRequest, WriteError<LeaveRequestFormField>> {
        refuse_overlap(
            &mut *ex,
            form.employee_id,
            form.start_date,
            form.end_date,
            None,
        )
        .await?;
        let days = days_requested(form.start_date, form.end_date);
        Ok(form.into_create().days_requested(days).exec(ex).await?)
    }

    /// Refuses dates that overlap another request of the employee, then recomputes the day
    /// count from the stored dates. A refusal rolls the edit back.
    async fn update_record(
        cx: &Cx,
        record: LeaveRequest,
        posted: tablo::Posted<LeaveRequestForm>,
        ex: &mut dyn toasty::Executor,
    ) -> Result<LeaveRequest, WriteError<LeaveRequestFormField>> {
        let mut record = tablo::write_update::<Self>(cx, record, posted, ex).await?;
        refuse_overlap(
            &mut *ex,
            record.employee_id,
            record.start_date,
            record.end_date,
            Some(record.id),
        )
        .await?;
        let days = days_requested(record.start_date, record.end_date);
        if record.days_requested != days {
            record.update().days_requested(days).exec(&mut *ex).await?;
            record.days_requested = days;
        }
        Ok(record)
    }

    crate::audit!();
}

/// Refuses the dates `start` to `end` of `employee_id` when another request of that employee
/// covers any of them and is neither rejected nor cancelled. `except` is the request being
/// edited, which would otherwise overlap its own dates.
async fn refuse_overlap(
    ex: &mut dyn toasty::Executor,
    employee_id: u64,
    start: jiff::civil::Date,
    end: jiff::civil::Date,
    except: Option<u64>,
) -> Result<(), WriteError<LeaveRequestFormField>> {
    let fields = LeaveRequest::fields();
    let overlapping = LeaveRequest::filter(
        fields
            .employee_id()
            .eq(employee_id)
            .and(fields.start_date().le(end))
            .and(fields.end_date().ge(start))
            .and(fields.status().ne(LeaveStatus::Rejected))
            .and(fields.status().ne(LeaveStatus::Cancelled)),
    )
    .exec(&mut *ex)
    .await?;
    if overlapping.iter().any(|leave| Some(leave.id) != except) {
        return Err(WriteError::field(
            LeaveRequestFormField::StartDate,
            tablo::text!("error-leave-overlap"),
        ));
    }
    Ok(())
}

/// The days a leave request covers, counting both ends.
fn days_requested(start: jiff::civil::Date, end: jiff::civil::Date) -> f64 {
    let days = end.since(start).map(|span| span.get_days()).unwrap_or(0) + 1;
    f64::from(days.max(1))
}

/// The leave request's record form.
#[derive(tablo::RecordForm)]
#[form(model = LeaveRequest)]
pub struct LeaveRequestForm {
    #[form(relationship = EmployeeResource)]
    pub employee_id: u64,
    #[form(options)]
    pub kind: LeaveType,
    #[form(options)]
    pub status: LeaveStatus,
    pub start_date: jiff::civil::Date,
    pub end_date: jiff::civil::Date,
    pub start_time: Option<jiff::civil::Time>,
    pub end_time: Option<jiff::civil::Time>,
    pub days_requested: f64,
    pub reason: String,
    pub reviewer_notes: Option<String>,
    #[form(relationship = EmployeeResource)]
    pub approver_id: Option<u64>,
}

/// What a rejection asks for.
#[derive(FormInput)]
pub struct Rejection {
    #[form(multiline = 3)]
    pub rejection_reason: String,
}

/// Approves a pending leave request, confirming first.
pub struct ApproveLeaveRequest;

impl Action<LeaveRequestResource> for ApproveLeaveRequest {
    type Input = ();
    const NAME: &'static str = "approve";
    const PLACES: Places = Places::ROW.with(Places::DETAIL);
    const CONFIRM: bool = true;

    fn can_run(_cx: &Cx, request: &LeaveRequest) -> bool {
        request.status == LeaveStatus::Pending
    }

    async fn run(
        _cx: &Cx,
        requests: &[LeaveRequest],
        (): (),
        ex: &mut dyn toasty::Executor,
    ) -> Result<Outcome, WriteError> {
        review(requests, LeaveStatus::Approved, None, ex).await
    }

    fn success(_count: usize) -> Text {
        tablo::text!("flash-leave-approved")
    }
}

/// Rejects a pending leave request, with the reason.
pub struct RejectLeaveRequest;

impl Action<LeaveRequestResource> for RejectLeaveRequest {
    type Input = Rejection;
    const NAME: &'static str = "reject";
    const PLACES: Places = Places::ROW.with(Places::DETAIL);

    fn can_run(_cx: &Cx, request: &LeaveRequest) -> bool {
        request.status == LeaveStatus::Pending
    }

    async fn run(
        _cx: &Cx,
        requests: &[LeaveRequest],
        rejection: Rejection,
        ex: &mut dyn toasty::Executor,
    ) -> Result<Outcome, WriteError> {
        review(
            requests,
            LeaveStatus::Rejected,
            Some(rejection.rejection_reason),
            ex,
        )
        .await
    }

    fn success(_count: usize) -> Text {
        tablo::text!("flash-leave-rejected")
    }
}

/// Approves every pending request of the selection.
pub struct BulkApproveLeaveRequests;

impl Action<LeaveRequestResource> for BulkApproveLeaveRequests {
    type Input = ();
    const NAME: &'static str = "bulk-approve";
    const PLACES: Places = Places::BULK;

    fn label() -> Text {
        tablo::text!("action-bulk-approve")
    }
    const CONFIRM: bool = true;

    fn can_run(_cx: &Cx, request: &LeaveRequest) -> bool {
        request.status == LeaveStatus::Pending
    }

    async fn run(
        _cx: &Cx,
        requests: &[LeaveRequest],
        (): (),
        ex: &mut dyn toasty::Executor,
    ) -> Result<Outcome, WriteError> {
        review(requests, LeaveStatus::Approved, None, ex).await
    }

    fn success(count: usize) -> Text {
        tablo::text!("flash-leaves-approved", count = count)
    }
}

/// Rejects every pending request of the selection.
pub struct BulkRejectLeaveRequests;

impl Action<LeaveRequestResource> for BulkRejectLeaveRequests {
    type Input = Rejection;
    const NAME: &'static str = "bulk-reject";
    const PLACES: Places = Places::BULK;

    fn label() -> Text {
        tablo::text!("action-bulk-reject")
    }

    fn can_run(_cx: &Cx, request: &LeaveRequest) -> bool {
        request.status == LeaveStatus::Pending
    }

    async fn run(
        _cx: &Cx,
        requests: &[LeaveRequest],
        rejection: Rejection,
        ex: &mut dyn toasty::Executor,
    ) -> Result<Outcome, WriteError> {
        review(
            requests,
            LeaveStatus::Rejected,
            Some(rejection.rejection_reason),
            ex,
        )
        .await
    }

    fn success(count: usize) -> Text {
        tablo::text!("flash-leaves-rejected", count = count)
    }
}

/// Records `status` and the review time on every pending request, with the reviewer's notes.
async fn review(
    requests: &[LeaveRequest],
    status: LeaveStatus,
    notes: Option<String>,
    ex: &mut dyn toasty::Executor,
) -> Result<Outcome, WriteError> {
    for request in requests {
        let mut update = LeaveRequest::filter(LeaveRequest::fields().id().eq(request.id)).update();
        update = update.status(status).reviewed_at(Some(Timestamp::now()));
        // Approving keeps whatever notes the request already carries.
        if let Some(notes) = notes.clone() {
            update = update.reviewer_notes(Some(notes));
        }
        update.exec(&mut *ex).await?;
    }
    Ok(Outcome::done())
}

/// Projects.
pub struct ProjectResource;

impl Resource for ProjectResource {
    type Model = Project;
    type Form = ProjectForm;

    fn declare() -> ResourceDef<Self> {
        let c = ProjectForm::controls();
        // `gte` borrows the start date it compares with, so the start date is built first and
        // the schema takes it after the comparison.
        let start_date = c.start_date;
        let end_date = c.end_date.gte(&start_date);
        ResourceDef::new()
            .icon(crate::icons::PROJECTS)
            .navigation_group(tablo::text!("navigation-projects"))
            .navigation_order(0)
            .navigation_badge(Project::fields().status().eq(ProjectStatus::Active))
            .policy(roles::policy(Domain::Projects))
            .table(
                Table::new((
                    TextColumn::new(lens!(Project.name)).searchable().sortable(),
                    TextColumn::new(lens!(Project.status)).badge(),
                    TextColumn::new(lens!(Project.priority)).badge(),
                    RelationColumn::of::<DepartmentResource>(relation!(Project.department))
                        .label(tablo::text!("label-department"))
                        .toggleable(),
                    TextColumn::new(lens!(Project.budget))
                        .money(Currency::USD)
                        .sortable(),
                    TextColumn::new(lens!(Project.spent))
                        .money(Currency::USD)
                        .sortable(),
                    ComputedColumn::new(tablo::text!("label-progress"), |project: &Project| {
                        progress(project)
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
                    SelectFilter::relationship::<DepartmentResource>(lens!(Project.department_id))
                        .label(tablo::text!("label-department")),
                )),
            )
            .detail(Detail::new(Tabs::new([
                Tab::new(tablo::text!("tab-overview")).columns(
                    Grid::new(2).columns((
                        TextColumn::new(lens!(Project.name)),
                        TextColumn::new(lens!(Project.slug)),
                        RelationColumn::of::<DepartmentResource>(relation!(Project.department))
                            .label(tablo::text!("label-department")),
                        TextColumn::new(lens!(Project.status)).badge(),
                        TextColumn::new(lens!(Project.priority)).badge(),
                        ColorColumn::new(lens!(Project.color)),
                        DateColumn::new(lens!(Project.start_date)),
                        DateColumn::new(lens!(Project.end_date)),
                        TextColumn::new(lens!(Project.description)),
                    )),
                ),
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
                Tab::new(tablo::text!("tab-overview")).schema(
                    Grid::new(2).schema((
                        c.name,
                        c.slug.disabled_on_edit(),
                        c.description.multiline(4),
                        c.department_id
                            .searchable()
                            .label(tablo::text!("label-department")),
                        c.status.buttons(),
                        c.priority.radio(),
                        c.color,
                        start_date,
                        end_date,
                    )),
                ),
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

    crate::audit!();
}

/// How far along a project is, as the list reads it.
fn progress(project: &Project) -> String {
    if project.estimated_hours > 0.0 {
        format!(
            "{:.0}%",
            (project.actual_hours / project.estimated_hours) * 100.0
        )
    } else {
        "0%".to_string()
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
    pub budget: f64,
    pub spent: f64,
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
    ) -> Result<Outcome, WriteError> {
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
    ) -> Result<Outcome, WriteError> {
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
    ) -> Result<Outcome, WriteError> {
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
    ) -> Result<Outcome, WriteError> {
        for project in projects {
            Project::filter(Project::fields().id().eq(project.id))
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
async fn set_project_status(
    projects: &[Project],
    status: ProjectStatus,
    ex: &mut dyn toasty::Executor,
) -> Result<Outcome, WriteError> {
    for project in projects {
        Project::filter(Project::fields().id().eq(project.id))
            .update()
            .status(status)
            .exec(&mut *ex)
            .await?;
    }
    Ok(Outcome::done())
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
            .icon(crate::icons::TASKS)
            .navigation_group(tablo::text!("navigation-projects"))
            .navigation_order(1)
            .navigation_badge(Task::fields().status().eq(TaskStatus::InProgress))
            .policy(roles::policy(Domain::Projects))
            .table(
                Table::new((
                    TextColumn::new(lens!(Task.title)).searchable().sortable(),
                    RelationColumn::of::<ProjectResource>(relation!(Task.project))
                        .label(tablo::text!("label-project")),
                    RelationColumn::of::<EmployeeResource>(relation!(Task.assignee))
                        .label(tablo::text!("label-assignee"))
                        .toggleable(),
                    TextColumn::new(lens!(Task.status)).badge(),
                    TextColumn::new(lens!(Task.priority)).badge(),
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
                    SelectFilter::relationship::<ProjectResource>(lens!(Task.project_id))
                        .label(tablo::text!("label-project")),
                    SelectFilter::relationship::<EmployeeResource>(lens!(Task.assigned_to))
                        .label(tablo::text!("label-assignee")),
                ))
                .group_by(lens!(Task.status)),
            )
            .detail(Detail::new(
                Section::new(tablo::text!("section-task")).columns((
                    Grid::new(2).columns((
                        TextColumn::new(lens!(Task.title)),
                        RelationColumn::of::<ProjectResource>(relation!(Task.project))
                            .label(tablo::text!("label-project")),
                        RelationColumn::of::<EmployeeResource>(relation!(Task.assignee))
                            .label(tablo::text!("label-assignee")),
                        TextColumn::new(lens!(Task.status)).badge(),
                        TextColumn::new(lens!(Task.priority)).badge(),
                        TextColumn::new(lens!(Task.estimated_hours)),
                        TextColumn::new(lens!(Task.actual_hours)),
                        DateColumn::new(lens!(Task.due_date)),
                        DateColumn::new(lens!(Task.completed_at)),
                    )),
                    TextColumn::new(lens!(Task.description)),
                )),
            ))
            .form(Schema::new(
                Section::new(tablo::text!("section-task-details")).schema(
                    Grid::new(2).schema((
                        c.title,
                        c.project_id
                            .searchable()
                            .label(tablo::text!("label-project")),
                        c.assigned_to
                            .searchable()
                            .label(tablo::text!("label-assignee")),
                        c.status.buttons(),
                        c.priority.radio(),
                        c.estimated_hours.suffix(tablo::text!("unit-hours")),
                        c.due_date,
                        c.description.multiline(4),
                        completed_at,
                    )),
                ),
            ))
            .record_title(lens!(Task.title))
            .action::<StartTask>()
            .action::<SendTaskToReview>()
            .action::<CompleteTask>()
            .action::<AssignTask>()
            .action::<SetTaskPriority>()
            .action::<BulkSetTaskStatus>()
            .action::<BulkAssignTask>()
            .action::<ReplicateTask>()
    }

    crate::audit!();
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
    ) -> Result<Outcome, WriteError> {
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
    ) -> Result<Outcome, WriteError> {
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
    ) -> Result<Outcome, WriteError> {
        for task in tasks {
            Task::filter(Task::fields().id().eq(task.id))
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

/// Assigns a task to an employee.
pub struct AssignTask;

impl Action<TaskResource> for AssignTask {
    type Input = AssigneeChoice;
    const NAME: &'static str = "assign";
    const PLACES: Places = Places::ROW;

    async fn run(
        _cx: &Cx,
        tasks: &[Task],
        choice: AssigneeChoice,
        ex: &mut dyn toasty::Executor,
    ) -> Result<Outcome, WriteError> {
        assign(tasks, choice.assigned_to, ex).await
    }
}

/// Assigns every task in the selection.
pub struct BulkAssignTask;

impl Action<TaskResource> for BulkAssignTask {
    type Input = AssigneeChoice;
    const NAME: &'static str = "bulk-assign";
    const PLACES: Places = Places::BULK;

    fn label() -> Text {
        tablo::text!("action-bulk-assign")
    }

    async fn run(
        _cx: &Cx,
        tasks: &[Task],
        choice: AssigneeChoice,
        ex: &mut dyn toasty::Executor,
    ) -> Result<Outcome, WriteError> {
        assign(tasks, choice.assigned_to, ex).await
    }

    fn success(count: usize) -> Text {
        tablo::text!("flash-tasks-assigned", count = count)
    }
}

/// Assigns every task of `tasks` to `assigned_to`.
async fn assign(
    tasks: &[Task],
    assigned_to: Option<u64>,
    ex: &mut dyn toasty::Executor,
) -> Result<Outcome, WriteError> {
    for task in tasks {
        Task::filter(Task::fields().id().eq(task.id))
            .update()
            .assigned_to(assigned_to)
            .exec(&mut *ex)
            .await?;
    }
    Ok(Outcome::done())
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
    ) -> Result<Outcome, WriteError> {
        for task in tasks {
            Task::filter(Task::fields().id().eq(task.id))
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
    ) -> Result<Outcome, WriteError> {
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
    ) -> Result<Outcome, WriteError> {
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
async fn set_task_status(
    tasks: &[Task],
    status: TaskStatus,
    ex: &mut dyn toasty::Executor,
) -> Result<Outcome, WriteError> {
    for task in tasks {
        Task::filter(Task::fields().id().eq(task.id))
            .update()
            .status(status)
            .exec(&mut *ex)
            .await?;
    }
    Ok(Outcome::done())
}

/// Timesheets.
pub struct TimesheetResource;

impl Resource for TimesheetResource {
    type Model = Timesheet;
    type Form = TimesheetForm;

    fn declare() -> ResourceDef<Self> {
        let c = TimesheetForm::controls();
        let task_id = c
            .task_id
            .searchable()
            .label(tablo::text!("label-task"))
            .depends_on(&c.project_id, Task::fields().project_id());
        ResourceDef::new()
            .icon(crate::icons::TIMESHEETS)
            .navigation_group(tablo::text!("navigation-projects"))
            .navigation_order(2)
            .policy(roles::policy(Domain::Projects))
            .table(
                Table::new((
                    RelationColumn::of::<EmployeeResource>(relation!(Timesheet.employee))
                        .label(tablo::text!("label-employee")),
                    RelationColumn::of::<ProjectResource>(relation!(Timesheet.project))
                        .label(tablo::text!("label-project")),
                    RelationColumn::of::<TaskResource>(relation!(Timesheet.task))
                        .label(tablo::text!("label-task"))
                        .toggleable(),
                    TextColumn::new(lens!(Timesheet.date)).sortable(),
                    TextColumn::new(lens!(Timesheet.hours))
                        .numeric(1)
                        .sortable(),
                    BooleanColumn::new(lens!(Timesheet.is_billable))
                        .label(tablo::text!("label-billable"))
                        .toggleable(),
                    TextColumn::new(lens!(Timesheet.hourly_rate))
                        .money(Currency::USD)
                        .sortable()
                        .toggleable(),
                    TextColumn::new(lens!(Timesheet.total_cost))
                        .money(Currency::USD)
                        .sortable(),
                ))
                .default_sort_desc(lens!(Timesheet.date))
                .action_layout(ActionLayout::Menu)
                .filters((
                    SelectFilter::relationship::<EmployeeResource>(lens!(Timesheet.employee_id))
                        .label(tablo::text!("label-employee")),
                    SelectFilter::relationship::<ProjectResource>(lens!(Timesheet.project_id))
                        .label(tablo::text!("label-project")),
                    TernaryFilter::new(lens!(Timesheet.is_billable)),
                    DateFilter::new(lens!(Timesheet.date))
                        .label(tablo::text!("label-date"))
                        .range(),
                ))
                .group_by(lens!(Timesheet.date)),
            )
            .detail(Detail::new(
                Section::new(tablo::text!("section-timesheet")).columns((
                    RelationColumn::of::<EmployeeResource>(relation!(Timesheet.employee))
                        .label(tablo::text!("label-employee")),
                    RelationColumn::of::<ProjectResource>(relation!(Timesheet.project))
                        .label(tablo::text!("label-project")),
                    RelationColumn::of::<TaskResource>(relation!(Timesheet.task))
                        .label(tablo::text!("label-task")),
                    TextColumn::new(lens!(Timesheet.date)),
                    TextColumn::new(lens!(Timesheet.hours)),
                    TextColumn::new(lens!(Timesheet.minutes)),
                    BooleanColumn::new(lens!(Timesheet.is_billable))
                        .label(tablo::text!("label-billable")),
                    TextColumn::new(lens!(Timesheet.hourly_rate)).money(Currency::USD),
                    TextColumn::new(lens!(Timesheet.total_cost)).money(Currency::USD),
                    TextColumn::new(lens!(Timesheet.description)),
                )),
            ))
            .form(Schema::new(
                Section::new(tablo::text!("section-timesheet-entry")).schema(
                    Grid::new(2).schema((
                        c.employee_id
                            .searchable()
                            .label(tablo::text!("label-employee")),
                        c.project_id
                            .searchable()
                            .label(tablo::text!("label-project")),
                        task_id,
                        c.date,
                        c.hours.suffix(tablo::text!("unit-hours")),
                        c.minutes.suffix(tablo::text!("unit-minutes")),
                        c.is_billable.switch().label(tablo::text!("label-billable")),
                        c.hourly_rate.prefix("$"),
                        c.total_cost.prefix("$"),
                        c.description.multiline(2),
                    )),
                ),
            ))
            .action::<ToggleTimesheetBillable>()
            .action::<BulkMarkBillable>()
            .action::<BulkMarkNonBillable>()
    }

    /// Stores the cost the hours and the rate come to.
    async fn create_record(
        _cx: &Cx,
        form: TimesheetForm,
        ex: &mut dyn toasty::Executor,
    ) -> Result<Timesheet, WriteError<TimesheetFormField>> {
        let cost = form.hours * form.hourly_rate;
        Ok(form.into_create().total_cost(cost).exec(ex).await?)
    }

    /// Recomputes the cost from the stored hours and rate.
    async fn update_record(
        cx: &Cx,
        record: Timesheet,
        posted: tablo::Posted<TimesheetForm>,
        ex: &mut dyn toasty::Executor,
    ) -> Result<Timesheet, WriteError<TimesheetFormField>> {
        let mut record = tablo::write_update::<Self>(cx, record, posted, ex).await?;
        let cost = record.hours * record.hourly_rate;
        if record.total_cost != cost {
            record.update().total_cost(cost).exec(&mut *ex).await?;
            record.total_cost = cost;
        }
        Ok(record)
    }

    crate::audit!();
}

/// The timesheet's record form.
#[derive(tablo::RecordForm)]
#[form(model = Timesheet)]
pub struct TimesheetForm {
    #[form(relationship = EmployeeResource)]
    pub employee_id: u64,
    #[form(relationship = ProjectResource)]
    pub project_id: u64,
    #[form(relationship = TaskResource)]
    pub task_id: Option<u64>,
    pub date: jiff::civil::Date,
    pub hours: f64,
    pub minutes: i64,
    pub description: Option<String>,
    pub is_billable: bool,
    pub hourly_rate: f64,
    pub total_cost: f64,
}

/// Marks a recent timesheet billable or not.
pub struct ToggleTimesheetBillable;

impl Action<TimesheetResource> for ToggleTimesheetBillable {
    type Input = ();
    const NAME: &'static str = "toggle-billable";
    const PLACES: Places = Places::ROW;

    /// Only the last week's entries may change, which the demo's button disables.
    fn can_run(_cx: &Cx, timesheet: &Timesheet) -> bool {
        let week_ago = Zoned::now()
            .date()
            .checked_sub(Span::new().days(7))
            .unwrap_or(timesheet.date);
        timesheet.date >= week_ago
    }

    async fn run(
        _cx: &Cx,
        timesheets: &[Timesheet],
        (): (),
        ex: &mut dyn toasty::Executor,
    ) -> Result<Outcome, WriteError> {
        for timesheet in timesheets {
            Timesheet::filter(Timesheet::fields().id().eq(timesheet.id))
                .update()
                .is_billable(!timesheet.is_billable)
                .exec(&mut *ex)
                .await?;
        }
        Ok(Outcome::done())
    }
}

/// Marks the selection billable, confirming first.
pub struct BulkMarkBillable;

impl Action<TimesheetResource> for BulkMarkBillable {
    type Input = ();
    const NAME: &'static str = "mark-billable";
    const PLACES: Places = Places::BULK;
    const CONFIRM: bool = true;

    async fn run(
        _cx: &Cx,
        timesheets: &[Timesheet],
        (): (),
        ex: &mut dyn toasty::Executor,
    ) -> Result<Outcome, WriteError> {
        set_billable(timesheets, true, ex).await
    }

    fn success(count: usize) -> Text {
        tablo::text!("flash-timesheets-billable", count = count)
    }
}

/// Marks the selection not billable, confirming first.
pub struct BulkMarkNonBillable;

impl Action<TimesheetResource> for BulkMarkNonBillable {
    type Input = ();
    const NAME: &'static str = "mark-non-billable";
    const PLACES: Places = Places::BULK;
    const CONFIRM: bool = true;

    async fn run(
        _cx: &Cx,
        timesheets: &[Timesheet],
        (): (),
        ex: &mut dyn toasty::Executor,
    ) -> Result<Outcome, WriteError> {
        set_billable(timesheets, false, ex).await
    }

    fn success(count: usize) -> Text {
        tablo::text!("flash-timesheets-not-billable", count = count)
    }
}

/// Sets `is_billable` on every timesheet of `timesheets`.
async fn set_billable(
    timesheets: &[Timesheet],
    billable: bool,
    ex: &mut dyn toasty::Executor,
) -> Result<Outcome, WriteError> {
    for timesheet in timesheets {
        Timesheet::filter(Timesheet::fields().id().eq(timesheet.id))
            .update()
            .is_billable(billable)
            .exec(&mut *ex)
            .await?;
    }
    Ok(Outcome::done())
}

/// Expenses.
pub struct ExpenseResource;

impl Resource for ExpenseResource {
    type Model = Expense;
    type Form = ExpenseForm;

    fn declare() -> ResourceDef<Self> {
        let c = ExpenseForm::controls();
        ResourceDef::new()
            .icon(crate::icons::EXPENSES)
            .navigation_group(tablo::text!("navigation-hr"))
            .navigation_order(3)
            .navigation_badge(Expense::fields().status().eq(ExpenseStatus::Submitted))
            .policy(roles::policy(Domain::Hr))
            .table(
                Table::new((
                    TextColumn::new(lens!(Expense.expense_number))
                        .searchable()
                        .sortable(),
                    RelationColumn::of::<EmployeeResource>(relation!(Expense.employee))
                        .label(tablo::text!("label-employee")),
                    TextColumn::new(lens!(Expense.category)).badge(),
                    TextColumn::new(lens!(Expense.status)).badge(),
                    TextColumn::new(lens!(Expense.total_amount))
                        .money(Currency::USD)
                        .sortable(),
                    RelationColumn::of::<ProjectResource>(relation!(Expense.project))
                        .label(tablo::text!("label-project"))
                        .toggleable(),
                    DateColumn::new(lens!(Expense.submitted_at))
                        .sortable()
                        .toggleable(),
                ))
                .action_layout(ActionLayout::Menu)
                .row_color(|expense: &Expense| {
                    (expense.status == ExpenseStatus::Rejected).then_some(Color::Danger)
                })
                .row_color_reading(Reads::of(lens!(Expense.status)))
                .option_tabs(|status: ExpenseStatus| Expense::fields().status().eq(status))
                .filters((
                    SelectFilter::of(lens!(Expense.category)),
                    SelectFilter::relationship::<EmployeeResource>(lens!(Expense.employee_id))
                        .label(tablo::text!("label-employee")),
                )),
            )
            .detail(Detail::new((
                Section::new(tablo::text!("section-expense-details")).columns(
                    Grid::new(3).columns((
                        TextColumn::new(lens!(Expense.expense_number)),
                        RelationColumn::of::<EmployeeResource>(relation!(Expense.employee))
                            .label(tablo::text!("label-employee")),
                        TextColumn::new(lens!(Expense.category)).badge(),
                        TextColumn::new(lens!(Expense.status)).badge(),
                        TextColumn::new(lens!(Expense.total_amount)).money(Currency::USD),
                        TextColumn::new(lens!(Expense.currency)),
                        RelationColumn::of::<ProjectResource>(relation!(Expense.project))
                            .label(tablo::text!("label-project")),
                        DateColumn::new(lens!(Expense.submitted_at)),
                        DateColumn::new(lens!(Expense.approved_at)),
                        TextColumn::new(lens!(Expense.description)),
                        TextColumn::new(lens!(Expense.notes)),
                    )),
                ),
                Section::new(tablo::text!("section-receipt")).columns(
                    FileColumn::new(lens!(Expense.receipt_path))
                        .label(tablo::text!("label-receipt")),
                ),
            )))
            .form(Schema::new((
                Section::new(tablo::text!("section-details")).schema(
                    Grid::new(2).schema((
                        c.expense_number.disabled_on_edit(),
                        c.employee_id
                            .searchable()
                            .label(tablo::text!("label-employee")),
                        c.project_id
                            .searchable()
                            .label(tablo::text!("label-project")),
                        c.category.radio(),
                        c.status.buttons(),
                        c.description.multiline(3),
                    )),
                ),
                Section::new(tablo::text!("section-summary")).schema(
                    Grid::new(2).schema((
                        c.total_amount,
                        c.currency.choice().options(["USD"]),
                        c.receipt_path
                            .extensions(RECEIPT_EXTENSIONS)
                            .max_bytes(RECEIPT_MAX_BYTES),
                        c.notes.multiline(3),
                    )),
                ),
            )))
            .record_title(lens!(Expense.expense_number))
            .relation(Relation::has_many::<ExpenseLineResource>(
                ExpenseLine::fields().expense_id(),
            ))
            .action::<SubmitExpense>()
            .action::<ApproveExpense>()
            .action::<RejectExpense>()
            .action::<ReimburseExpense>()
            .action::<FlagExpense>()
    }

    crate::audit!();
}

/// The expense's record form.
#[derive(tablo::RecordForm)]
#[form(model = Expense)]
pub struct ExpenseForm {
    pub expense_number: String,
    #[form(relationship = EmployeeResource)]
    pub employee_id: u64,
    #[form(relationship = ProjectResource)]
    pub project_id: Option<u64>,
    #[form(options)]
    pub category: ExpenseCategory,
    #[form(options)]
    pub status: ExpenseStatus,
    pub description: String,
    pub total_amount: f64,
    pub currency: String,
    #[form(file, optional)]
    pub receipt_path: String,
    pub notes: Option<String>,
}

/// Submits a draft expense, confirming first.
pub struct SubmitExpense;

impl Action<ExpenseResource> for SubmitExpense {
    type Input = ();
    const NAME: &'static str = "submit";
    const PLACES: Places = Places::ROW.with(Places::DETAIL);
    const CONFIRM: bool = true;

    fn can_run(_cx: &Cx, expense: &Expense) -> bool {
        expense.status == ExpenseStatus::Draft
    }

    async fn run(
        _cx: &Cx,
        expenses: &[Expense],
        (): (),
        ex: &mut dyn toasty::Executor,
    ) -> Result<Outcome, WriteError> {
        for expense in expenses {
            if expense.total_amount <= 0.0 {
                return Err(topcoat::Error::msg("Cannot submit an expense with no amount").into());
            }
            Expense::filter(Expense::fields().id().eq(expense.id))
                .update()
                .status(ExpenseStatus::Submitted)
                .submitted_at(Some(Timestamp::now()))
                .exec(&mut *ex)
                .await?;
        }
        Ok(Outcome::done())
    }

    fn success(count: usize) -> Text {
        tablo::text!("flash-expenses-submitted", count = count)
    }
}

/// Approves a submitted expense, confirming first.
pub struct ApproveExpense;

impl Action<ExpenseResource> for ApproveExpense {
    type Input = ();
    const NAME: &'static str = "approve";
    const PLACES: Places = Places::ROW.with(Places::DETAIL);
    const CONFIRM: bool = true;

    fn can_run(_cx: &Cx, expense: &Expense) -> bool {
        expense.status == ExpenseStatus::Submitted
    }

    async fn run(
        _cx: &Cx,
        expenses: &[Expense],
        (): (),
        ex: &mut dyn toasty::Executor,
    ) -> Result<Outcome, WriteError> {
        for expense in expenses {
            Expense::filter(Expense::fields().id().eq(expense.id))
                .update()
                .status(ExpenseStatus::Approved)
                .approved_at(Some(Timestamp::now()))
                .exec(&mut *ex)
                .await?;
        }
        Ok(Outcome::done())
    }

    fn success(_count: usize) -> Text {
        tablo::text!("flash-expense-approved")
    }
}

/// What a rejection asks for.
#[derive(FormInput)]
pub struct RejectionReason {
    #[form(multiline = 3)]
    pub reason: String,
}

/// Rejects a submitted expense, storing the reason.
pub struct RejectExpense;

impl Action<ExpenseResource> for RejectExpense {
    type Input = RejectionReason;
    const NAME: &'static str = "reject";
    const PLACES: Places = Places::ROW.with(Places::DETAIL);

    fn can_run(_cx: &Cx, expense: &Expense) -> bool {
        expense.status == ExpenseStatus::Submitted
    }

    async fn run(
        _cx: &Cx,
        expenses: &[Expense],
        rejection: RejectionReason,
        ex: &mut dyn toasty::Executor,
    ) -> Result<Outcome, WriteError> {
        for expense in expenses {
            Expense::filter(Expense::fields().id().eq(expense.id))
                .update()
                .status(ExpenseStatus::Rejected)
                .notes(Some(rejection.reason.clone()))
                .exec(&mut *ex)
                .await?;
        }
        Ok(Outcome::done())
    }

    fn success(_count: usize) -> Text {
        tablo::text!("flash-expense-rejected")
    }
}

/// Marks an approved expense reimbursed, confirming first.
pub struct ReimburseExpense;

impl Action<ExpenseResource> for ReimburseExpense {
    type Input = ();
    const NAME: &'static str = "reimburse";
    const PLACES: Places = Places::ROW.with(Places::DETAIL);
    const CONFIRM: bool = true;

    fn can_run(_cx: &Cx, expense: &Expense) -> bool {
        expense.status == ExpenseStatus::Approved
    }

    async fn run(
        _cx: &Cx,
        expenses: &[Expense],
        (): (),
        ex: &mut dyn toasty::Executor,
    ) -> Result<Outcome, WriteError> {
        for expense in expenses {
            Expense::filter(Expense::fields().id().eq(expense.id))
                .update()
                .status(ExpenseStatus::Reimbursed)
                .exec(&mut *ex)
                .await?;
        }
        Ok(Outcome::done())
    }

    fn success(_count: usize) -> Text {
        tablo::text!("flash-expense-reimbursed")
    }
}

/// Returns an expense to draft, with the reason it was flagged.
pub struct FlagExpense;

impl Action<ExpenseResource> for FlagExpense {
    type Input = RejectionReason;
    const NAME: &'static str = "flag";
    const PLACES: Places = Places::ROW.with(Places::DETAIL);

    async fn run(
        _cx: &Cx,
        expenses: &[Expense],
        flag: RejectionReason,
        ex: &mut dyn toasty::Executor,
    ) -> Result<Outcome, WriteError> {
        for expense in expenses {
            Expense::filter(Expense::fields().id().eq(expense.id))
                .update()
                .status(ExpenseStatus::Draft)
                .notes(Some(flag.reason.clone()))
                .exec(&mut *ex)
                .await?;
        }
        Ok(Outcome::done())
    }

    fn success(_count: usize) -> Text {
        tablo::text!("flash-expense-flagged")
    }
}

/// One line of an expense claim.
pub struct ExpenseLineResource;

impl Resource for ExpenseLineResource {
    type Model = ExpenseLine;
    type Form = ExpenseLineForm;

    fn declare() -> ResourceDef<Self> {
        let c = ExpenseLineForm::controls();
        ResourceDef::new()
            .icon(crate::icons::EXPENSE_LINES)
            .navigation_group(tablo::text!("navigation-hr"))
            .navigation_order(4)
            .policy(roles::policy(Domain::Hr))
            .without_global_search()
            .table(Table::new((
                RelationColumn::of::<ExpenseResource>(relation!(ExpenseLine.expense))
                    .label(tablo::text!("label-expense")),
                TextColumn::new(lens!(ExpenseLine.description)).searchable(),
                TextColumn::new(lens!(ExpenseLine.quantity)).numeric(0),
                TextColumn::new(lens!(ExpenseLine.unit_price)).money(Currency::USD),
                TextColumn::new(lens!(ExpenseLine.amount)).money(Currency::USD),
                TextColumn::new(lens!(ExpenseLine.date)),
            )))
            .detail(Detail::new(
                Section::new(tablo::text!("section-expense-line")).columns((
                    RelationColumn::of::<ExpenseResource>(relation!(ExpenseLine.expense))
                        .label(tablo::text!("label-expense")),
                    TextColumn::new(lens!(ExpenseLine.description)),
                    TextColumn::new(lens!(ExpenseLine.quantity)).numeric(0),
                    TextColumn::new(lens!(ExpenseLine.unit_price)).money(Currency::USD),
                    TextColumn::new(lens!(ExpenseLine.amount)).money(Currency::USD),
                    TextColumn::new(lens!(ExpenseLine.date)),
                )),
            ))
            .form(Schema::new(
                Section::new(tablo::text!("section-expense-line")).schema(
                    Grid::new(2).schema((
                        c.expense_id
                            .searchable()
                            .label(tablo::text!("label-expense")),
                        c.description,
                        c.quantity,
                        c.unit_price,
                        c.amount,
                        c.date,
                    )),
                ),
            ))
    }

    /// Stores the amount the quantity and the unit price come to.
    async fn create_record(
        _cx: &Cx,
        form: ExpenseLineForm,
        ex: &mut dyn toasty::Executor,
    ) -> Result<ExpenseLine, WriteError<ExpenseLineFormField>> {
        let amount = round(form.quantity as f64 * form.unit_price);
        let line = form.into_create().amount(amount).exec(&mut *ex).await?;
        refresh_expense_total(line.expense_id, ex).await?;
        Ok(line)
    }

    /// Recomputes the amount from the stored quantity and price.
    async fn update_record(
        cx: &Cx,
        record: ExpenseLine,
        posted: tablo::Posted<ExpenseLineForm>,
        ex: &mut dyn toasty::Executor,
    ) -> Result<ExpenseLine, WriteError<ExpenseLineFormField>> {
        let previous = record.expense_id;
        let mut record = tablo::write_update::<Self>(cx, record, posted, ex).await?;
        let amount = round(record.quantity as f64 * record.unit_price);
        if record.amount != amount {
            record.update().amount(amount).exec(&mut *ex).await?;
            record.amount = amount;
        }
        refresh_expense_total(previous, ex).await?;
        if record.expense_id != previous {
            refresh_expense_total(record.expense_id, ex).await?;
        }
        Ok(record)
    }

    /// Removes the line and takes it off the claim it belonged to.
    async fn delete_record(
        _cx: &Cx,
        record: &ExpenseLine,
        ex: &mut dyn toasty::Executor,
    ) -> Result<()> {
        ExpenseLine::filter(ExpenseLine::fields().id().eq(record.id))
            .delete()
            .exec(&mut *ex)
            .await?;
        refresh_expense_total(record.expense_id, ex).await
    }

    crate::audit!();
}

/// Sets a claim's total to what its lines now come to.
async fn refresh_expense_total(expense_id: u64, ex: &mut dyn toasty::Executor) -> Result<()> {
    let lines = ExpenseLine::filter(ExpenseLine::fields().expense_id().eq(expense_id))
        .exec(&mut *ex)
        .await?;
    let total: f64 = lines.iter().map(|line| line.amount).sum();
    Expense::filter(Expense::fields().id().eq(expense_id))
        .update()
        .total_amount(round(total))
        .exec(&mut *ex)
        .await?;
    Ok(())
}

/// The expense line's record form.
#[derive(tablo::RecordForm)]
#[form(model = ExpenseLine)]
pub struct ExpenseLineForm {
    #[form(relationship = ExpenseResource)]
    pub expense_id: u64,
    pub description: String,
    pub quantity: i64,
    pub unit_price: f64,
    pub amount: f64,
    pub date: jiff::civil::Date,
}
