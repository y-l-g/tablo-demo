//! The two panels and the router that serves them.

use std::path::PathBuf;

use tablo::{Auth, Brand, Page, PageDef, Panel, RouterBuilderPanelExt, membership};
use toasty::Db;
use topcoat::{
    Result,
    asset::{AssetBundle, RouterBuilderAssetExt},
    context::Cx,
    font::{Font, fontsource::fontsource_font},
    router::{Router, RouterBuilderDiscoverExt},
    tailwind,
    view::{View, view},
};

use crate::{
    auth::DemoAuth,
    blog::{AuthorResource, CommentResource, PostCategoryResource, PostResource},
    dashboards::{Dashboard, HrDashboard, ShopDashboard},
    seed,
};

/// The theme's sans font, which the Filament demo names `Albert Sans`.
const ALBERT_SANS: Font = fontsource_font!(ALBERT_SANS, host: Asset);

/// The URL prefix uploaded images are served under.
pub const UPLOAD_URL_PREFIX: &str = "/uploads";

/// The login hint both panels show, with the demo account the seed creates.
pub fn login_hint() -> String {
    format!("{} / {}", seed::ADMIN_EMAIL, seed::ADMIN_PASSWORD)
}

/// The admin panel: every resource of the demo.
pub fn admin_panel() -> Panel {
    Panel::new("admin")
        .auth(Auth::custom(DemoAuth))
        .brand(Brand::new("Filament Demo"))
        .login_hint(login_hint())
        .notifications()
        .home::<Dashboard>()
        .page::<ShopDashboard>()
        .page::<HrDashboard>()
        // The first resource of a group fixes the group's place in the sidebar,
        // so the registration order is the demo's navigation order.
        .resource::<crate::shop::ProductResource>()
        .resource::<crate::hr::DepartmentResource>()
        .resource::<crate::hr::ProjectResource>()
        .resource::<PostResource>()
        .resource::<crate::shop::BrandResource>()
        .resource::<crate::shop::ProductCategoryResource>()
        .resource::<crate::shop::CustomerResource>()
        .resource::<crate::shop::OrderResource>()
        .resource::<crate::shop::OrderItemResource>()
        .resource::<crate::shop::OrderAddressResource>()
        .resource::<crate::shop::PaymentResource>()
        .resource::<crate::shop::AddressResource>()
        .resource::<crate::hr::EmployeeResource>()
        .resource::<crate::hr::LeaveRequestResource>()
        .resource::<crate::hr::ExpenseResource>()
        .resource::<crate::hr::ExpenseLineResource>()
        .resource::<crate::hr::TaskResource>()
        .resource::<crate::hr::TimesheetResource>()
        .resource::<PostCategoryResource>()
        .resource::<AuthorResource>()
        .resource::<CommentResource>()
        .uploads_dir(UPLOAD_URL_PREFIX, upload_dir())
}

/// The tenant panel: sign-up, a tenant switcher and the settings page.
pub fn app_panel() -> Panel {
    Panel::new("app")
        .auth(Auth::custom(DemoAuth).registration(DemoAuth))
        .brand(Brand::new("Filament Demo"))
        .login_hint(login_hint())
        .home::<Dashboard>()
        .page::<Settings>()
}

/// The tenant panel's settings page, which the Filament demo leaves empty.
pub struct Settings;

impl Page for Settings {
    fn declare() -> PageDef {
        PageDef::new().icon(crate::icons::SETTINGS)
    }

    async fn render(cx: &Cx) -> Result<impl View> {
        let team = membership(cx).map(|membership| membership.name.clone());
        Ok(view! {
            cx =>
            tablo::ui::page(
                tablo::ui::page_header(
                    tablo::ui::page_title("Settings")
                    tablo::ui::page_description(
                        "The Filament demo renders an empty page here."
                    )
                )
                tablo::ui::page_content(
                    tablo::ui::card(
                        tablo::ui::card_header(
                            tablo::ui::card_title("Team")
                            tablo::ui::card_description(
                                "The tenant this session acts for."
                            )
                        )
                        tablo::ui::card_content(
                            <p class="text-sm">
                                (team.unwrap_or_else(|| "No team selected.".to_string()))
                            </p>
                        )
                    )
                )
            )
        })
    }
}

/// The directory uploaded images are written to.
pub fn upload_dir() -> PathBuf {
    std::env::var_os("TABLO_DEMO_UPLOAD_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("target/demo-uploads"))
}

/// Mounts both panels on a router, with the asset bundle the build script wrote.
pub fn router(db: Db) -> Router {
    let bundle = AssetBundle::load().expect("load the asset bundle beside the executable");
    Router::builder()
        .discover()
        .app_context(db)
        .assets(bundle)
        .panel(admin_panel().shell_assets(tailwind::stylesheet!(), ALBERT_SANS))
        .expect("the admin panel mounts")
        .panel(app_panel().shell_assets(tailwind::stylesheet!(), ALBERT_SANS))
        .expect("the app panel mounts")
        .build()
}
