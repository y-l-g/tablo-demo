//! The `app` panel: where an account signs up, acts for a team, and edits its profile.

use tablo::{
    Auth, AuthMail, EmailVerification, Page, PageDef, Panel, PasswordReset, TenantRegistration,
    TwoFactor, auth::membership,
};
use topcoat::{
    Result,
    context::Cx,
    icon::iconify::iconify_icon,
    view::{View, view},
};

use crate::accounts::{DemoAuth, DemoResetter, DemoTenantRegistrar, DemoVerifier, SignedIn};
use crate::config::Config;
use crate::home::Dashboard;

/// The tenant panel: sign-up, a tenant switcher and the settings page.
pub fn app_panel(config: &Config) -> Panel {
    super::shared(Panel::new("app"))
        .auth(app_auth(config))
        .home::<Dashboard>()
        .page::<Settings>()
}

/// The app panel's auth: the demo's own sign-in, with sign-up, password reset,
/// email verification, the profile, two-factor sign-in, and creating and editing
/// a team.
fn app_auth(config: &Config) -> Auth<SignedIn> {
    Auth::custom(DemoAuth)
        .registration(DemoAuth)
        .mail(AuthMail::new(
            config.sender.clone(),
            config.base_url.clone(),
        ))
        .password_reset(PasswordReset::new(DemoResetter))
        .email_verification(EmailVerification::new(DemoVerifier))
        .profile(DemoAuth)
        // An authenticator app, or codes mailed to the account's address.
        .two_factor(TwoFactor::new().email_codes())
        .tenant_registration(TenantRegistration::new(DemoTenantRegistrar).required())
        .tenant_profile(DemoTenantRegistrar)
}

/// The tenant panel's settings page, which shows only the team.
pub struct Settings;

impl Page for Settings {
    fn declare() -> PageDef {
        PageDef::new().icon(iconify_icon!("lucide:settings"))
    }

    async fn render(cx: &Cx) -> Result<impl View> {
        let team = membership(cx).map(|membership| membership.name.clone());
        Ok(view! {
            cx =>
            tablo::ui::page(
                tablo::ui::page_header(
                    tablo::ui::page_title((tablo::t!(cx, "page-settings")))
                    tablo::ui::page_description(
                        (tablo::t!(cx, "copy-settings-description"))
                    )
                )
                tablo::ui::page_content(
                    tablo::ui::card(
                        tablo::ui::card_header(
                            tablo::ui::card_title((tablo::t!(cx, "label-team")))
                            tablo::ui::card_description(
                                (tablo::t!(cx, "copy-team-description"))
                            )
                        )
                        tablo::ui::card_content(
                            <p class="text-sm">
                                (team.unwrap_or_else(|| tablo::t!(cx, "copy-no-team")))
                            </p>
                        )
                    )
                )
            )
        })
    }
}
