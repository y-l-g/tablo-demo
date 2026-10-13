//! The panel's dashboards, one page each.

use std::collections::HashMap;

use jiff::{Span, Timestamp, Zoned};
use tablo::{Page, PageDef, db::db};
use topcoat::{
    Result,
    context::Cx,
    view::{BoxView, View, view},
};

use crate::{
    charts::{Point, bar_chart, date_of, month_key, month_of, stat_card, table_card},
    enums::{EmploymentType, OrderStatus, ProjectStatus},
    models::{
        Customer, Department, Employee, Expense, LeaveRequest, Order, OrderItem, Product, Project,
        Task, Timesheet,
    },
};

/// The panel home: what the demo covers.
pub struct Dashboard;

impl Page for Dashboard {
    fn declare() -> PageDef {
        PageDef::new()
            .icon(tablo::ui::icons::LAYOUT_DASHBOARD)
            .navigation_order(-2)
    }

    async fn render(cx: &Cx) -> Result<impl View> {
        let mut db = db(cx);
        let posts = crate::models::Post::all().exec(&mut db).await?.len();
        let products = Product::all().exec(&mut db).await?.len();
        let orders = Order::all().exec(&mut db).await?.len();
        let employees = Employee::all().exec(&mut db).await?.len();

        Ok(view! {
            cx =>
            tablo::ui::page(
                tablo::ui::page_header(
                    tablo::ui::page_title("Welcome to the Filament demo!")
                    tablo::ui::page_description(
                        "The same demo application, rebuilt on Tablo."
                    )
                )
                tablo::ui::page_content(
                    <div class="grid gap-4 sm:grid-cols-2 xl:grid-cols-4">
                        (stat_card(cx, "Posts", posts.to_string(), "the blog's articles"))
                        (stat_card(cx, "Products", products.to_string(), "what the shop sells"))
                        (stat_card(cx, "Orders", orders.to_string(), "placed by customers"))
                        (stat_card(cx, "Employees", employees.to_string(), "across every department"))
                    </div>
                    <div class="grid gap-6 lg:grid-cols-2">
                        tablo::ui::card(
                            tablo::ui::card_header(
                                tablo::ui::card_title("The demo's domains")
                                tablo::ui::card_description(
                                    "Each one is a resource group in the sidebar."
                                )
                            )
                            tablo::ui::card_content(
                                <ul class="flex flex-col gap-2 text-sm">
                                    <li><strong>"Blog"</strong>" — authors, categories and posts, with the comments on each post."</li>
                                    <li><strong>"Shop"</strong>" — brands, categories, products, customers and orders, with payments."</li>
                                    <li><strong>"HR"</strong>" — departments, employees, leave requests and expenses."</li>
                                    <li><strong>"Projects"</strong>" — projects, their tasks and their timesheets."</li>
                                </ul>
                            )
                        )
                        tablo::ui::card(
                            tablo::ui::card_header(
                                tablo::ui::card_title("What to try")
                                tablo::ui::card_description(
                                    "Pages that exercise Tablo's own features."
                                )
                            )
                            tablo::ui::card_content(
                                <ul class="flex flex-col gap-2 text-sm">
                                    <li>"Search, sort, filter and page any list; the export button writes a CSV."</li>
                                    <li>"Run a row action: publish a post, approve leave, ship an order."</li>
                                    <li>"Open a record: relations render as the related resource's own table."</li>
                                    <li>"Upload an image on a post or a product."</li>
                                </ul>
                            )
                        )
                    </div>
                )
            )
        })
    }
}

/// The shop's dashboard.
pub struct ShopDashboard;

impl Page for ShopDashboard {
    fn declare() -> PageDef {
        PageDef::new()
            .slug("shop")
            .label("Shop dashboard")
            .icon(tablo::ui::icons::INBOX)
            .navigation_order(2)
    }

    async fn render(cx: &Cx) -> Result<impl View> {
        let mut db = db(cx);
        let orders: Vec<Order> = Order::all().exec(&mut db).await?;
        let customers: Vec<Customer> = Customer::all().exec(&mut db).await?;
        let items: Vec<OrderItem> = OrderItem::all().exec(&mut db).await?;
        let products: Vec<Product> = Product::all().exec(&mut db).await?;

        let orders_per_customer = count_by(&orders, |order| order.customer_id);
        let repeat = customers
            .iter()
            .filter(|customer| {
                orders_per_customer
                    .get(&Some(customer.id))
                    .copied()
                    .unwrap_or(0)
                    >= 2
            })
            .count();
        // A cancelled order is not revenue, and it sells nothing.
        let sold: Vec<&Order> = orders
            .iter()
            .filter(|order| order.status != OrderStatus::Cancelled)
            .collect();
        let revenue: f64 = sold.iter().filter_map(|order| order.total_price).sum();
        let cancelled = orders
            .iter()
            .filter(|order| order.status == OrderStatus::Cancelled)
            .count();
        let repeat_rate = share(repeat, customers.len());
        let cancellation_rate = share(cancelled, orders.len());
        let revenue_per_customer = if customers.is_empty() {
            0.0
        } else {
            revenue / customers.len() as f64
        };
        let items_per_order = if orders.is_empty() {
            0.0
        } else {
            items.len() as f64 / orders.len() as f64
        };

        let revenue_by_month = monthly_total(&orders, 12, |order| {
            if order.status == OrderStatus::Cancelled {
                0.0
            } else {
                order.total_price.unwrap_or(0.0)
            }
        });
        let customers_by_month = monthly_total(&customers, 12, |_| 1.0);
        let segments = customer_segments(&orders_per_customer);
        let sold_ids: Vec<u64> = sold.iter().map(|order| order.id).collect();
        let top_products = top_products_by_revenue(
            items
                .iter()
                .filter(|item| item.order_id.is_some_and(|id| sold_ids.contains(&id))),
            &products,
        );

        Ok(view! {
            cx =>
            tablo::ui::page(
                tablo::ui::page_header(
                    tablo::ui::page_title("Shop dashboard")
                    tablo::ui::page_description(
                        "How the shop is doing, over every customer and order."
                    )
                )
                tablo::ui::page_content(
                    <div class="grid gap-4 sm:grid-cols-2 xl:grid-cols-4">
                        (stat_card(cx, "Repeat customer rate", percent(repeat_rate), format!("{repeat} of {} customers", customers.len())))
                        (stat_card(cx, "Avg items / order", format!("{items_per_order:.1}"), format!("{} items, {} orders", items.len(), orders.len())))
                        (stat_card(cx, "Cancellation rate", percent(cancellation_rate), format!("{cancelled} cancelled orders")))
                        (stat_card(cx, "Revenue / customer", crate::models::money(&revenue_per_customer), format!("{} total revenue", crate::models::dollars(revenue))))
                    </div>
                    <div class="grid gap-6 lg:grid-cols-2">
                        (bar_chart(cx, "Revenue by month", &revenue_by_month, crate::models::dollars))
                        (bar_chart(cx, "New customers by month", &customers_by_month, |value| format!("{value:.0}")))
                        (bar_chart(cx, "Customers by order count", &segments, |value| format!("{value:.0}")))
                        (table_card(cx, "Top products by revenue", &top_products))
                    </div>
                    (flagged_orders(cx, &orders))
                )
            )
        })
    }
}

/// The orders no one has picked up yet, oldest first.
fn flagged_orders<'a>(cx: &'a Cx, orders: &[Order]) -> BoxView<'a> {
    let now = Timestamp::now();
    let mut flagged: Vec<(String, f64)> = orders
        .iter()
        .filter(|order| {
            // A timestamp's span counts seconds, whatever its length.
            let age = now
                .since(order.created_at)
                .map(|span| span.get_seconds() / 86_400)
                .unwrap_or(0);
            match order.status {
                OrderStatus::New => age >= 3,
                OrderStatus::Processing => age >= 7,
                _ => false,
            }
        })
        .map(|order| (order.number.clone(), order.total_price.unwrap_or(0.0)))
        .collect();
    flagged.sort_by(|left, right| left.0.cmp(&right.0));
    flagged.truncate(8);
    table_card(cx, "Flagged orders (oldest first)", &flagged)
}

/// The HR dashboard.
pub struct HrDashboard;

impl Page for HrDashboard {
    fn declare() -> PageDef {
        PageDef::new()
            .slug("hr")
            .label("HR dashboard")
            .icon(tablo::ui::icons::USERS)
            .navigation_order(3)
    }

    async fn render(cx: &Cx) -> Result<impl View> {
        let mut db = db(cx);
        let employees: Vec<Employee> = Employee::all().exec(&mut db).await?;
        let departments: Vec<Department> = Department::all().exec(&mut db).await?;
        let projects: Vec<Project> = Project::all().exec(&mut db).await?;
        let leave: Vec<LeaveRequest> = LeaveRequest::all().exec(&mut db).await?;
        let timesheets: Vec<Timesheet> = Timesheet::all().exec(&mut db).await?;
        let expenses: Vec<Expense> = Expense::all().exec(&mut db).await?;
        let tasks: Vec<Task> = Task::all().exec(&mut db).await?;

        let today = Zoned::now().date();
        let tenure: f64 = employees
            .iter()
            .map(|employee| {
                today
                    .since(employee.hire_date)
                    .map(|span| span.get_days())
                    .unwrap_or(0) as f64
                    / 365.0
            })
            .sum::<f64>()
            / employees.len().max(1) as f64;
        let inactive = employees
            .iter()
            .filter(|employee| !employee.is_active)
            .count();
        let contractors = employees
            .iter()
            .filter(|employee| employee.employment_type == EmploymentType::Contractor)
            .count();
        let capacity: i64 = departments
            .iter()
            .map(|department| department.headcount_limit)
            .sum();
        let open_capacity = capacity
            - employees
                .iter()
                .filter(|employee| employee.is_active)
                .count() as i64;

        let leave_by_department = leave_by_department(&leave, &employees, &departments);
        let project_health = project_health(&projects);
        let utilization = utilization_by_week(&timesheets);
        let burn = burn_rate(&timesheets, &expenses, &projects);

        Ok(view! {
            cx =>
            tablo::ui::page(
                tablo::ui::page_header(
                    tablo::ui::page_title("HR dashboard")
                    tablo::ui::page_description(
                        "The workforce, its leave, its projects and what they cost."
                    )
                )
                tablo::ui::page_content(
                    <div class="grid gap-4 sm:grid-cols-2 xl:grid-cols-4">
                        (stat_card(cx, "Avg tenure", format!("{tenure:.1} yrs"), format!("{} employees", employees.len())))
                        (stat_card(cx, "Turnover rate", percent(share(inactive, employees.len())), format!("{inactive} inactive")))
                        (stat_card(cx, "Open capacity", open_capacity.to_string(), format!("{capacity} seats, {} filled", employees.len() - inactive)))
                        (stat_card(cx, "Contractor ratio", percent(share(contractors, employees.len())), format!("{contractors} contractors")))
                    </div>
                    <div class="grid gap-6 lg:grid-cols-2">
                        (bar_chart(cx, "Leave days by department", &leave_by_department, |value| format!("{value:.0}")))
                        (bar_chart(cx, "Budget used by project", &project_health, percent))
                        (bar_chart(cx, "Billable utilization by week", &utilization, percent))
                        (bar_chart(cx, "Cumulative spend", &burn, crate::models::dollars))
                    </div>
                    <div class="grid gap-4 sm:grid-cols-2 xl:grid-cols-4">
                        (stat_card(cx, "Tasks", tasks.len().to_string(), "across every project"))
                        (stat_card(cx, "Timesheets", timesheets.len().to_string(), "booked entries"))
                        (stat_card(cx, "Leave requests", leave.len().to_string(), "all statuses"))
                        (stat_card(cx, "Expenses", expenses.len().to_string(), "claims filed"))
                    </div>
                )
            )
        })
    }
}

/// Leave days per department, the busiest first.
fn leave_by_department(
    leave: &[LeaveRequest],
    employees: &[Employee],
    departments: &[Department],
) -> Vec<Point> {
    let department_of: HashMap<u64, u64> = employees
        .iter()
        .filter_map(|employee| employee.department_id.map(|id| (employee.id, id)))
        .collect();
    let mut days: HashMap<u64, f64> = HashMap::new();
    for request in leave {
        if let Some(department) = department_of.get(&request.employee_id) {
            *days.entry(*department).or_default() += request.days_requested;
        }
    }
    let mut points: Vec<Point> = departments
        .iter()
        .filter_map(|department| {
            let days = days.get(&department.id).copied().unwrap_or(0.0);
            (days > 0.0).then(|| (department.name.clone(), days))
        })
        .collect();
    points.sort_by(|left, right| right.1.total_cmp(&left.1));
    points
}

/// How much of each project's budget is spent.
fn project_health(projects: &[Project]) -> Vec<Point> {
    let mut points: Vec<Point> = projects
        .iter()
        .filter(|project| project.budget > 0.0 && project.status != ProjectStatus::Cancelled)
        .map(|project| (project.name.clone(), project.spent / project.budget * 100.0))
        .collect();
    points.sort_by(|left, right| right.1.total_cmp(&left.1));
    points.truncate(8);
    points
}

/// The share of the last eight weeks' hours that were billable.
fn utilization_by_week(timesheets: &[Timesheet]) -> Vec<Point> {
    let today = Zoned::now().date();
    let weeks: Vec<Point> = (0..8)
        .rev()
        .map(|weeks_ago| {
            let end = today
                .checked_sub(Span::new().days(weeks_ago * 7))
                .unwrap_or(today);
            let start = end.checked_sub(Span::new().days(6)).unwrap_or(end);
            let (billable, total) = timesheets
                .iter()
                .filter(|timesheet| timesheet.date >= start && timesheet.date <= end)
                .fold((0.0, 0.0), |(billable, total), timesheet| {
                    (
                        billable
                            + if timesheet.is_billable {
                                timesheet.hours
                            } else {
                                0.0
                            },
                        total + timesheet.hours,
                    )
                });
            (
                start.strftime("%b %d").to_string(),
                if total > 0.0 {
                    billable / total * 100.0
                } else {
                    0.0
                },
            )
        })
        .collect();
    weeks
}

/// What the projects have cost so far, month by month, against their budgets.
fn burn_rate(timesheets: &[Timesheet], expenses: &[Expense], projects: &[Project]) -> Vec<Point> {
    let budget: f64 = projects.iter().map(|project| project.budget).sum();
    let mut months: Vec<Point> = (0..6)
        .rev()
        .map(|months_ago| {
            let month = month_key(months_ago);
            let cost: f64 = timesheets
                .iter()
                .filter(|timesheet| date_of(timesheet.date) == month)
                .map(|timesheet| timesheet.total_cost)
                .sum::<f64>()
                + expenses
                    .iter()
                    .filter(|expense| month_of(expense.created_at) == month)
                    .map(|expense| expense.total_amount)
                    .sum::<f64>();
            (month, cost)
        })
        .collect();
    months.push(("Budget".to_string(), budget));
    months
}

/// How many customers bought once, a few times, often, or at all.
fn customer_segments(orders_per_customer: &HashMap<Option<u64>, usize>) -> Vec<Point> {
    let mut segments = vec![
        ("One-time (1)".to_string(), 0.0),
        ("Occasional (2-3)".to_string(), 0.0),
        ("Regular (4-9)".to_string(), 0.0),
        ("VIP (10+)".to_string(), 0.0),
    ];
    for count in orders_per_customer
        .iter()
        .filter_map(|(customer, count)| customer.is_some().then_some(count))
    {
        let bucket = match count {
            0 => continue,
            1 => 0,
            2..=3 => 1,
            4..=9 => 2,
            _ => 3,
        };
        segments[bucket].1 += 1.0;
    }
    segments
}

/// The eight products that took the most money.
fn top_products_by_revenue<'a>(
    items: impl Iterator<Item = &'a OrderItem>,
    products: &[Product],
) -> Vec<Point> {
    let names: HashMap<u64, &str> = products
        .iter()
        .map(|product| (product.id, product.name.as_str()))
        .collect();
    let mut revenue: HashMap<u64, f64> = HashMap::new();
    for item in items {
        if let Some(product_id) = item.product_id {
            *revenue.entry(product_id).or_default() += item.qty as f64 * item.unit_price;
        }
    }
    let mut points: Vec<Point> = revenue
        .into_iter()
        .filter_map(|(product_id, revenue)| {
            names
                .get(&product_id)
                .map(|name| ((*name).to_string(), revenue))
        })
        .collect();
    points.sort_by(|left, right| right.1.total_cmp(&left.1));
    points.truncate(8);
    points
}

/// How many rows of `rows` carry each key.
fn count_by<T>(rows: &[T], key: impl Fn(&T) -> Option<u64>) -> HashMap<Option<u64>, usize> {
    let mut counts: HashMap<Option<u64>, usize> = HashMap::new();
    for row in rows {
        *counts.entry(key(row)).or_default() += 1;
    }
    counts
}

/// What `rows` come to in each of the last `months` months, oldest first.
fn monthly_total<T>(rows: &[T], months: i64, amount: impl Fn(&T) -> f64) -> Vec<Point> {
    (0..months)
        .rev()
        .map(|ago| (month_key(ago), rows.iter().map(&amount).sum()))
        .collect()
}

/// The share of `total` that `part` is, as a percentage.
fn share(part: usize, total: usize) -> f64 {
    if total == 0 {
        0.0
    } else {
        part as f64 / total as f64 * 100.0
    }
}

/// A percentage, rounded to one decimal.
fn percent(value: f64) -> String {
    format!("{value:.1}%")
}
