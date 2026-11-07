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

use tablo::Panel;
use topcoat::l10n::locale;

use crate::accounts::{ADMIN_EMAIL, ADMIN_PASSWORD};

/// What both panels share: the brand, a login hint naming the demo administrator's account, and
/// English and French, the browser's `Accept-Language` choosing, with the demo's own words from
/// `locales/`.
fn shared(panel: Panel) -> Panel {
    panel
        .brand(brand::brand())
        .login_hint(format!("{ADMIN_EMAIL} / {ADMIN_PASSWORD}"))
        .locales([locale!("en"), locale!("fr")])
        .translations(locale!("en"), include_str!("../../locales/en.ftl"))
        .translations(locale!("fr"), include_str!("../../locales/fr.ftl"))
}
