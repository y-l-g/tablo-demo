//! The three panels: `admin`, where staff run the business, `app`, where customers sign up, and
//! `partners`, which the host app signs in.
//!
//! All wear the demo's brand in Ledger, the theme of Tablo's site, and speak English and French; the first two
//! show the same login hint.

mod admin;
mod app;
mod appearance;
mod brand;
mod partners;

pub use admin::admin_panel;
pub use app::app_panel;
pub use brand::LOGO_URL;
pub use partners::{PartnerHome, partners_panel};

use tablo::{Panel, Theme};
use topcoat::l10n::locale;

use crate::accounts::{ADMIN_EMAIL, ADMIN_PASSWORD};

/// What every panel shares: the brand in Ledger, which the visitor may swap from the
/// appearance drawer, a login hint naming the demo administrator's account, and English and
/// French, the browser's `Accept-Language` choosing, with the demo's own words from `locales/`.
fn shared(panel: Panel) -> Panel {
    panel
        .brand(brand::brand(Theme::Ledger).look(appearance::visitor_look))
        .banner::<appearance::Appearance>()
        .login_hint(format!("{ADMIN_EMAIL} / {ADMIN_PASSWORD}"))
        .locales([locale!("en"), locale!("fr")])
        .translations(locale!("en"), include_str!("../../locales/en.ftl"))
        .translations(locale!("fr"), include_str!("../../locales/fr.ftl"))
}
