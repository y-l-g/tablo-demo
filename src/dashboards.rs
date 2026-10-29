//! The panel's dashboards, and the widgets above three lists. Each page is a grid of Tablo widgets:
//! rows of stat tiles that read the seeded data, with trends, sparklines and links included; line
//! and bar charts over the last twelve weeks; and tables of the newest records and of the queues
//! awaiting a decision. The orders, leave requests and posts lists show their figures above their
//! tables.

use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
};

use jiff::{Timestamp, Zoned, civil::Date};
use tablo::{
    Chart, Color, Currency, FormScalar, Page, PageDef, RecordTable, Resource, Series, Stat, Text,
    Trend, Widget, WidgetDef, can_list, db::db, scoped_query,
};
use topcoat::{
    Result,
    context::Cx,
    view::{BoxView, View, ViewExt, view},
};

use crate::{
    blog::PostResource,
    enums::{EmploymentType, ExpenseStatus, LeaveStatus, OrderStatus},
    hr::{
        DepartmentResource, EmployeeResource, ExpenseResource, LeaveRequestResource,
        ProjectResource, TimesheetResource,
    },
    models::{Customer, Employee, Expense, LeaveRequest, Order, Post, Product, Timesheet},
    roles::{self, Domain},
    shop::{CustomerResource, OrderItemResource, OrderResource, ProductResource},
};

/// The window the tiles compare, a month of days, against the month before it.
const MONTH: i64 = 30;
/// The sparklines draw this many weeks, oldest first.
const WEEK: i64 = 7;
const WEEKS: usize = 12;
/// How long an order waits before the pipeline flags it: three days as new, seven as processing.
const NEW_FLAG_DAYS: i64 = 3;
const PROCESSING_FLAG_DAYS: i64 = 7;

/// The panel home: how many records of each resource the demo holds.
pub struct Dashboard;

impl Page for Dashboard {
    fn declare() -> PageDef {
        PageDef::new()
            .label(tablo::text!("page-welcome"))
            .title(tablo::text!("page-welcome-title"))
            .description(tablo::text!("page-welcome-description"))
            .icon(crate::icons::WELCOME)
            .navigation_order(-2)
            .widget::<RecordsWidget>()
            .widget::<LatestPostsWidget>()
    }
}

/// One tile per resource, counting its records and sparkling the weeks they were made in.
pub struct RecordsWidget;

impl Widget for RecordsWidget {
    fn declare() -> WidgetDef {
        WidgetDef::new().full_width()
    }

    async fn render(cx: &Cx) -> Result<impl View> {
        let tiles = [
            record_tile::<PostResource>(cx, |post: &Post| post.created_at)
                .await?
                .map(|stat| stat.detail(tablo::text!("stat-detail-posts"))),
            record_tile::<ProductResource>(cx, |product: &Product| product.created_at)
                .await?
                .map(|stat| stat.detail(tablo::text!("stat-detail-products"))),
            record_tile::<OrderResource>(cx, |order: &Order| order.created_at)
                .await?
                .map(|stat| stat.detail(tablo::text!("stat-detail-orders"))),
            record_tile::<EmployeeResource>(cx, |employee: &Employee| employee.created_at)
                .await?
                .map(|stat| stat.detail(tablo::text!("stat-detail-employees"))),
        ];
        Ok(tablo::stats(cx, tiles.into_iter().flatten()))
    }
}

/// The newest posts, under the post list's columns, each linking to its page, with a link to the
/// full list. Only a user who may open the post list sees it.
pub struct LatestPostsWidget;

impl Widget for LatestPostsWidget {
    fn declare() -> WidgetDef {
        WidgetDef::new()
            .full_width()
            .access(can_list::<PostResource>)
    }

    async fn render(cx: &Cx) -> Result<impl View> {
        let table = RecordTable::new()
            .heading(tablo::text!("widget-latest-posts"))
            .column(Post::fields().title())
            // A relation column is named by its label's key, as the list names it.
            .column_named("label-author")
            .column(Post::fields().published_at())
            .order_by(Post::fields().created_at().desc());
        tablo::record_table::<PostResource>(cx, table).await
    }
}

/// The shop's dashboard.
pub struct ShopDashboard;

impl Page for ShopDashboard {
    fn declare() -> PageDef {
        PageDef::new()
            .slug("shop")
            .label(tablo::text!("page-shop-dashboard"))
            .description(tablo::text!("dashboard-shop-description"))
            .icon(crate::icons::SHOP_DASHBOARD)
            .navigation_order(2)
            // The widgets read the shop's rows directly, so the page asks the role first.
            .access(|cx| roles::reaches(cx, Domain::Shop))
            .widget::<SalesWidget>()
            .widget::<RevenueWidget>()
            .widget::<PipelineWidget>()
            .widget::<LatestOrdersWidget>()
            .widget::<CustomersWidget>()
    }
}

/// What the shop sold over the last month, against the month before it.
pub struct SalesWidget;

impl Widget for SalesWidget {
    fn declare() -> WidgetDef {
        WidgetDef::new().full_width()
    }

    async fn render(cx: &Cx) -> Result<impl View> {
        let sales = load_sales(cx).await?;
        let last = Totals::window(&sales, 0, MONTH);
        let before = Totals::window(&sales, MONTH, MONTH);
        let weeks = weeks_of(&sales);

        let revenue = trended(
            Stat::new(tablo::text!("stat-revenue"), dollars(last.revenue))
                .chart(weeks.iter().map(|week| week.revenue)),
            last.revenue,
            before.revenue,
            true,
        )
        .detail(percent_change(last.revenue, before.revenue));
        let placed = trended(
            Stat::new(tablo::text!("stat-orders-placed"), last.placed.to_string())
                .chart(weeks.iter().map(|week| week.placed)),
            last.placed,
            before.placed,
            true,
        )
        .detail(percent_change(last.placed, before.placed));
        let average = trended(
            Stat::new(tablo::text!("stat-average-order"), money(last.average()))
                .chart(weeks.iter().map(Totals::average)),
            last.average(),
            before.average(),
            true,
        )
        .detail(percent_change(last.average(), before.average()));
        // A cancellation rate is good when it falls.
        let cancellation = trended(
            Stat::new(
                tablo::text!("stat-cancellation-rate"),
                percent(last.cancellation()),
            )
            .chart(weeks.iter().map(Totals::cancellation)),
            last.cancellation(),
            before.cancellation(),
            false,
        )
        .detail(points_change(last.cancellation(), before.cancellation()));

        Ok(tablo::stats(cx, [revenue, placed, average, cancellation]))
    }
}

/// Revenue over the last twelve weeks, a point a week, oldest first. Only a user who may open the
/// order list sees it.
pub struct RevenueWidget;

impl Widget for RevenueWidget {
    fn declare() -> WidgetDef {
        WidgetDef::new()
            .full_width()
            .access(can_list::<OrderResource>)
    }

    async fn render(cx: &Cx) -> Result<impl View> {
        let weeks = weeks_of(&load_sales(cx).await?);
        let chart = Chart::line(week_labels(Zoned::now().date()))
            .heading(tablo::text!("chart-revenue"))
            .description(tablo::text!("chart-weeks"))
            .series(Series::new(
                tablo::text!("series-revenue"),
                weeks.iter().map(|week| week.revenue),
            ));
        Ok(contained(cx, tablo::chart(cx, chart)))
    }
}

/// Where the shop's orders sit now, a tile per status linked to the orders in it, and the orders
/// that have waited too long. The shop dashboard shows it, and so does the order list.
pub struct PipelineWidget;

impl Widget for PipelineWidget {
    fn declare() -> WidgetDef {
        WidgetDef::new()
            .full_width()
            .access(can_list::<OrderResource>)
    }

    async fn render(cx: &Cx) -> Result<impl View> {
        let now = Timestamp::now();
        let orders = scoped_query::<OrderResource>(cx)?
            .exec(&mut db(cx)?)
            .await?;
        let url = tablo::url::resource::<OrderResource>(cx);
        let mut tiles: Vec<Stat> = [
            OrderStatus::New,
            OrderStatus::Processing,
            OrderStatus::Shipped,
            OrderStatus::Delivered,
            OrderStatus::Cancelled,
        ]
        .into_iter()
        .map(|status| {
            let count = orders.iter().filter(|order| order.status == status).count();
            let tile = Stat::new(status.label(), count.to_string())
                .detail(tablo::text!(
                    "stat-detail-share",
                    share = percent(share(count as f64, orders.len() as f64))
                ))
                .color(status.color().unwrap_or(Color::Gray));
            linked(
                tile,
                url.as_ref()
                    .map(|url| format!("{url}?tab={}", status.value())),
            )
        })
        .collect();

        let flagged = orders
            .iter()
            .filter(|order| {
                let age = days_since(now, order.created_at);
                match order.status {
                    OrderStatus::New => age >= NEW_FLAG_DAYS,
                    OrderStatus::Processing => age >= PROCESSING_FLAG_DAYS,
                    _ => false,
                }
            })
            .count();
        // Oldest first, the orders list opens on the longest waits.
        tiles.push(linked(
            Stat::new(tablo::text!("stat-flagged-orders"), flagged.to_string())
                .detail(tablo::text!(
                    "stat-detail-flagged",
                    new = NEW_FLAG_DAYS.to_string(),
                    processing = PROCESSING_FLAG_DAYS.to_string()
                ))
                .color(alert(flagged, Color::Danger)),
            url.map(|url| format!("{url}?sort=created_at&dir=asc")),
        ));
        Ok(tablo::stats(cx, tiles))
    }
}

/// The newest orders, under the order list's columns, each linking to its page, with a link to the
/// full list. Only a user who may open the order list sees it.
pub struct LatestOrdersWidget;

impl Widget for LatestOrdersWidget {
    fn declare() -> WidgetDef {
        WidgetDef::new()
            .full_width()
            .access(can_list::<OrderResource>)
    }

    async fn render(cx: &Cx) -> Result<impl View> {
        let table = RecordTable::new()
            .heading(tablo::text!("widget-latest-orders"))
            .column(Order::fields().number())
            .column_named("label-customer")
            .column(Order::fields().status())
            .column(Order::fields().total_price())
            .column(Order::fields().created_at())
            .order_by(Order::fields().created_at().desc());
        tablo::record_table::<OrderResource>(cx, table).await
    }
}

/// The customers the shop has, how many came back, and what each one is worth.
pub struct CustomersWidget;

impl Widget for CustomersWidget {
    fn declare() -> WidgetDef {
        WidgetDef::new().full_width()
    }

    async fn render(cx: &Cx) -> Result<impl View> {
        let orders = scoped_query::<OrderResource>(cx)?
            .exec(&mut db(cx)?)
            .await?;
        let customers = scoped_query::<CustomerResource>(cx)?
            .exec(&mut db(cx)?)
            .await?;
        let item_rows = scoped_query::<OrderItemResource>(cx)?
            .exec(&mut db(cx)?)
            .await?;

        let visible: HashSet<u64> = orders.iter().map(|order| order.id).collect();
        let items = item_rows
            .iter()
            .filter(|item| item.order_id.is_some_and(|id| visible.contains(&id)))
            .count();
        let per_customer = count_by(&orders, |order| order.customer_id);
        let repeat = customers
            .iter()
            .filter(|customer| per_customer.get(&Some(customer.id)).copied().unwrap_or(0) >= 2)
            .count();
        // A cancelled order is not revenue, and it sells nothing.
        let revenue: f64 = orders
            .iter()
            .filter(|order| order.status != OrderStatus::Cancelled)
            .filter_map(|order| order.total_price)
            .sum();
        let revenue_per_customer = if customers.is_empty() {
            0.0
        } else {
            revenue / customers.len() as f64
        };
        let items_per_order = if orders.is_empty() {
            0.0
        } else {
            items as f64 / orders.len() as f64
        };

        let customer_tile =
            record_tile::<CustomerResource>(cx, |customer: &Customer| customer.created_at).await?;
        Ok(tablo::stats(
            cx,
            customer_tile.into_iter().chain([
                Stat::new(
                    tablo::text!("stat-repeat-customer-rate"),
                    percent(share(repeat as f64, customers.len() as f64)),
                )
                .detail(tablo::text!(
                    "stat-detail-repeat",
                    repeat = repeat.to_string(),
                    total = customers.len().to_string()
                )),
                Stat::new(
                    tablo::text!("stat-avg-items-per-order"),
                    format!("{items_per_order:.1}"),
                )
                .detail(tablo::text!(
                    "stat-detail-items",
                    items = items.to_string(),
                    orders = orders.len().to_string()
                )),
                Stat::new(
                    tablo::text!("stat-revenue-per-customer"),
                    money(revenue_per_customer),
                )
                .detail(tablo::text!(
                    "stat-detail-revenue",
                    total = dollars(revenue)
                )),
            ]),
        ))
    }
}

/// The HR dashboard.
pub struct HrDashboard;

impl Page for HrDashboard {
    fn declare() -> PageDef {
        PageDef::new()
            .slug("hr")
            .label(tablo::text!("page-hr-dashboard"))
            .description(tablo::text!("dashboard-hr-description"))
            .icon(crate::icons::HR_DASHBOARD)
            .navigation_order(3)
            .access(|cx| roles::reaches(cx, Domain::Hr))
            .widget::<WorkforceWidget>()
            .widget::<TimeWidget>()
            .widget::<HoursWidget>()
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
                    tablo::t!(cx, "stat-years", years = format!("{tenure:.1}")),
                )
                .detail(tablo::text!("stat-detail-tenure")),
            ),
            Some(
                Stat::new(
                    tablo::text!("stat-contractor-ratio"),
                    percent(share(contractors as f64, active.len() as f64)),
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
            Stat::new(tablo::text!("stat-hours-booked"), format!("{hours:.1} h")).chart(weeks),
            hours,
            previous,
            true,
        )
        .detail(percent_change(hours, previous));
        let billable_tile = Stat::new(
            tablo::text!("stat-billable-share"),
            percent(share(billable, hours)),
        )
        .detail(tablo::text!(
            "stat-detail-billable",
            hours = format!("{billable:.1}")
        ));
        let timesheets = record_tile::<TimesheetResource>(cx, |sheet: &Timesheet| sheet.created_at)
            .await?
            .map(|stat| stat.detail(tablo::text!("stat-detail-timesheets")));

        let projects = scoped_query::<ProjectResource>(cx)?
            .exec(&mut db(cx)?)
            .await?;
        let budget: f64 = projects.iter().map(|project| project.budget).sum();
        let spent: f64 = projects.iter().map(|project| project.spent).sum();
        let used = share(spent, budget);
        let budget_tile = Stat::new(tablo::text!("stat-project-budget"), percent(used))
            .detail(tablo::text!(
                "stat-detail-budget",
                spent = dollars(spent),
                budget = dollars(budget)
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
        WidgetDef::new()
            .full_width()
            .access(can_list::<TimesheetResource>)
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
        Ok(contained(cx, tablo::chart(cx, chart)))
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
                (count + 1, total + expense.total_amount)
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
                Stat::new(tablo::text!("stat-reimbursed"), dollars(paid)).detail(tablo::text!(
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
        WidgetDef::new()
            .full_width()
            .access(can_list::<LeaveRequestResource>)
    }

    async fn render(cx: &Cx) -> Result<impl View> {
        let table = RecordTable::new()
            .heading(tablo::text!("widget-leave-queue"))
            .description(tablo::text!("widget-queue-description"))
            .column_named("label-employee")
            .column(LeaveRequest::fields().kind())
            .column(LeaveRequest::fields().start_date())
            .column(LeaveRequest::fields().end_date())
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
        WidgetDef::new()
            .full_width()
            .access(can_list::<ExpenseResource>)
    }

    async fn render(cx: &Cx) -> Result<impl View> {
        let table = RecordTable::new()
            .heading(tablo::text!("widget-expense-queue"))
            .description(tablo::text!("widget-queue-description"))
            .column(Expense::fields().expense_number())
            .column_named("label-employee")
            .column(Expense::fields().total_amount())
            .column(Expense::fields().submitted_at())
            .filter(Expense::fields().status().eq(ExpenseStatus::Submitted))
            .order_by(Expense::fields().submitted_at().asc());
        tablo::record_table::<ExpenseResource>(cx, table).await
    }
}

/// The leave requests by status, a tile per status linked to the requests in it, above the leave
/// list. Only a user who may open the leave list sees it.
pub struct LeaveStatusWidget;

impl Widget for LeaveStatusWidget {
    fn declare() -> WidgetDef {
        WidgetDef::new()
            .full_width()
            .access(can_list::<LeaveRequestResource>)
    }

    async fn render(cx: &Cx) -> Result<impl View> {
        let requests = scoped_query::<LeaveRequestResource>(cx)?
            .exec(&mut db(cx)?)
            .await?;
        let url = tablo::url::resource::<LeaveRequestResource>(cx);
        let tiles: Vec<Stat> = [
            LeaveStatus::Pending,
            LeaveStatus::Approved,
            LeaveStatus::Taken,
            LeaveStatus::Rejected,
            LeaveStatus::Cancelled,
        ]
        .into_iter()
        .map(|status| {
            let count = requests
                .iter()
                .filter(|request| request.status == status)
                .count();
            let tile = Stat::new(status.label(), count.to_string())
                .detail(tablo::text!(
                    "stat-detail-share-requests",
                    share = percent(share(count as f64, requests.len() as f64))
                ))
                .color(status.color().unwrap_or(Color::Gray));
            linked(
                tile,
                url.as_ref()
                    .map(|url| format!("{url}?tab={}", status.value())),
            )
        })
        .collect();
        Ok(tablo::stats(cx, tiles))
    }
}

/// The articles published and the drafts, above the post list. A post dated after today counts as
/// a draft, as the list's status column shows it. The list filters by date alone, so the tiles do
/// not link. Only a user who may open the post list sees it.
pub struct PostStatusWidget;

impl Widget for PostStatusWidget {
    fn declare() -> WidgetDef {
        WidgetDef::new()
            .full_width()
            .access(can_list::<PostResource>)
    }

    async fn render(cx: &Cx) -> Result<impl View> {
        let posts = scoped_query::<PostResource>(cx)?.exec(&mut db(cx)?).await?;
        let published = posts.iter().filter(|post| post.is_published()).count();
        let drafts = posts.len() - published;
        let detail = |count: usize| {
            tablo::text!(
                "stat-detail-share-articles",
                share = percent(share(count as f64, posts.len() as f64))
            )
        };
        Ok(tablo::stats(
            cx,
            [
                Stat::new(tablo::text!("status-published"), published.to_string())
                    .detail(detail(published))
                    .color(Color::Success),
                Stat::new(tablo::text!("status-draft"), drafts.to_string())
                    .detail(detail(drafts))
                    .color(Color::Gray),
            ],
        ))
    }
}

/// The tile counting `R`'s records, with a sparkline of the weeks they were created in. Nothing
/// when the request may not list `R`, and nothing is read then. `created` reads a record's
/// creation time.
async fn record_tile<R: Resource>(
    cx: &Cx,
    created: impl Fn(&R::Model) -> Timestamp,
) -> Result<Option<Stat>> {
    let Some(stat) = Stat::records::<R>(cx).await? else {
        return Ok(None);
    };
    let now = Timestamp::now();
    let records = scoped_query::<R>(cx)?.exec(&mut db(cx)?).await?;
    let weeks = weekly_sums(
        records
            .iter()
            .map(|record| (days_since(now, created(record)), 1.0)),
    );
    Ok(Some(stat.chart(weeks)))
}

/// `body`, clipped to its own cell. Tablo's charts and record tables label parts of themselves for
/// screen readers with `sr-only`, which positions them against the nearest positioned ancestor, the
/// page. A label that lands past the cell's edge, such as a chart's data table, scrolls the page
/// sideways; a positioned, clipped cell holds it in place.
fn contained<'a>(cx: &'a Cx, body: BoxView<'a>) -> BoxView<'a> {
    view! {
        cx =>
        <div class="relative overflow-hidden">
            (body)
        </div>
    }
    .boxed()
}

/// The orders the sales figures read, each as a [`Sale`].
async fn load_sales(cx: &Cx) -> Result<Vec<Sale>> {
    let now = Timestamp::now();
    let orders = scoped_query::<OrderResource>(cx)?
        .exec(&mut db(cx)?)
        .await?;
    Ok(orders.iter().map(|order| Sale::of(now, order)).collect())
}

/// What the sales placed in each of the last [`WEEKS`] weeks came to, oldest week first.
fn weeks_of(sales: &[Sale]) -> Vec<Totals> {
    (0..WEEKS as i64)
        .rev()
        .map(|week| Totals::window(sales, week * WEEK, WEEK))
        .collect()
}

/// Each of the last [`WEEKS`] weeks, oldest first, labelled with the day it began, spelled as the
/// locale spells a day and a month.
fn week_labels(today: Date) -> Vec<Text> {
    (0..WEEKS as i64)
        .rev()
        .map(|week| {
            // The week's newest day is `week` weeks before today, and its oldest one six days
            // before that.
            let began = today.saturating_sub(jiff::Span::new().days(week * WEEK + WEEK - 1));
            tablo::text!(
                "chart-week-start",
                month = format!("{:02}", began.month()),
                day = format!("{:02}", began.day())
            )
        })
        .collect()
}

/// One order as the sales figures read it: how many days ago it was placed, what it brought in
/// and whether it was cancelled.
struct Sale {
    age: i64,
    revenue: f64,
    cancelled: bool,
}

impl Sale {
    fn of(now: Timestamp, order: &Order) -> Self {
        Self {
            age: days_since(now, order.created_at),
            revenue: order.total_price.unwrap_or(0.0),
            cancelled: order.status == OrderStatus::Cancelled,
        }
    }
}

/// What the orders placed in a stretch of days came to.
#[derive(Default)]
struct Totals {
    placed: f64,
    cancelled: f64,
    sold: f64,
    revenue: f64,
}

impl Totals {
    /// The orders placed `from` days ago, for `days` days.
    fn window(sales: &[Sale], from: i64, days: i64) -> Self {
        sales
            .iter()
            .filter(|sale| (from..from + days).contains(&sale.age))
            .fold(Self::default(), |mut totals, sale| {
                totals.placed += 1.0;
                if sale.cancelled {
                    totals.cancelled += 1.0;
                } else {
                    totals.sold += 1.0;
                    totals.revenue += sale.revenue;
                }
                totals
            })
    }

    /// What a sold order brought in on average.
    fn average(&self) -> f64 {
        if self.sold == 0.0 {
            0.0
        } else {
            self.revenue / self.sold
        }
    }

    /// The share of the orders placed that were cancelled, as a percentage.
    fn cancellation(&self) -> f64 {
        share(self.cancelled, self.placed)
    }
}

/// Sums `(days ago, value)` entries into each of the last [`WEEKS`] weeks, oldest first. An entry
/// from the future falls in no week.
fn weekly_sums(entries: impl IntoIterator<Item = (i64, f64)>) -> Vec<f64> {
    let mut weeks = vec![0.0; WEEKS];
    for (days_ago, value) in entries {
        let week = days_ago.div_euclid(WEEK);
        if (0..WEEKS as i64).contains(&week) {
            weeks[WEEKS - 1 - week as usize] += value;
        }
    }
    weeks
}

/// Whole days from `then` to `now`, negative when `then` is ahead of it.
fn days_since(now: Timestamp, then: Timestamp) -> i64 {
    (now.as_second() - then.as_second()).div_euclid(86_400)
}

/// Whole days from `date` to `today`, negative when `date` is ahead of it.
fn days_before(today: Date, date: Date) -> i64 {
    i64::from(today.since(date).map(|span| span.get_days()).unwrap_or(0))
}

/// Sets `stat`'s trend from `now` against `before`: an arrow, and a colour that reads the move as
/// good when `rising_is_good` says a rise is, or as bad.
fn trended(stat: Stat, now: f64, before: f64, rising_is_good: bool) -> Stat {
    let (trend, color) = match now.total_cmp(&before) {
        Ordering::Greater => (
            Trend::Up,
            if rising_is_good {
                Color::Success
            } else {
                Color::Danger
            },
        ),
        Ordering::Less => (
            Trend::Down,
            if rising_is_good {
                Color::Danger
            } else {
                Color::Success
            },
        ),
        Ordering::Equal => (Trend::Flat, Color::Gray),
    };
    stat.trend(trend).color(color)
}

/// The line under a figure that moved from `before` to `now`: its change as a percentage.
fn percent_change(now: f64, before: f64) -> Text {
    let change = if before == 0.0 {
        0.0
    } else {
        (now - before) / before * 100.0
    };
    tablo::text!("stat-detail-change", change = format!("{change:+.1}%"))
}

/// The line under a percentage that moved from `before` to `now`: its change in points.
fn points_change(now: f64, before: f64) -> Text {
    tablo::text!(
        "stat-detail-points",
        change = format!("{:+.1}", now - before)
    )
}

/// `color` while `count` is above zero, and muted while nothing waits.
fn alert(count: usize, color: Color) -> Color {
    if count > 0 { color } else { Color::Gray }
}

/// `stat`, linked to `url` when there is one.
fn linked(stat: Stat, url: Option<String>) -> Stat {
    match url {
        Some(url) => stat.href(url),
        None => stat,
    }
}

/// How many rows of `rows` carry each key.
fn count_by<T>(rows: &[T], key: impl Fn(&T) -> Option<u64>) -> HashMap<Option<u64>, usize> {
    let mut counts: HashMap<Option<u64>, usize> = HashMap::new();
    for row in rows {
        *counts.entry(key(row)).or_default() += 1;
    }
    counts
}

/// The share of `total` that `part` is, as a percentage.
fn share(part: f64, total: f64) -> f64 {
    if total == 0.0 {
        0.0
    } else {
        part / total * 100.0
    }
}

/// An amount in dollars and cents, `$1,234.50`.
fn money(amount: f64) -> String {
    Currency::USD.format(amount)
}

/// An amount in whole dollars, `$1,235`.
fn dollars(amount: f64) -> String {
    Currency::USD.decimals(0).format(amount)
}

/// A percentage, rounded to one decimal.
fn percent(value: f64) -> String {
    format!("{value:.1}%")
}
