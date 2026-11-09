//! What the dashboards' figures share: the windows they compare, the weeks their sparklines and
//! charts draw, and how a figure, its trend and its change are spelled.

use std::cmp::Ordering;

use jiff::{Timestamp, civil::Date};
use rust_decimal::{Decimal, prelude::ToPrimitive};
use tablo::prelude::*;
use tablo::{Number, db::db};
use topcoat::{Result, context::Cx};

/// The window the tiles compare, a month of days, against the month before it.
pub const MONTH: i64 = 30;
/// The sparklines draw this many weeks, oldest first.
pub const WEEK: i64 = 7;
pub const WEEKS: usize = 12;

/// The tile counting `R`'s records, with a sparkline of the weeks they were created in. Nothing
/// when the request may not list `R`, and nothing is read then. `created` reads a record's
/// creation time.
pub async fn record_tile<R: Resource>(
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

/// Each of the last [`WEEKS`] weeks, oldest first, labelled with the day it began, spelled as the
/// locale spells a day and a month.
pub fn week_labels(today: Date) -> Vec<Text> {
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

/// Sums `(days ago, value)` entries into each of the last [`WEEKS`] weeks, oldest first. An entry
/// from the future falls in no week.
pub fn weekly_sums(entries: impl IntoIterator<Item = (i64, f64)>) -> Vec<f64> {
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
pub fn days_since(now: Timestamp, then: Timestamp) -> i64 {
    (now.as_second() - then.as_second()).div_euclid(86_400)
}

/// Whole days from `date` to `today`, negative when `date` is ahead of it.
pub fn days_before(today: Date, date: Date) -> i64 {
    i64::from(today.since(date).map(|span| span.get_days()).unwrap_or(0))
}

/// Sets `stat`'s trend from `now` against `before`: an arrow, and a colour that reads the move as
/// good when `rising_is_good` says a rise is, or as bad.
pub fn trended(stat: Stat, now: f64, before: f64, rising_is_good: bool) -> Stat {
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

/// The line under a figure that moved from `before` to `now`: its change as a percentage, signed,
/// in the reader's locale: `+12.5%`, or `+12,5 %` in French.
pub fn percent_change(cx: &Cx, now: f64, before: f64, days: i64) -> Text {
    let change = if before == 0.0 {
        0.0
    } else {
        (now - before) / before * 100.0
    };
    tablo::text!(
        "stat-detail-change",
        change = format!("{}{}", plus(change), Number::percent_in(cx, change, 1)),
        days = days.to_string()
    )
}

/// The line under a percentage that moved from `before` to `now`: its change in points, signed,
/// in the reader's locale.
pub fn points_change(cx: &Cx, now: f64, before: f64, days: i64) -> Text {
    let change = now - before;
    tablo::text!(
        "stat-detail-points",
        change = format!("{}{}", plus(change), Number::format_in(cx, change, 1)),
        days = days.to_string()
    )
}

/// The sign a rise is spelled with: a fall carries its own.
fn plus(change: f64) -> &'static str {
    if change > 0.0 { "+" } else { "" }
}

/// `color` while `count` is above zero, and muted while nothing waits.
pub fn alert(count: usize, color: Color) -> Color {
    if count > 0 { color } else { Color::Gray }
}

/// `stat`, linked to `url` when there is one.
pub fn linked(stat: Stat, url: Option<String>) -> Stat {
    match url {
        Some(url) => stat.href(url),
        None => stat,
    }
}

/// The share of `total` that `part` is, as a percentage.
pub fn share(part: f64, total: f64) -> f64 {
    if total == 0.0 {
        0.0
    } else {
        part / total * 100.0
    }
}

/// An amount in dollars and cents in the reader's locale: `$1,234.50`, or `1 234,50 $` in French.
pub fn money(cx: &Cx, amount: f64) -> String {
    Currency::USD.format_in(cx, amount)
}

/// An amount in whole dollars in the reader's locale: `$1,235`, or `1 235 $` in French.
pub fn dollars(cx: &Cx, amount: f64) -> String {
    Currency::USD.decimals(0).format_in(cx, amount)
}

/// A percentage, rounded to one decimal, in the reader's locale: `12.5%`, or `12,5 %` in French.
pub fn percent(cx: &Cx, value: f64) -> String {
    Number::percent_in(cx, value, 1)
}

/// A money amount as a float, for the dashboards' figures and charts.
pub fn float(amount: Decimal) -> f64 {
    amount.to_f64().unwrap_or_default()
}
