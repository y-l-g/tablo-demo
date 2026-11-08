//! The three panels: `admin`, where staff run the business, `app`, where customers sign up, and
//! `partners`, which the host app signs in.
//!
//! All wear the demo's brand and speak English and French; the first two show the same login hint.

mod admin;
mod app;
mod brand;
mod partners;

pub use admin::admin_panel;
pub use app::app_panel;
pub use partners::{PartnerHome, partners_panel};

use tablo::{Color, Environment, Panel};
use topcoat::l10n::locale;

use crate::accounts::{ADMIN_EMAIL, ADMIN_PASSWORD};

/// What every panel shares: the brand, the "Demo" ribbon, a login hint naming the demo
/// administrator's account, and English and French, the browser's `Accept-Language` choosing,
/// with the demo's own words from `locales/`.
fn shared(panel: Panel) -> Panel {
    panel
        .brand(brand::brand())
        // Every page, sign-in included, says its data is the demo's, and so does the tab title.
        .environment(Environment::new("Demo").color(Color::Info))
        .login_hint(format!("{ADMIN_EMAIL} / {ADMIN_PASSWORD}"))
        .locales([locale!("en"), locale!("fr")])
        .translations(locale!("en"), include_str!("../../locales/en.ftl"))
        .translations(locale!("fr"), include_str!("../../locales/fr.ftl"))
}
