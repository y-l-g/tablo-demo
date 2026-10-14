//! The cards the dashboards draw.
//!
//! Tablo ships no dashboard widgets, so a dashboard is a [`Page`](tablo::Page)
//! that renders these directly: one card per figure or list.

use topcoat::{
    context::Cx,
    view::{BoxView, ViewExt, view},
};

/// One row of a card: its label and its value.
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
