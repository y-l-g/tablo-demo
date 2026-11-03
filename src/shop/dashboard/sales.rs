//! The orders the sales figures read, and what they came to over a stretch of days.

use jiff::{Timestamp, Zoned, civil::Date, tz::TimeZone};
use tablo::db::db;
use tablo::prelude::*;
use topcoat::{Result, context::Cx};

use super::PERIOD;
use crate::shared::stats::{MONTH, WEEK, WEEKS, days_before, days_since, float, share};
use crate::shop::{Order, OrderResource, OrderStatus};

/// The period the sales figures cover, and every order placed before its end, each as a
/// [`Sale`] aged from it.
pub(super) async fn load_sales(cx: &Cx) -> Result<(Period, Vec<Sale>)> {
    let period = Period::of(cx)?;
    let orders = scoped_query::<OrderResource>(cx)?
        .exec(&mut db(cx)?)
        .await?;
    let sales = orders
        .iter()
        .filter(|order| order.created_at < period.until)
        .map(|order| Sale::of(period.until, order))
        .collect();
    Ok((period, sales))
}

/// What the sales figures cover, from the [`PERIOD`] filter: the tiles its days against as many
/// days before them, the charts the twelve weeks up to its last day. Without the filter, or with
/// either end open, the tiles cover the last [`MONTH`] up to the last day, or up to now.
pub(super) struct Period {
    /// How many days the tiles cover.
    pub(super) days: i64,
    /// Just past the last day, or now when the period ends today, later, or never.
    pub(super) until: Timestamp,
    /// The last day.
    pub(super) today: Date,
}

impl Period {
    pub(super) fn of(cx: &Cx) -> Result<Self> {
        let (first, last) = page_filters(cx).date_range(PERIOD);
        let now = Zoned::now();
        // A period ending today or later ends now: no order is placed in the future.
        let (until, today) = match last {
            Some(last) => match last.tomorrow()?.to_zoned(TimeZone::system())?.timestamp() {
                until if until < now.timestamp() => (until, last),
                _ => (now.timestamp(), now.date()),
            },
            None => (now.timestamp(), now.date()),
        };
        let days = match first {
            Some(first) => (days_before(today, first) + 1).max(1),
            None => MONTH,
        };
        Ok(Self { days, until, today })
    }
}

/// What the sales placed in each of the last [`WEEKS`] weeks came to, oldest week first.
pub(super) fn weeks_of(sales: &[Sale]) -> Vec<Totals> {
    (0..WEEKS as i64)
        .rev()
        .map(|week| Totals::window(sales, week * WEEK, WEEK))
        .collect()
}

/// One order as the sales figures read it: how many days ago it was placed, what it brought in
/// and whether it was cancelled.
pub(super) struct Sale {
    pub(super) age: i64,
    pub(super) revenue: f64,
    pub(super) cancelled: bool,
}

impl Sale {
    pub(super) fn of(now: Timestamp, order: &Order) -> Self {
        Self {
            age: days_since(now, order.created_at),
            revenue: order.total_price.map_or(0.0, float),
            cancelled: order.status == OrderStatus::Cancelled,
        }
    }
}

/// What the orders placed in a stretch of days came to.
#[derive(Default)]
pub(super) struct Totals {
    pub(super) placed: f64,
    pub(super) cancelled: f64,
    pub(super) sold: f64,
    pub(super) revenue: f64,
}

impl Totals {
    /// The orders placed `from` days ago, for `days` days.
    pub(super) fn window(sales: &[Sale], from: i64, days: i64) -> Self {
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
    pub(super) fn average(&self) -> f64 {
        if self.sold == 0.0 {
            0.0
        } else {
            self.revenue / self.sold
        }
    }

    /// The share of the orders placed that were cancelled, as a percentage.
    pub(super) fn cancellation(&self) -> f64 {
        share(self.cancelled, self.placed)
    }
}
