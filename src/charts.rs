//! The cards and charts the dashboards draw.
//!
//! Tablo ships no dashboard widgets, so a dashboard is a [`Page`](tablo::Page)
//! that renders these directly: one card per figure, one column per series
//! point. The columns are plain markup, so a chart needs no script and no
//! client-side charting library.

use jiff::{Span, Timestamp, Zoned, tz::TimeZone};
use topcoat::{
    context::Cx,
    view::{BoxView, ViewExt, view},
};

/// The name of the month `months_ago` months back.
pub fn month_key(months_ago: i64) -> String {
    let now = Zoned::now();
    let date = now
        .date()
        .checked_sub(Span::new().months(months_ago))
        .unwrap_or(now.date());
    date.strftime("%b %Y").to_string()
}

/// The month `timestamp` falls in, spelled as [`month_key`] spells it.
pub fn month_of(timestamp: Timestamp) -> String {
    timestamp
        .to_zoned(TimeZone::UTC)
        .date()
        .strftime("%b %Y")
        .to_string()
}

/// One point of a chart: its label and its value.
pub type Point = (String, f64);

/// A card showing one figure, its caption and where it came from.
pub fn stat_card<'a>(
    cx: &'a Cx,
    label: impl Into<String>,
    value: impl Into<String>,
    caption: impl Into<String>,
) -> BoxView<'a> {
    let (label, value, caption) = (label.into(), value.into(), caption.into());
    view! {
        cx =>
        <div class="rounded-xl border border-border bg-card p-4 text-card-foreground">
            <p class="text-sm font-medium text-muted-foreground">(label)</p>
            <p class="mt-1 text-3xl font-semibold tracking-tight">(value)</p>
            <p class="mt-1 text-sm text-muted-foreground">(caption)</p>
        </div>
    }
    .boxed()
}

/// A card of `series`, drawn as one column per point.
pub fn bar_chart<'a>(
    cx: &'a Cx,
    caption: impl Into<String>,
    series: &[Point],
    format: fn(f64) -> String,
) -> BoxView<'a> {
    let caption = caption.into();
    let peak = series
        .iter()
        .map(|(_, value)| *value)
        .fold(0.0_f64, f64::max)
        .max(f64::MIN_POSITIVE);
    let bars: Vec<(String, String, String)> = series
        .iter()
        .map(|(label, value)| {
            (
                label.clone(),
                format!("height: {:.1}%", value / peak * 100.0),
                format(*value),
            )
        })
        .collect();
    view! {
        cx =>
        <figure class="rounded-xl border border-border bg-card p-4 text-card-foreground">
            <figcaption class="text-sm font-medium text-muted-foreground">(caption)</figcaption>
            <div class="mt-4 flex h-40 items-end gap-2">
                for (label, height, value) in bars {
                    <div class="flex h-full min-w-0 flex-1 flex-col justify-end gap-1">
                        <span class="text-center text-xs text-muted-foreground">(value)</span>
                        <div class="rounded-t bg-primary" style=(height)></div>
                        <span class="truncate text-center text-xs text-muted-foreground">(label)</span>
                    </div>
                }
            </div>
        </figure>
    }
    .boxed()
}

/// A table of `rows`, one `(label, value)` pair per line.
pub fn table_card<'a>(cx: &'a Cx, caption: impl Into<String>, rows: &[Point]) -> BoxView<'a> {
    let caption = caption.into();
    let rows: Vec<(String, String)> = rows
        .iter()
        .map(|(label, value)| (label.clone(), format!("{value:.0}")))
        .collect();
    view! {
        cx =>
        <figure class="rounded-xl border border-border bg-card p-4 text-card-foreground">
            <figcaption class="text-sm font-medium text-muted-foreground">(caption)</figcaption>
            <ol class="mt-3 flex flex-col gap-1">
                for (label, value) in rows {
                    <li class="flex items-center justify-between gap-4 text-sm">
                        <span class="truncate">(label)</span>
                        <span class="text-muted-foreground">(value)</span>
                    </li>
                }
            </ol>
        </figure>
    }
    .boxed()
}
