//! The panel home, which both panels open on: how many records the demo holds, revenue and
//! hours by week, and the newest posts.

use tablo::prelude::*;
use topcoat::{Result, context::Cx, icon::iconify::iconify_icon, view::View};

use crate::blog::{Post, PostResource};
use crate::hr::{Employee, EmployeeResource, HoursWidget};
use crate::shared::stats::record_tile;
use crate::shop::{Order, OrderResource, Product, ProductResource, RevenueWidget};

/// The panel home: how many records of each resource the demo holds.
pub struct Dashboard;

impl Page for Dashboard {
    fn declare() -> PageDef {
        PageDef::new()
            .label(tablo::text!("page-welcome"))
            .title(tablo::text!("page-welcome-title"))
            .description(tablo::text!("page-welcome-description"))
            .icon(iconify_icon!("lucide:layout-dashboard"))
            .navigation_order(-2)
            .widget::<RecordsWidget>()
            .widget::<RevenueWidget>()
            .widget::<HoursWidget>()
            .widget::<LatestPostsWidget>()
    }
}

/// One tile per resource, counting its records and sparkling the weeks they were made in.
pub struct RecordsWidget;

impl Widget for RecordsWidget {
    fn declare() -> WidgetDef {
        WidgetDef::new().full_width()
    }

    async fn render(cx: &Cx) -> Result<impl View> {
        let tiles = [
            record_tile::<PostResource>(cx, |post: &Post| post.created_at)
                .await?
                .map(|stat| stat.detail(tablo::text!("stat-detail-posts"))),
            record_tile::<ProductResource>(cx, |product: &Product| product.created_at)
                .await?
                .map(|stat| stat.detail(tablo::text!("stat-detail-products"))),
            record_tile::<OrderResource>(cx, |order: &Order| order.created_at)
                .await?
                .map(|stat| stat.detail(tablo::text!("stat-detail-orders"))),
            record_tile::<EmployeeResource>(cx, |employee: &Employee| employee.created_at)
                .await?
                .map(|stat| stat.detail(tablo::text!("stat-detail-employees"))),
        ];
        Ok(tablo::stats(cx, tiles.into_iter().flatten()))
    }
}

/// The newest posts, under the post list's columns, each linking to its page, with a link to the
/// full list. Only a user who may open the post list sees it.
pub struct LatestPostsWidget;

impl Widget for LatestPostsWidget {
    fn declare() -> WidgetDef {
        WidgetDef::new()
            .full_width()
            .access(can_list::<PostResource>)
    }

    async fn render(cx: &Cx) -> Result<impl View> {
        let table = RecordTable::new()
            .heading(tablo::text!("widget-latest-posts"))
            .column(Post::fields().title())
            .column(Post::fields().author())
            .column(Post::fields().published_at())
            .order_by(Post::fields().created_at().desc());
        tablo::record_table::<PostResource>(cx, table).await
    }
}
