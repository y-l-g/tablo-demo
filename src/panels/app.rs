//! The `app` panel: where an account signs up, acts for a team, and edits its profile.

use tablo::{
    Area, Auth, AuthMail, EmailVerification, Panel, PasswordReset, TenantRegistration, Theme,
    TwoFactor,
};
use topcoat::icon::iconify::iconify_icon;

use crate::accounts::{DemoSignUp, DemoTenantRegistrar, SignedIn, password_auth};
use crate::config::Config;
use crate::home::Dashboard;

/// The tenant panel: sign-up, a tenant switcher and a settings area.
pub fn app_panel(config: &Config) -> Panel {
    super::shared(Panel::new("app"), Theme::Iris)
        .auth(app_auth(config))
        .home::<Dashboard>()
        .area(settings())
}

/// The account's and the team's settings under `/app/settings`, with a sidebar of their own: the
/// panel's sidebar lists the area as one entry, so pages a user opens rarely stay out of its way.
fn settings() -> Area {
    Area::new("settings")
        .label(tablo::text!("page-settings"))
        .icon(iconify_icon!("lucide:settings"))
        .group(tablo::text!("settings-account"), |group| {
            group
                .profile()
                .two_factor_with(|def| def.label(tablo::text!("settings-security")))
        })
        // Headed by the team the session acts for, and shown only while it acts for one.
        .tenant_group(|group| {
            group.tenant_profile_with(|def| def.slug("team").label(tablo::text!("settings-team")))
        })
}

/// The app panel's auth: the demo's own sign-in, with sign-up, password reset,
/// email verification, the profile, two-factor sign-in, and creating and editing
/// a team.
fn app_auth(config: &Config) -> Auth<SignedIn> {
    Auth::custom(password_auth())
        .registration(DemoSignUp)
        .mail(AuthMail::new(
            config.sender.clone(),
            config.base_url.clone(),
        ))
        .password_reset(PasswordReset::new(password_auth()))
        .email_verification(EmailVerification::new(password_auth()))
        .profile(password_auth())
        // An authenticator app, or codes mailed to the account's address.
        .two_factor(TwoFactor::new().email_codes())
        .tenant_registration(TenantRegistration::new(DemoTenantRegistrar).required())
        .tenant_profile(DemoTenantRegistrar)
}
