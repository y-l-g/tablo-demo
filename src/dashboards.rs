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
    cards::{Point, stat_card, table_card},
    enums::{EmploymentType, OrderStatus},
    models::{
        Customer, Department, Employee, Expense, LeaveRequest, Order, OrderItem, Product, Task,
        Timesheet,
    },
};

/// The panel home: what the demo covers.
pub struct Dashboard;

impl Page for Dashboard {
    fn declare() -> PageDef {
        PageDef::new()
            .label("Welcome")
            .icon(crate::icons::WELCOME)
            .navigation_order(-2)
    }

    async fn render(cx: &Cx) -> Result<impl View> {
        let mut db = db(cx);
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
            .icon(crate::icons::SHOP_DASHBOARD)
            .navigation_order(2)
    }

    async fn render(cx: &Cx) -> Result<impl View> {
        let mut db = db(cx);
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
                        (table_card(cx, "Top products by revenue", &top_products))
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
    table_card(cx, "Flagged orders (oldest first)", &rows)
}

/// The HR dashboard.
pub struct HrDashboard;

impl Page for HrDashboard {
    fn declare() -> PageDef {
        PageDef::new()
            .slug("hr")
            .label("HR dashboard")
            .icon(crate::icons::HR_DASHBOARD)
            .navigation_order(3)
    }

    async fn render(cx: &Cx) -> Result<impl View> {
        let mut db = db(cx);
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
                        (stat_card(cx, "Avg tenure", format!("{tenure:.1} yrs"), format!("{active_count} active employees")))
                        (stat_card(cx, "Turnover rate", percent(share(inactive, employees.len())), format!("{inactive} of {} inactive", employees.len())))
                        (stat_card(cx, "Open capacity", open_capacity.to_string(), format!("{capacity} seats across the active departments")))
                        (stat_card(cx, "Contractor ratio", percent(share(contractors, active_count)), format!("{contractors} contractors")))
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
