//! The panel's dashboards.

use tablo::{Page, PageDef, db::db};
use topcoat::{Result, context::Cx, view::{View, view}};

/// The panel home: what the demo covers.
pub struct Dashboard;

impl Page for Dashboard {
    fn declare() -> PageDef {
        PageDef::new().icon(tablo::ui::icons::LAYOUT_DASHBOARD)
    }

    async fn render(cx: &Cx) -> Result<impl View> {
        let _ = db(cx);
        Ok(view! { cx => <p>"Dashboard"</p> })
    }
}

/// The shop's dashboard.
pub struct ShopDashboard;

impl Page for ShopDashboard {
    fn declare() -> PageDef {
        PageDef::new().slug("shop").label("Shop dashboard")
    }

    async fn render(cx: &Cx) -> Result<impl View> {
        Ok(view! { cx => <p>"Shop dashboard"</p> })
    }
}

/// The HR dashboard.
pub struct HrDashboard;

impl Page for HrDashboard {
    fn declare() -> PageDef {
        PageDef::new().slug("hr").label("HR dashboard")
    }

    async fn render(cx: &Cx) -> Result<impl View> {
        Ok(view! { cx => <p>"HR dashboard"</p> })
    }
}
