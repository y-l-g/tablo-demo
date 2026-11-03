//! The order pipeline: where the orders sit now, and those that have waited too long.

use jiff::Timestamp;
use tablo::prelude::*;
use tablo::{FormScalar, db};
use topcoat::{Result, context::Cx, view::View};

use super::OrderResource;
use crate::shared::stats::{alert, linked, percent, share};
use crate::shop::{Order, OrderStatus};

/// How long an order waits before the pipeline flags it: three days as new, seven as processing.
const NEW_FLAG_DAYS: i64 = 3;
const PROCESSING_FLAG_DAYS: i64 = 7;

/// Where the shop's orders sit now, a tile per status linked to the orders in it, and the orders
/// that have waited too long. The shop dashboard shows it, and so does the order list, where the
/// figures follow the list's tab, search and filters.
pub struct PipelineWidget;

impl Widget for PipelineWidget {
    fn declare() -> WidgetDef {
        WidgetDef::new()
            .full_width()
            .access(can_list::<OrderResource>)
    }

    async fn render(cx: &Cx) -> Result<impl View> {
        let orders = list_query::<OrderResource>(cx).await?;
        let total = db::count(cx, orders.clone()).await?;
        let url = tablo::url::resource::<OrderResource>(cx);
        let mut tiles = Vec::new();
        for option in [
            OrderStatus::New,
            OrderStatus::Processing,
            OrderStatus::Shipped,
            OrderStatus::Delivered,
            OrderStatus::Cancelled,
        ] {
            let count = db::count(
                cx,
                orders.clone().filter(Order::fields().status().eq(option)),
            )
            .await?;
            let tile = Stat::new(option.label(), count.to_string())
                .detail(tablo::text!(
                    "stat-detail-share",
                    share = percent(share(count as f64, total as f64))
                ))
                .color(option.color().unwrap_or(Color::Gray));
            tiles.push(linked(
                tile,
                url.as_ref()
                    .map(|url| format!("{url}?tab={}", option.value())),
            ));
        }

        let now = Timestamp::now();
        let waited = |days: i64| now - jiff::SignedDuration::from_hours(days * 24);
        let fields = Order::fields();
        let late = fields
            .status()
            .eq(OrderStatus::New)
            .and(fields.created_at().le(waited(NEW_FLAG_DAYS)))
            .or(fields
                .status()
                .eq(OrderStatus::Processing)
                .and(fields.created_at().le(waited(PROCESSING_FLAG_DAYS))));
        let flagged = db::count(cx, orders.filter(late)).await?;
        // Oldest first, the orders list opens on the longest waits.
        tiles.push(linked(
            Stat::new(tablo::text!("stat-flagged-orders"), flagged.to_string())
                .detail(tablo::text!(
                    "stat-detail-flagged",
                    new = NEW_FLAG_DAYS.to_string(),
                    processing = PROCESSING_FLAG_DAYS.to_string()
                ))
                .color(alert(flagged as usize, Color::Danger)),
            url.map(|url| format!("{url}?sort=created_at&dir=asc")),
        ));
        Ok(tablo::stats(cx, tiles))
    }
}
