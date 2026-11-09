//! The HR dashboard: the workforce, the hours booked, each department's headcount, and the
//! leave and expenses waiting for a decision.

use jiff::Zoned;
use tablo::prelude::*;
use tablo::{Number, db::db};
use topcoat::{Result, context::Cx, icon::iconify::iconify_icon, view::View};

use super::{
    DepartmentResource, Employee, EmployeeResource, EmploymentType, Expense, ExpenseResource,
    ExpenseStatus, LeaveRequest, LeaveRequestResource, LeaveStatus,
};
use crate::access::{self, Domain};
use crate::projects::{ProjectResource, Timesheet, TimesheetResource};
use crate::shared::stats::{
    MONTH, alert, days_before, dollars, float, linked, percent, percent_change, record_tile, share,
    trended, week_labels, weekly_sums,
};

/// The HR dashboard.
pub struct HrDashboard;

impl Page for HrDashboard {
    fn declare() -> PageDef {
        PageDef::new()
            .slug("hr")
            .label(tablo::text!("page-hr-dashboard"))
            .description(tablo::text!("dashboard-hr-description"))
            .icon(iconify_icon!("lucide:briefcase"))
            .navigation_order(3)
            .access(|cx| access::reaches(cx, Domain::Hr))
            .widget::<WorkforceWidget>()
            .widget::<TimeWidget>()
            .widget::<HoursWidget>()
            .widget::<HeadcountWidget>()
            .widget::<ApprovalsWidget>()
            .widget::<LeaveQueueWidget>()
            .widget::<ExpenseQueueWidget>()
    }
}

/// The workforce: its size, its tenure, its contractors and the seats still open.
pub struct WorkforceWidget;

impl Widget for WorkforceWidget {
    fn declare() -> WidgetDef {
        WidgetDef::new().full_width()
    }

    async fn render(cx: &Cx) -> Result<impl View> {
        let today = Zoned::now().date();
        let employees = scoped_query::<EmployeeResource>(cx)?
            .exec(&mut db(cx)?)
            .await?;
        let departments = scoped_query::<DepartmentResource>(cx)?
            .exec(&mut db(cx)?)
            .await?;

        let active: Vec<&Employee> = employees
            .iter()
            .filter(|employee| employee.is_active)
            .collect();
        let tenure = active
            .iter()
            .map(|employee| days_before(today, employee.hire_date) as f64 / 365.0)
            .sum::<f64>()
            / active.len().max(1) as f64;
        let contractors = active
            .iter()
            .filter(|employee| employee.employment_type == EmploymentType::Contractor)
            .count();
        let capacity: i64 = departments
            .iter()
            .filter(|department| department.is_active)
            .map(|department| department.headcount_limit)
            .sum();
        let open_capacity = (capacity - active.len() as i64).max(0);

        let tiles = [
            record_tile::<EmployeeResource>(cx, |employee: &Employee| employee.created_at)
                .await?
                .map(|stat| {
                    stat.detail(tablo::text!(
                        "stat-detail-active",
                        count = active.len().to_string()
                    ))
                }),
            Some(
                Stat::new(
                    tablo::text!("stat-avg-tenure"),
                    tablo::t!(cx, "stat-years", years = Number::format_in(cx, tenure, 1)),
                )
                .detail(tablo::text!("stat-detail-tenure")),
            ),
            Some(
                Stat::new(
                    tablo::text!("stat-contractor-ratio"),
                    percent(cx, share(contractors as f64, active.len() as f64)),
                )
                .detail(tablo::text!(
                    "stat-detail-contractors",
                    count = contractors.to_string()
                )),
            ),
            Some(
                Stat::new(
                    tablo::text!("stat-open-capacity"),
                    open_capacity.to_string(),
                )
                .detail(tablo::text!(
                    "stat-detail-seats",
                    seats = capacity.to_string()
                )),
            ),
        ];
        Ok(tablo::stats(cx, tiles.into_iter().flatten()))
    }
}

/// The hours booked lately, how many were billable, and how far the projects have spent their
/// budgets.
pub struct TimeWidget;

impl Widget for TimeWidget {
    fn declare() -> WidgetDef {
        WidgetDef::new().full_width()
    }

    async fn render(cx: &Cx) -> Result<impl View> {
        let today = Zoned::now().date();
        let sheets = scoped_query::<TimesheetResource>(cx)?
            .exec(&mut db(cx)?)
            .await?;
        let ago: Vec<(i64, &Timesheet)> = sheets
            .iter()
            .map(|sheet| (days_before(today, sheet.date), sheet))
            .collect();
        // The hours booked in the month that began `from` days ago, and how many were billable.
        let booked = |from: i64| -> (f64, f64) {
            ago.iter()
                .filter(|(days, _)| (from..from + MONTH).contains(days))
                .fold((0.0, 0.0), |(hours, billable), (_, sheet)| {
                    let billable = if sheet.is_billable {
                        billable + sheet.hours
                    } else {
                        billable
                    };
                    (hours + sheet.hours, billable)
                })
        };
        let (hours, billable) = booked(0);
        let (previous, _) = booked(MONTH);
        let weeks = weekly_sums(ago.iter().map(|(days, sheet)| (*days, sheet.hours)));

        let hours_tile = trended(
            Stat::new(
                tablo::text!("stat-hours-booked"),
                format!("{} h", Number::format_in(cx, hours, 1)),
            )
            .chart(weeks),
            hours,
            previous,
            true,
        )
        .detail(percent_change(cx, hours, previous, MONTH));
        let billable_tile = Stat::new(
            tablo::text!("stat-billable-share"),
            percent(cx, share(billable, hours)),
        )
        .detail(tablo::text!(
            "stat-detail-billable",
            hours = Number::format_in(cx, billable, 1)
        ));
        let timesheets = record_tile::<TimesheetResource>(cx, |sheet: &Timesheet| sheet.created_at)
            .await?
            .map(|stat| stat.detail(tablo::text!("stat-detail-timesheets")));

        let projects = scoped_query::<ProjectResource>(cx)?
            .exec(&mut db(cx)?)
            .await?;
        let budget: f64 = projects.iter().map(|project| float(project.budget)).sum();
        let spent: f64 = projects.iter().map(|project| float(project.spent)).sum();
        let used = share(spent, budget);
        let budget_tile = Stat::new(tablo::text!("stat-project-budget"), percent(cx, used))
            .detail(tablo::text!(
                "stat-detail-budget",
                spent = dollars(cx, spent),
                budget = dollars(cx, budget)
            ))
            .color(if used > 100.0 {
                Color::Danger
            } else if used >= 90.0 {
                Color::Warning
            } else {
                Color::Gray
            });

        Ok(tablo::stats(
            cx,
            [
                Some(hours_tile),
                Some(billable_tile),
                timesheets,
                Some(budget_tile),
            ]
            .into_iter()
            .flatten(),
        ))
    }
}

/// The hours booked each week over the last twelve, billable and not, as a pair of bars. Only a
/// user who may open the timesheet list sees it.
pub struct HoursWidget;

impl Widget for HoursWidget {
    fn declare() -> WidgetDef {
        WidgetDef::new().access(can_list::<TimesheetResource>)
    }

    async fn render(cx: &Cx) -> Result<impl View> {
        let today = Zoned::now().date();
        let sheets = scoped_query::<TimesheetResource>(cx)?
            .exec(&mut db(cx)?)
            .await?;
        let billable = weekly_sums(
            sheets
                .iter()
                .filter(|sheet| sheet.is_billable)
                .map(|sheet| (days_before(today, sheet.date), sheet.hours)),
        );
        let other = weekly_sums(
            sheets
                .iter()
                .filter(|sheet| !sheet.is_billable)
                .map(|sheet| (days_before(today, sheet.date), sheet.hours)),
        );
        let chart = Chart::bar(week_labels(today))
            .heading(tablo::text!("chart-hours"))
            .description(tablo::text!("chart-weeks"))
            .series(Series::new(tablo::text!("series-billable"), billable).color(Color::Primary))
            .series(Series::new(tablo::text!("series-not-billable"), other).color(Color::Gray));
        Ok(tablo::chart(cx, chart))
    }
}

/// How many active employees each department holds, against the seats it may fill. Only a user
/// who may open the employee list sees it.
pub struct HeadcountWidget;

impl Widget for HeadcountWidget {
    fn declare() -> WidgetDef {
        WidgetDef::new().access(can_list::<EmployeeResource>)
    }

    async fn render(cx: &Cx) -> Result<impl View> {
        let employees = scoped_query::<EmployeeResource>(cx)?
            .exec(&mut db(cx)?)
            .await?;
        let mut departments = scoped_query::<DepartmentResource>(cx)?
            .exec(&mut db(cx)?)
            .await?;
        departments.retain(|department| department.is_active);
        departments.sort_by(|a, b| a.name.cmp(&b.name));
        let staffed = departments.iter().map(|department| {
            employees
                .iter()
                .filter(|employee| {
                    employee.is_active && employee.department_id == Some(department.id)
                })
                .count() as f64
        });
        let seats = departments
            .iter()
            .map(|department| department.headcount_limit as f64);
        let labels: Vec<Text> = departments
            .iter()
            .map(|department| department.name.clone().into())
            .collect();
        let chart = Chart::bar(labels)
            .heading(tablo::text!("chart-headcount"))
            .description(tablo::text!("chart-headcount-description"))
            .series(Series::new(tablo::text!("series-staffed"), staffed).color(Color::Primary))
            .series(Series::new(tablo::text!("series-seats"), seats).color(Color::Gray));
        Ok(tablo::chart(cx, chart))
    }
}

/// The queues someone must clear: leave to review and expenses to approve, and what has been paid
/// back. Shown to those who may open both lists.
pub struct ApprovalsWidget;

impl Widget for ApprovalsWidget {
    fn declare() -> WidgetDef {
        WidgetDef::new()
            .full_width()
            .access(|cx| can_list::<LeaveRequestResource>(cx) && can_list::<ExpenseResource>(cx))
    }

    async fn render(cx: &Cx) -> Result<impl View> {
        let leave = scoped_query::<LeaveRequestResource>(cx)?
            .exec(&mut db(cx)?)
            .await?;
        let expenses = scoped_query::<ExpenseResource>(cx)?
            .exec(&mut db(cx)?)
            .await?;
        let pending = leave
            .iter()
            .filter(|request| request.status == LeaveStatus::Pending)
            .count();
        let submitted = expenses
            .iter()
            .filter(|expense| expense.status == ExpenseStatus::Submitted)
            .count();
        let (reimbursed, paid) = expenses
            .iter()
            .filter(|expense| expense.status == ExpenseStatus::Reimbursed)
            .fold((0, 0.0), |(count, total), expense| {
                (count + 1, total + float(expense.total_amount))
            });

        let leave_url = tablo::url::resource::<LeaveRequestResource>(cx);
        let expense_url = tablo::url::resource::<ExpenseResource>(cx);
        Ok(tablo::stats(
            cx,
            [
                linked(
                    Stat::new(tablo::text!("stat-leave-to-review"), pending.to_string())
                        .detail(tablo::text!("stat-detail-pending-leave"))
                        .color(alert(pending, Color::Warning)),
                    leave_url.map(|url| format!("{url}?tab={}", LeaveStatus::Pending.value())),
                ),
                linked(
                    Stat::new(
                        tablo::text!("stat-expenses-to-review"),
                        submitted.to_string(),
                    )
                    .detail(tablo::text!("stat-detail-submitted"))
                    .color(alert(submitted, Color::Warning)),
                    expense_url
                        .map(|url| format!("{url}?tab={}", ExpenseStatus::Submitted.value())),
                ),
                Stat::new(tablo::text!("stat-reimbursed"), dollars(cx, paid)).detail(tablo::text!(
                    "stat-detail-reimbursed",
                    count = reimbursed.to_string()
                )),
            ],
        ))
    }
}

/// The leave requests waiting for a decision, the oldest first, under the leave list's columns.
/// Only a user who may open the leave list sees it.
pub struct LeaveQueueWidget;

impl Widget for LeaveQueueWidget {
    fn declare() -> WidgetDef {
        WidgetDef::new().access(can_list::<LeaveRequestResource>)
    }

    async fn render(cx: &Cx) -> Result<impl View> {
        let table = RecordTable::new()
            .heading(tablo::text!("widget-leave-queue"))
            .description(tablo::text!("widget-queue-description"))
            .column(LeaveRequest::fields().employee())
            .column(LeaveRequest::fields().kind())
            .column(LeaveRequest::fields().start_date())
            .filter(LeaveRequest::fields().status().eq(LeaveStatus::Pending))
            .order_by(LeaveRequest::fields().created_at().asc());
        tablo::record_table::<LeaveRequestResource>(cx, table).await
    }
}

/// The expenses submitted and not yet decided, the oldest first, under the expense list's columns.
/// Only a user who may open the expense list sees it.
pub struct ExpenseQueueWidget;

impl Widget for ExpenseQueueWidget {
    fn declare() -> WidgetDef {
        WidgetDef::new().access(can_list::<ExpenseResource>)
    }

    async fn render(cx: &Cx) -> Result<impl View> {
        let table = RecordTable::new()
            .heading(tablo::text!("widget-expense-queue"))
            .description(tablo::text!("widget-queue-description"))
            .column(Expense::fields().expense_number())
            .column(Expense::fields().employee())
            .column(Expense::fields().total_amount())
            .filter(Expense::fields().status().eq(ExpenseStatus::Submitted))
            .order_by(Expense::fields().submitted_at().asc());
        tablo::record_table::<ExpenseResource>(cx, table).await
    }
}
