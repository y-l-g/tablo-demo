//! The two panels: `admin`, where staff run the business, and `app`, where customers sign up.
//!
//! Both wear the demo's brand, show the same login hint, and speak English and French.

mod admin;
mod app;
mod brand;

pub use admin::admin_panel;
pub use app::{Settings, app_panel};

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
