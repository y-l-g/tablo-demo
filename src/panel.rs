//! The two panels and the router that serves them.

use std::path::PathBuf;

use tablo::{Auth, Brand, Panel, RouterBuilderPanelExt};
use toasty::Db;
use topcoat::{
    asset::{AssetBundle, RouterBuilderAssetExt},
    font::{Font, fontsource::fontsource_font},
    router::{Router, RouterBuilderDiscoverExt},
    tailwind,
};

use crate::{
    auth::DemoAuth,
    blog::{AuthorResource, CommentResource, PostCategoryResource, PostResource},
    dashboards::{Dashboard, HrDashboard, ShopDashboard},
};

/// The theme's sans font, which the Filament demo names `Albert Sans`.
const ALBERT_SANS: Font = fontsource_font!(ALBERT_SANS, host: Asset);

/// Where uploaded images are written.
pub const UPLOAD_URL_PREFIX: &str = "/uploads";

/// The login hint both panels show, with the demo account.
pub const LOGIN_HINT: &str = "admin@filamentphp.com / demo.Filament@2021!";

/// The admin panel: every resource of the demo.
pub fn admin_panel() -> Panel {
    Panel::new("admin")
        .auth(Auth::custom(DemoAuth))
        .brand(Brand::new("Filament Demo"))
        .login_hint(LOGIN_HINT)
        .home::<Dashboard>()
        .page::<ShopDashboard>()
        .page::<HrDashboard>()
        .resource::<PostResource>()
        .resource::<PostCategoryResource>()
        .resource::<AuthorResource>()
        .resource::<CommentResource>()
        .resource::<crate::shop::ProductResource>()
        .resource::<crate::shop::BrandResource>()
        .resource::<crate::shop::ProductCategoryResource>()
        .resource::<crate::shop::CustomerResource>()
        .resource::<crate::shop::OrderResource>()
        .resource::<crate::shop::OrderItemResource>()
        .resource::<crate::shop::PaymentResource>()
        .resource::<crate::shop::AddressResource>()
        .resource::<crate::hr::DepartmentResource>()
        .resource::<crate::hr::EmployeeResource>()
        .resource::<crate::hr::LeaveRequestResource>()
        .resource::<crate::hr::ExpenseResource>()
        .resource::<crate::hr::ExpenseLineResource>()
        .resource::<crate::hr::ProjectResource>()
        .resource::<crate::hr::TaskResource>()
        .resource::<crate::hr::TimesheetResource>()
        .uploads_dir(UPLOAD_URL_PREFIX, upload_dir())
}

/// The tenant panel: sign-up, a tenant switcher and the settings page.
pub fn app_panel() -> Panel {
    Panel::new("app")
        .auth(Auth::custom(DemoAuth).registration(DemoAuth))
        .brand(Brand::new("Filament Demo"))
        .login_hint(LOGIN_HINT)
        .home::<Dashboard>()
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
