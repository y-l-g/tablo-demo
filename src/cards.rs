//! The lists the dashboards draw.
//!
//! The figures are Tablo's `ui::stat` tiles; Tablo ships no list widget, so a
//! dashboard draws its ranked lists here.

use topcoat::{
    context::Cx,
    view::{BoxView, ViewExt, view},
};

/// One row of a card: its label and its value.
pub type Point = (String, f64);

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
