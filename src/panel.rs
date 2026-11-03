//! The two panels and the router that serves them.

use tablo::{
    Auth, AuthMail, Brand, EmailVerification, Page, PageDef, Panel, PasswordReset,
    RouterBuilderPanelExt, TenantRegistration, TwoFactor, auth::membership,
};
use toasty::Db;
use topcoat::{
    Result,
    asset::{AssetBundle, RouterBuilderAssetExt},
    context::Cx,
    font::{Font, fontsource::fontsource_font},
    l10n::locale,
    mail::{MailConfig, RouterBuilderMailExt},
    router::{Router, RouterBuilderDiscoverExt},
    tailwind,
    view::{View, view},
};

use crate::{
    auth::{self, DemoAuth, DemoResetter, DemoTenantRegistrar, DemoVerifier, SignedIn},
    blog::{AuthorResource, CommentResource, PostCategoryResource, PostResource, TagResource},
    config::Config,
    dashboards::{Dashboard, HrDashboard, ShopDashboard},
    mailbox::Postbox,
    welcome::{AdminPanel, AppPanel},
};

/// The theme's sans font, `Albert Sans`.
const ALBERT_SANS: Font = fontsource_font!(ALBERT_SANS, host: Asset);

/// The URL prefix uploaded images are served under.
pub const UPLOAD_URL_PREFIX: &str = "/uploads";

/// The image types a product or a post takes, and the most bytes an image may hold.
pub const IMAGE_EXTENSIONS: [&str; 4] = ["jpg", "jpeg", "png", "webp"];
pub const IMAGE_MAX_BYTES: usize = 2 * 1024 * 1024;

/// The file types an expense's receipt takes, and the most bytes a receipt may hold.
pub const RECEIPT_EXTENSIONS: [&str; 5] = ["jpg", "jpeg", "png", "webp", "pdf"];
pub const RECEIPT_MAX_BYTES: usize = 5 * 1024 * 1024;

/// The login hint both panels show, with the demo account the seed creates.
pub fn login_hint() -> String {
    format!("{} / {}", auth::ADMIN_EMAIL, auth::ADMIN_PASSWORD)
}

/// The admin panel: every resource of the demo.
pub fn admin_panel(config: &Config) -> Panel {
    localized(Panel::new("admin"))
        .auth(Auth::custom(DemoAuth))
        .brand(brand())
        .login_hint(login_hint())
        .notifications()
        // Every write of every resource below leaves a row in the audit trail.
        .before_commit::<crate::audit::Audit>()
        .home::<Dashboard>()
        .page::<ShopDashboard>()
        .page::<HrDashboard>()
        // The first resource of a group fixes the group's place in the sidebar,
        // so the registration order is the demo's navigation order.
        .resource::<crate::audit::AuditTrail>()
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
        .resource::<TagResource>()
        .resource::<AuthorResource>()
        .resource::<CommentResource>()
        .uploads_dir(UPLOAD_URL_PREFIX, config.upload_dir.clone())
}

/// The demo's brand on both panels: its name, and a mark that turns light on a dark page.
fn brand() -> Brand {
    Brand::new("Tablo Demo")
        .logo(crate::brand::LOGO_URL)
        .dark_logo(crate::brand::DARK_LOGO_URL)
        .favicon(crate::brand::LOGO_URL)
}

/// The tenant panel: sign-up, a tenant switcher and the settings page.
pub fn app_panel(config: &Config) -> Panel {
    localized(Panel::new("app"))
        .auth(app_auth(config))
        .brand(brand())
        .login_hint(login_hint())
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

/// Offers English and French, the browser's `Accept-Language` choosing, with
/// the demo's own words from `locales/`.
fn localized(panel: Panel) -> Panel {
    panel
        .locales([locale!("en"), locale!("fr")])
        .translations(locale!("en"), include_str!("../locales/en.ftl"))
        .translations(locale!("fr"), include_str!("../locales/fr.ftl"))
}

/// The tenant panel's settings page, which shows only the team.
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

/// Mounts both panels on a router, with the asset bundle the build script wrote, and keeps their
/// handles for the welcome page at `/`.
pub fn router(db: Db, config: &Config) -> Router {
    let bundle = AssetBundle::load().expect("load the asset bundle beside the executable");
    let builder = Router::builder()
        .discover()
        .with(db)
        .with(config.cookie_key.clone())
        .mail(
            MailConfig::builder()
                .transport(Postbox::new(config.smtp.as_ref()))
                .build(),
        )
        .assets(bundle)
        .panel(admin_panel(config).shell_assets(tailwind::stylesheet!(), ALBERT_SANS))
        .expect("the admin panel mounts")
        .panel(app_panel(config).shell_assets(tailwind::stylesheet!(), ALBERT_SANS))
        .expect("the app panel mounts");
    let admin = builder.panel_handle("admin").expect("mounted above");
    let app = builder.panel_handle("app").expect("mounted above");
    builder.with(AdminPanel(admin)).with(AppPanel(app)).build()
}
