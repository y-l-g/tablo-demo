//! The two panels and the router that serves them.

use std::path::PathBuf;

use tablo::{
    Auth, AuthMail, Brand, EmailVerification, Page, PageDef, Panel, PasswordReset,
    RouterBuilderPanelExt, TenantRegistration, TwoFactor, auth::membership,
};
use toasty::Db;
use topcoat::{
    Result,
    asset::{AssetBundle, RouterBuilderAssetExt},
    context::Cx,
    cookie::Key,
    font::{Font, fontsource::fontsource_font},
    l10n::locale,
    mail::{MailConfig, Mailbox as Sender, RouterBuilderMailExt},
    router::{Router, RouterBuilderDiscoverExt},
    tailwind,
    view::{View, view},
};

use crate::{
    auth::{DemoAuth, DemoResetter, DemoTenantRegistrar, DemoVerifier, SignedIn},
    blog::{AuthorResource, CommentResource, PostCategoryResource, PostResource},
    dashboards::{Dashboard, HrDashboard, ShopDashboard},
    mailbox::Postbox,
    seed,
};

/// The theme's sans font, which the Filament demo names `Albert Sans`.
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
    format!("{} / {}", seed::ADMIN_EMAIL, seed::ADMIN_PASSWORD)
}

/// The admin panel: every resource of the demo.
pub fn admin_panel() -> Panel {
    localized(Panel::new("admin"))
        .auth(Auth::custom(DemoAuth))
        .brand(Brand::new("Filament Demo"))
        .login_hint(login_hint())
        .notifications()
        .home::<Dashboard>()
        .page::<ShopDashboard>()
        .page::<HrDashboard>()
        .page::<crate::audit::AuditTrail>()
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
    localized(Panel::new("app"))
        .auth(app_auth())
        .brand(Brand::new("Filament Demo"))
        .login_hint(login_hint())
        .home::<Dashboard>()
        .page::<Settings>()
}

/// The app panel's auth: the demo's own sign-in, with every page the Filament
/// demo's app panel has: sign-up, password reset, email verification, the
/// profile, two-factor sign-in, and creating and editing a team.
fn app_auth() -> Auth<SignedIn> {
    Auth::custom(DemoAuth)
        .registration(DemoAuth)
        .mail(AuthMail::new(sender(), base_url()))
        .password_reset(PasswordReset::new(DemoResetter))
        .email_verification(EmailVerification::new(DemoVerifier))
        .profile(DemoAuth)
        // An authenticator app, or codes mailed to the account's address.
        .two_factor(TwoFactor::new().email_codes())
        .tenant_registration(TenantRegistration::new(DemoTenantRegistrar).required())
        .tenant_profile(DemoTenantRegistrar)
}

/// The address the app panel's mail comes from: `TABLO_DEMO_MAIL_FROM`, which
/// a mail provider has verified, or a placeholder.
fn sender() -> Sender {
    let address = std::env::var("TABLO_DEMO_MAIL_FROM")
        .unwrap_or_else(|_| "no-reply@demo.example.com".to_string());
    Sender::named("Filament Demo", &address).expect("TABLO_DEMO_MAIL_FROM is an email address")
}

/// Offers English and French, the browser's `Accept-Language` choosing, with
/// the demo's own words from `locales/`.
fn localized(panel: Panel) -> Panel {
    panel
        .locales([locale!("en"), locale!("fr")])
        .translations(locale!("en"), include_str!("../locales/en.ftl"))
        .translations(locale!("fr"), include_str!("../locales/fr.ftl"))
}

/// The URL the app answers on, which every mailed link starts with.
fn base_url() -> String {
    std::env::var("TABLO_DEMO_BASE_URL").unwrap_or_else(|_| "http://127.0.0.1:3000".to_string())
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

/// The directory uploaded images are written to.
pub fn upload_dir() -> PathBuf {
    std::env::var_os("TABLO_DEMO_UPLOAD_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("target/demo-uploads"))
}

/// The router's cookie key, which signs the links to uploaded files. `TABLO_DEMO_COOKIE_KEY` holds
/// its 64 bytes as 128 hex characters (`openssl rand -hex 64`); without it each start draws a
/// key, so the upload links a page showed before a restart stop working after it.
fn cookie_key() -> Key {
    let Ok(secret) = std::env::var("TABLO_DEMO_COOKIE_KEY") else {
        return Key::generate();
    };
    let bytes: Option<Vec<u8>> = (0..secret.len())
        .step_by(2)
        .map(|at| {
            secret
                .get(at..at + 2)
                .and_then(|pair| u8::from_str_radix(pair, 16).ok())
        })
        .collect();
    bytes
        .and_then(|bytes| Key::try_from(bytes.as_slice()).ok())
        .expect("TABLO_DEMO_COOKIE_KEY holds 128 hex characters")
}

/// Mounts both panels on a router, with the asset bundle the build script wrote.
pub fn router(db: Db) -> Router {
    let bundle = AssetBundle::load().expect("load the asset bundle beside the executable");
    Router::builder()
        .discover()
        .with(db)
        .with(cookie_key())
        // The app panel's mail goes out through the SMTP server the environment names.
        .mail(MailConfig::builder().transport(Postbox::from_env()).build())
        .assets(bundle)
        .panel(admin_panel().shell_assets(tailwind::stylesheet!(), ALBERT_SANS))
        .expect("the admin panel mounts")
        .panel(app_panel().shell_assets(tailwind::stylesheet!(), ALBERT_SANS))
        .expect("the app panel mounts")
        .build()
}
