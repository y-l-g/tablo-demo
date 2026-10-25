//! The panel's dashboards, one page each.

use std::collections::HashMap;

use jiff::{Timestamp, Zoned};
use tablo::{Page, PageDef, db::db};
use topcoat::{
    Result,
    context::Cx,
    view::{BoxView, View, view},
};

use crate::{
    blog::PostResource,
    cards::{Point, table_card},
    enums::{EmploymentType, OrderStatus},
    hr::{
        EmployeeResource, ExpenseResource, LeaveRequestResource, TaskResource, TimesheetResource,
    },
    models::{
        Customer, Department, Employee, Expense, LeaveRequest, Order, OrderItem, Product, Task,
        Timesheet,
    },
    shop::{OrderResource, ProductResource},
};

/// The panel home: what the demo covers.
pub struct Dashboard;

impl Page for Dashboard {
    fn declare() -> PageDef {
        PageDef::new()
            .label(tablo::text!("page-welcome"))
            .icon(crate::icons::WELCOME)
            .navigation_order(-2)
    }

    async fn render(cx: &Cx) -> Result<impl View> {
        let mut db = db(cx)?;
        let posts = crate::models::Post::all().exec(&mut db).await?.len();
        let products = Product::all().exec(&mut db).await?.len();
        let orders = Order::filter(Order::fields().deleted_at().is_none())
            .exec(&mut db)
            .await?
            .len();
        let employees = Employee::filter(Employee::fields().deleted_at().is_none())
            .exec(&mut db)
            .await?
            .len();

        Ok(view! {
            cx =>
            tablo::ui::page(
                tablo::ui::page_header(
                    tablo::ui::page_title((tablo::t!(cx, "page-welcome-title")))
                    tablo::ui::page_description((tablo::t!(cx, "page-welcome-description")))
                )
                tablo::ui::page_content(
                    <div class="grid gap-4 sm:grid-cols-2 xl:grid-cols-4">
                        tablo::ui::stat(
                            label: tablo::t!(cx, "resource-post.plural"),
                            value: posts.to_string(),
                            detail: tablo::t!(cx, "stat-detail-posts"),
                            icon: crate::icons::POSTS,
                            href: tablo::url::resource::<PostResource>(cx)
                        )
                        tablo::ui::stat(
                            label: tablo::t!(cx, "resource-product.plural"),
                            value: products.to_string(),
                            detail: tablo::t!(cx, "stat-detail-products"),
                            icon: crate::icons::PRODUCTS,
                            href: tablo::url::resource::<ProductResource>(cx)
                        )
                        tablo::ui::stat(
                            label: tablo::t!(cx, "resource-order.plural"),
                            value: orders.to_string(),
                            detail: tablo::t!(cx, "stat-detail-orders"),
                            icon: crate::icons::ORDERS,
                            href: tablo::url::resource::<OrderResource>(cx)
                        )
                        tablo::ui::stat(
                            label: tablo::t!(cx, "resource-employee.plural"),
                            value: employees.to_string(),
                            detail: tablo::t!(cx, "stat-detail-employees"),
                            icon: crate::icons::EMPLOYEES,
                            href: tablo::url::resource::<EmployeeResource>(cx)
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
            .label(tablo::text!("page-shop-dashboard"))
            .icon(crate::icons::SHOP_DASHBOARD)
            .navigation_order(2)
    }

    async fn render(cx: &Cx) -> Result<impl View> {
        let mut db = db(cx)?;
        let orders: Vec<Order> = Order::filter(Order::fields().deleted_at().is_none())
            .exec(&mut db)
            .await?;
        let customers: Vec<Customer> = Customer::filter(Customer::fields().deleted_at().is_none())
            .exec(&mut db)
            .await?;
        let visible: Vec<u64> = orders.iter().map(|order| order.id).collect();
        let items: Vec<OrderItem> = OrderItem::all()
            .exec(&mut db)
            .await?
            .into_iter()
            .filter(|item| item.order_id.is_some_and(|id| visible.contains(&id)))
            .collect();
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
        let sold_ids: Vec<u64> = sold.iter().map(|order| order.id).collect();
        let top_products = top_products_by_revenue(
            items
                .iter()
                .filter(|item| item.order_id.is_some_and(|id| sold_ids.contains(&id))),
            &products,
        );
        let customer_count = customers.len().to_string();
        let order_count = orders.len().to_string();
        let item_count = items.len().to_string();

        Ok(view! {
            cx =>
            tablo::ui::page(
                tablo::ui::page_header(
                    tablo::ui::page_title((tablo::t!(cx, "page-shop-dashboard")))
                    tablo::ui::page_description((tablo::t!(cx, "dashboard-shop-description")))
                )
                tablo::ui::page_content(
                    <div class="grid gap-4 sm:grid-cols-2 xl:grid-cols-4">
                        tablo::ui::stat(
                            label: tablo::t!(cx, "stat-repeat-customer-rate"),
                            value: percent(repeat_rate),
                            detail: tablo::t!(
                                cx,
                                "stat-detail-repeat",
                                repeat = repeat.to_string(),
                                total = customer_count
                            )
                        )
                        tablo::ui::stat(
                            label: tablo::t!(cx, "stat-avg-items-per-order"),
                            value: format!("{items_per_order:.1}"),
                            detail: tablo::t!(
                                cx,
                                "stat-detail-items",
                                items = item_count,
                                orders = order_count
                            )
                        )
                        tablo::ui::stat(
                            label: tablo::t!(cx, "stat-cancellation-rate"),
                            value: percent(cancellation_rate),
                            detail: tablo::t!(
                                cx,
                                "stat-detail-cancelled",
                                count = cancelled.to_string()
                            )
                        )
                        tablo::ui::stat(
                            label: tablo::t!(cx, "stat-revenue-per-customer"),
                            value: crate::models::money(&revenue_per_customer),
                            detail: tablo::t!(
                                cx,
                                "stat-detail-revenue",
                                total = crate::models::dollars(revenue)
                            )
                        )
                    </div>
                    <div class="grid gap-6 lg:grid-cols-2">
                        (table_card(cx, tablo::t!(cx, "caption-top-products"), &top_products))
                        (flagged_orders(cx, &orders))
                    </div>
                )
            )
        })
    }
}

/// The orders no one has picked up yet, oldest first.
fn flagged_orders<'a>(cx: &'a Cx, orders: &[Order]) -> BoxView<'a> {
    let now = Timestamp::now();
    let mut flagged: Vec<&Order> = orders
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
        .collect();
    flagged.sort_by_key(|order| order.created_at);
    let rows: Vec<Point> = flagged
        .into_iter()
        .take(5)
        .map(|order| (order.number.clone(), order.total_price.unwrap_or(0.0)))
        .collect();
    table_card(cx, tablo::t!(cx, "caption-flagged-orders"), &rows)
}

/// The HR dashboard.
pub struct HrDashboard;

impl Page for HrDashboard {
    fn declare() -> PageDef {
        PageDef::new()
            .slug("hr")
            .label(tablo::text!("page-hr-dashboard"))
            .icon(crate::icons::HR_DASHBOARD)
            .navigation_order(3)
    }

    async fn render(cx: &Cx) -> Result<impl View> {
        let mut db = db(cx)?;
        let employees: Vec<Employee> = Employee::filter(Employee::fields().deleted_at().is_none())
            .exec(&mut db)
            .await?;
        let departments: Vec<Department> = Department::all().exec(&mut db).await?;
        let leave: Vec<LeaveRequest> = LeaveRequest::all().exec(&mut db).await?;
        let timesheets: Vec<Timesheet> = Timesheet::all().exec(&mut db).await?;
        let expenses: Vec<Expense> = Expense::all().exec(&mut db).await?;
        let tasks: Vec<Task> = Task::all().exec(&mut db).await?;

        let today = Zoned::now().date();
        let active = employees.iter().filter(|employee| employee.is_active);
        let active_count = active.clone().count();
        let tenure: f64 = active
            .clone()
            .map(|employee| {
                today
                    .since(employee.hire_date)
                    .map(|span| span.get_days())
                    .unwrap_or(0) as f64
                    / 365.0
            })
            .sum::<f64>()
            / active_count.max(1) as f64;
        let inactive = employees
            .iter()
            .filter(|employee| !employee.is_active)
            .count();
        let contractors = active
            .clone()
            .filter(|employee| employee.employment_type == EmploymentType::Contractor)
            .count();
        let capacity: i64 = departments
            .iter()
            .filter(|department| department.is_active)
            .map(|department| department.headcount_limit)
            .sum();
        let open_capacity = (capacity - active_count as i64).max(0);
        let employee_count = employees.len().to_string();

        Ok(view! {
            cx =>
            tablo::ui::page(
                tablo::ui::page_header(
                    tablo::ui::page_title((tablo::t!(cx, "page-hr-dashboard")))
                    tablo::ui::page_description((tablo::t!(cx, "dashboard-hr-description")))
                )
                tablo::ui::page_content(
                    <div class="grid gap-4 sm:grid-cols-2 xl:grid-cols-4">
                        tablo::ui::stat(
                            label: tablo::t!(cx, "stat-avg-tenure"),
                            value: tablo::t!(cx, "stat-years", years = format!("{tenure:.1}")),
                            detail: tablo::t!(
                                cx,
                                "stat-detail-active",
                                count = active_count.to_string()
                            )
                        )
                        tablo::ui::stat(
                            label: tablo::t!(cx, "stat-turnover-rate"),
                            value: percent(share(inactive, employees.len())),
                            detail: tablo::t!(
                                cx,
                                "stat-detail-inactive",
                                inactive = inactive.to_string(),
                                total = employee_count
                            )
                        )
                        tablo::ui::stat(
                            label: tablo::t!(cx, "stat-open-capacity"),
                            value: open_capacity.to_string(),
                            detail: tablo::t!(cx, "stat-detail-seats", seats = capacity.to_string())
                        )
                        tablo::ui::stat(
                            label: tablo::t!(cx, "stat-contractor-ratio"),
                            value: percent(share(contractors, active_count)),
                            detail: tablo::t!(
                                cx,
                                "stat-detail-contractors",
                                count = contractors.to_string()
                            )
                        )
                    </div>
                    <div class="grid gap-4 sm:grid-cols-2 xl:grid-cols-4">
                        tablo::ui::stat(
                            label: tablo::t!(cx, "resource-task.plural"),
                            value: tasks.len().to_string(),
                            detail: tablo::t!(cx, "stat-detail-tasks"),
                            icon: crate::icons::TASKS,
                            href: tablo::url::resource::<TaskResource>(cx)
                        )
                        tablo::ui::stat(
                            label: tablo::t!(cx, "resource-timesheet.plural"),
                            value: timesheets.len().to_string(),
                            detail: tablo::t!(cx, "stat-detail-timesheets"),
                            icon: crate::icons::TIMESHEETS,
                            href: tablo::url::resource::<TimesheetResource>(cx)
                        )
                        tablo::ui::stat(
                            label: tablo::t!(cx, "resource-leave-request.plural"),
                            value: leave.len().to_string(),
                            detail: tablo::t!(cx, "stat-detail-leave"),
                            icon: crate::icons::LEAVE_REQUESTS,
                            href: tablo::url::resource::<LeaveRequestResource>(cx)
                        )
                        tablo::ui::stat(
                            label: tablo::t!(cx, "resource-expense.plural"),
                            value: expenses.len().to_string(),
                            detail: tablo::t!(cx, "stat-detail-expenses"),
                            icon: crate::icons::EXPENSES,
                            href: tablo::url::resource::<ExpenseResource>(cx)
                        )
                    </div>
                )
            )
        })
    }
}

/// The ten products that took the most money.
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
    points.truncate(10);
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
