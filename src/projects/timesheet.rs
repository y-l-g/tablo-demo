//! Timesheets: the time booked to projects, its cost, and whether it is billable.

use rust_decimal::Decimal;
use tablo::prelude::*;
use topcoat::{Result, context::Cx, icon::iconify::iconify_icon};

use super::{ProjectResource, Task, TaskResource, Timesheet};
use crate::access::{self, Domain};
use crate::hr::EmployeeResource;

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
            .depends_on(&c.project_id, Task::fields().project_id());
        ResourceDef::new()
            .icon(iconify_icon!("lucide:clock"))
            .navigation_group(tablo::text!("navigation-projects"))
            .navigation_order(2)
            .policy(access::policy(Domain::Projects))
            .table(
                Table::new((
                    RelationColumn::of::<EmployeeResource>(relation!(Timesheet.employee)),
                    RelationColumn::of::<ProjectResource>(relation!(Timesheet.project)),
                    RelationColumn::of::<TaskResource>(relation!(Timesheet.task)).toggleable(),
                    TextColumn::new(lens!(Timesheet.date)).sortable(),
                    TextColumn::new(lens!(Timesheet.hours))
                        .numeric(1)
                        .sortable(),
                    BooleanColumn::new(lens!(Timesheet.is_billable)).toggleable(),
                    TextColumn::new(lens!(Timesheet.hourly_rate))
                        .money(Currency::USD)
                        .toggleable(),
                    TextColumn::new(lens!(Timesheet.total_cost)).money(Currency::USD),
                ))
                .default_sort_desc(lens!(Timesheet.date))
                .action_layout(ActionLayout::Menu)
                .filters((
                    SelectFilter::relationship::<EmployeeResource>(lens!(Timesheet.employee_id)),
                    SelectFilter::relationship::<ProjectResource>(lens!(Timesheet.project_id)),
                    TernaryFilter::new(lens!(Timesheet.is_billable)),
                    DateFilter::new(lens!(Timesheet.date)).range(),
                ))
                .group_by(lens!(Timesheet.date)),
            )
            .detail(Detail::new(
                Section::new(tablo::text!("section-timesheet")).columns((
                    RelationColumn::of::<EmployeeResource>(relation!(Timesheet.employee)),
                    RelationColumn::of::<ProjectResource>(relation!(Timesheet.project)),
                    RelationColumn::of::<TaskResource>(relation!(Timesheet.task)),
                    TextColumn::new(lens!(Timesheet.date)),
                    TextColumn::new(lens!(Timesheet.hours)),
                    TextColumn::new(lens!(Timesheet.minutes)),
                    BooleanColumn::new(lens!(Timesheet.is_billable)),
                    TextColumn::new(lens!(Timesheet.hourly_rate)).money(Currency::USD),
                    TextColumn::new(lens!(Timesheet.total_cost)).money(Currency::USD),
                    TextColumn::new(lens!(Timesheet.description)),
                )),
            ))
            .form(Schema::new(
                Section::new(tablo::text!("section-timesheet-entry")).schema(Grid::new(2).schema(
                    (
                        c.employee_id.searchable(),
                        c.project_id.searchable(),
                        task_id,
                        c.date,
                        c.hours.suffix(tablo::text!("unit-hours")),
                        c.minutes.suffix(tablo::text!("unit-minutes")),
                        c.is_billable.switch(),
                        c.hourly_rate.prefix("$"),
                        c.description.multiline(2),
                    ),
                )),
            ))
            .create_column(lens!(Timesheet.total_cost))
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
        let total_cost = Timesheet::cost(form.hours, form.hourly_rate);
        Ok(form.into_create().total_cost(total_cost).exec(ex).await?)
    }

    /// Recomputes the cost from the stored hours and rate.
    async fn update_record(
        cx: &Cx,
        record: Timesheet,
        posted: tablo::Posted<TimesheetForm>,
        ex: &mut dyn toasty::Executor,
    ) -> Result<Timesheet, WriteError<TimesheetFormField>> {
        let mut record = tablo::write_update::<Self>(cx, record, posted, ex).await?;
        let total_cost = Timesheet::cost(record.hours, record.hourly_rate);
        if record.total_cost != total_cost {
            record
                .update()
                .total_cost(total_cost)
                .exec(&mut *ex)
                .await?;
            record.total_cost = total_cost;
        }
        Ok(record)
    }
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
    pub hourly_rate: Decimal,
}

/// Marks a recent timesheet billable or not.
pub struct ToggleTimesheetBillable;

impl Action<TimesheetResource> for ToggleTimesheetBillable {
    type Input = ();
    const NAME: &'static str = "toggle-billable";
    const PLACES: Places = Places::ROW;

    /// Only the last week's entries may change; an older one shows the button disabled.
    fn can_run(_cx: &Cx, timesheet: &Timesheet) -> bool {
        !timesheet.is_locked()
    }

    fn disabled_reason(_cx: &Cx, timesheet: &Timesheet) -> Option<Text> {
        timesheet
            .is_locked()
            .then(|| tablo::text!("reason-timesheet-locked"))
    }

    async fn run(
        _cx: &Cx,
        timesheets: &[Timesheet],
        (): (),
        ex: &mut dyn toasty::Executor,
    ) -> Result<Outcome, WriteError<std::convert::Infallible>> {
        for timesheet in timesheets {
            Timesheet::filter_by_id(timesheet.id)
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
    ) -> Result<Outcome, WriteError<std::convert::Infallible>> {
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
    ) -> Result<Outcome, WriteError<std::convert::Infallible>> {
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
) -> Result<Outcome, WriteError<std::convert::Infallible>> {
    Timesheet::filter(filter_keys(timesheets))
        .update()
        .is_billable(billable)
        .exec(ex)
        .await?;
    Ok(Outcome::done())
}
