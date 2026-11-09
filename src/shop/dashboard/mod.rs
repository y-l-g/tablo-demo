//! The shop's dashboard: sales against the month before, revenue and orders by week, the
//! pipeline, the newest orders, the best sellers and the customers.

mod sales;

use std::collections::{HashMap, HashSet};

use tablo::prelude::*;
use tablo::{Number, db::db};
use topcoat::{Result, context::Cx, icon::iconify::iconify_icon, view::View};

use self::sales::{Totals, load_sales, weeks_of};
use super::{
    Customer, CustomerResource, Order, OrderItemResource, OrderResource, OrderStatus,
    PipelineWidget, Product, ProductResource,
};
use crate::access::{self, Domain};
use crate::shared::stats::{
    dollars, float, money, percent, percent_change, points_change, record_tile, share, trended,
    week_labels,
};

/// The shop dashboard's period filter, which the sales tiles and charts end on.
const PERIOD: &str = "period";

/// The shop's dashboard.
pub struct ShopDashboard;

impl Page for ShopDashboard {
    fn declare() -> PageDef {
        PageDef::new()
            .slug("shop")
            .label(tablo::text!("page-shop-dashboard"))
            .description(tablo::text!("dashboard-shop-description"))
            .icon(iconify_icon!("lucide:store"))
            .navigation_order(2)
            // The widgets read the shop's rows directly, so the page asks the role first.
            .access(|cx| access::reaches(cx, Domain::Shop))
            .filter(PageFilter::date_range(
                PERIOD,
                tablo::text!("dashboard-filter-period"),
            ))
            .widget::<SalesWidget>()
            .widget::<RevenueWidget>()
            .widget::<OrdersWidget>()
            .widget::<PipelineWidget>()
            .widget::<LatestOrdersWidget>()
            .widget::<TopProductsWidget>()
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
        let (period, sales) = load_sales(cx).await?;
        let days = period.days;
        let last = Totals::window(&sales, 0, days);
        let before = Totals::window(&sales, days, days);
        let weeks = weeks_of(&sales);

        let revenue = trended(
            Stat::new(
                tablo::text!("stat-revenue", days = days.to_string()),
                dollars(cx, last.revenue),
            )
            .chart(weeks.iter().map(|week| week.revenue)),
            last.revenue,
            before.revenue,
            true,
        )
        .detail(percent_change(cx, last.revenue, before.revenue, days));
        let placed = trended(
            Stat::new(
                tablo::text!("stat-orders-placed", days = days.to_string()),
                last.placed.to_string(),
            )
            .chart(weeks.iter().map(|week| week.placed)),
            last.placed,
            before.placed,
            true,
        )
        .detail(percent_change(cx, last.placed, before.placed, days));
        let average = trended(
            Stat::new(
                tablo::text!("stat-average-order", days = days.to_string()),
                money(cx, last.average()),
            )
            .chart(weeks.iter().map(Totals::average)),
            last.average(),
            before.average(),
            true,
        )
        .detail(percent_change(cx, last.average(), before.average(), days));
        // A cancellation rate is good when it falls.
        let cancellation = trended(
            Stat::new(
                tablo::text!("stat-cancellation-rate", days = days.to_string()),
                percent(cx, last.cancellation()),
            )
            .chart(weeks.iter().map(Totals::cancellation)),
            last.cancellation(),
            before.cancellation(),
            false,
        )
        .detail(points_change(
            cx,
            last.cancellation(),
            before.cancellation(),
            days,
        ));

        Ok(tablo::stats(cx, [revenue, placed, average, cancellation]))
    }
}

/// Revenue over the last twelve weeks, a point a week, oldest first. Only a user who may open the
/// order list sees it.
pub struct RevenueWidget;

impl Widget for RevenueWidget {
    fn declare() -> WidgetDef {
        WidgetDef::new().access(can_list::<OrderResource>)
    }

    async fn render(cx: &Cx) -> Result<impl View> {
        let (period, sales) = load_sales(cx).await?;
        let weeks = weeks_of(&sales);
        let chart = Chart::line(week_labels(period.today))
            .heading(tablo::text!("chart-revenue"))
            .description(tablo::text!("chart-weeks"))
            .series(Series::new(
                tablo::text!("series-revenue"),
                weeks.iter().map(|week| week.revenue),
            ));
        Ok(tablo::chart(cx, chart))
    }
}

/// The orders placed each week over the last twelve, oldest first. Only a user who may open the
/// order list sees it.
pub struct OrdersWidget;

impl Widget for OrdersWidget {
    fn declare() -> WidgetDef {
        WidgetDef::new().access(can_list::<OrderResource>)
    }

    async fn render(cx: &Cx) -> Result<impl View> {
        let (period, sales) = load_sales(cx).await?;
        let weeks = weeks_of(&sales);
        let chart = Chart::bar(week_labels(period.today))
            .heading(tablo::text!("chart-orders"))
            .description(tablo::text!("chart-weeks"))
            .series(
                Series::new(
                    tablo::text!("series-orders"),
                    weeks.iter().map(|week| week.placed),
                )
                .color(Color::Primary),
            );
        Ok(tablo::chart(cx, chart))
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
            .column(Order::fields().customer())
            .column(Order::fields().status())
            .column(Order::fields().total_price())
            .column(Order::fields().created_at())
            .order_by(Order::fields().created_at().desc());
        tablo::record_table::<OrderResource>(cx, table).await
    }
}

/// The products that sold most, by the units shipped, each linking to its record.
pub struct TopProductsWidget;

impl Widget for TopProductsWidget {
    fn declare() -> WidgetDef {
        WidgetDef::new().access(can_list::<ProductResource>)
    }

    async fn render(cx: &Cx) -> Result<impl View> {
        let table = RecordTable::new()
            .heading(tablo::text!("widget-top-products"))
            .column(Product::fields().name())
            .column(Product::fields().units_sold())
            .order_by(Product::fields().units_sold().desc())
            .limit(5);
        tablo::record_table::<ProductResource>(cx, table).await
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
            .filter_map(|order| order.total_price.map(float))
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
                    percent(cx, share(repeat as f64, customers.len() as f64)),
                )
                .detail(tablo::text!(
                    "stat-detail-repeat",
                    repeat = repeat.to_string(),
                    total = customers.len().to_string()
                )),
                Stat::new(
                    tablo::text!("stat-avg-items-per-order"),
                    Number::format_in(cx, items_per_order, 1),
                )
                .detail(tablo::text!(
                    "stat-detail-items",
                    items = Number::format_in(cx, items as f64, 0),
                    orders = Number::format_in(cx, orders.len() as f64, 0)
                )),
                Stat::new(
                    tablo::text!("stat-revenue-per-customer"),
                    money(cx, revenue_per_customer),
                )
                .detail(tablo::text!(
                    "stat-detail-revenue",
                    total = dollars(cx, revenue)
                )),
            ]),
        ))
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
