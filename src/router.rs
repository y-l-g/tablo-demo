//! The app's router: the panels, the demo's own sessions, the mail they send, the asset bundle they serve, and the
//! handles the welcome page borrows.

use tablo::RouterBuilderPanelExt;
use toasty::Db;
use topcoat::{
    asset::{AssetBundle, RouterBuilderAssetExt},
    cookie::RouterBuilderCookieExt,
    font::{Font, fontsource::fontsource_font},
    mail::{MailConfig, RouterBuilderMailExt},
    router::{Router, RouterBuilderDiscoverExt},
    session::{RouterBuilderSessionExt, SessionConfig},
    tailwind,
};

use crate::config::Config;
use crate::mailbox::Postbox;
use crate::panels::{admin_panel, app_panel, partners_panel};
use crate::partners::PartnerPanel;
use crate::welcome::{AdminPanel, AppPanel};

/// The theme's sans font, `Albert Sans`.
const ALBERT_SANS: Font = fontsource_font!(ALBERT_SANS, host: Asset);

/// Mounts the panels on a router, with the asset bundle the build script wrote, and keeps their
/// handles for the welcome page at `/`.
pub fn router(db: Db, config: &Config) -> Router {
    let bundle = AssetBundle::load().expect("load the asset bundle beside the executable");
    let builder = Router::builder()
        .discover()
        .with(db)
        .with(config.cookie_key.clone())
        .cookies()
        // The partner sign-in's, which the panels leave alone: theirs ride in a cookie of their own.
        .sessions(SessionConfig::default())
        .mail(
            MailConfig::builder()
                .transport(Postbox::new(config.smtp.as_ref()))
                .build(),
        )
        .assets(bundle)
        .panel(admin_panel(config).shell_assets(tailwind::stylesheet!(), ALBERT_SANS))
        .expect("the admin panel mounts")
        .panel(app_panel(config).shell_assets(tailwind::stylesheet!(), ALBERT_SANS))
        .expect("the app panel mounts")
        .panel(partners_panel().shell_assets(tailwind::stylesheet!(), ALBERT_SANS))
        .expect("the partner panel mounts");
    let admin = builder.panel_handle("admin").expect("mounted above");
    let app = builder.panel_handle("app").expect("mounted above");
    let partners = builder.panel_handle("partners").expect("mounted above");
    builder
        .with(AdminPanel(admin))
        .with(AppPanel(app))
        .with(PartnerPanel(partners))
        .build()
}
