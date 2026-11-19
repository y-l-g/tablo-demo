//! The public page at `/`: the three panels the demo mounts, and an account to sign in to each.
//!
//! It wears Ledger, the identity Tablo's site shares with the demo and the SaaS template
//! (`ledger.css`), rather than a panel theme: it is the demo's front page, not one of its panels.

use serde::Deserialize;
use tablo::PanelHandle;
use topcoat::{
    Result,
    context::{Cx, context},
    cookie::{Cookie, Cookies, SameSite, cookies},
    font::{Font, fontsource::fontsource_font},
    icon::{IconData, icon, iconify::iconify_icon},
    router::{
        HeaderMap, HeaderValue,
        content::Form,
        error::{SeeOther, forbidden, see_other},
        header::SET_COOKIE,
        page, route,
    },
    view::{View, component, view},
};

use crate::access::{Access, Domain};
use crate::accounts::{self, StaffRole};
use crate::panels::LOGO_URL;
use crate::partners::PartnerPanel;

/// The mounted admin panel, whose document, translations and URLs the page borrows.
pub struct AdminPanel(pub PanelHandle);

/// The mounted app panel, which the page links to.
pub struct AppPanel(pub PanelHandle);

const SITE: &str = "https://y-l.fr/tablo/";
const GUIDE: &str = "https://y-l.fr/tablo/nightly/guide/";
const API: &str = "https://y-l.fr/tablo/nightly/api/tablo/";
const ATLAS: &str = "https://y-l.fr/tablo/nightly/atlas/";
const EXAMPLES: &str = "https://y-l.fr/tablo/examples/";
const SAAS: &str = "https://saas.tablo.y-l.fr";
const SOURCE: &str = "https://github.com/y-l-g/tablo-demo";
const TABLO: &str = "https://github.com/y-l-g/tablo";
const TOPCOAT: &str = "https://github.com/tokio-rs/topcoat";
const TOASTY: &str = "https://github.com/tokio-rs/toasty";

/// Ledger's two faces: a grotesque for text, a mono for labels and code.
const SCHIBSTED_GROTESK: Font = fontsource_font!(SCHIBSTED_GROTESK, weight: [400, 600, 700, 800, 900], style: Normal, host: Asset);
const MARTIAN_MONO: Font =
    fontsource_font!(MARTIAN_MONO, weight: [400, 500, 600], style: Normal, host: Asset);

const ARROW: IconData = iconify_icon!("lucide:arrow-right");

/// What the demo shows, a cell each: its title and the line under it.
const FEATURES: [(&str, &str); 8] = [
    ("home-feature-resources", "home-feature-resources-body"),
    ("home-feature-tables", "home-feature-tables-body"),
    ("home-feature-forms", "home-feature-forms-body"),
    ("home-feature-actions", "home-feature-actions-body"),
    ("home-feature-dashboards", "home-feature-dashboards-body"),
    ("home-feature-auth", "home-feature-auth-body"),
    ("home-feature-search", "home-feature-search-body"),
    ("home-feature-i18n", "home-feature-i18n-body"),
];

/// The panels' locales, each named in its own language.
const LANGUAGES: [(&str, &str); 2] = [("en", "English"), ("fr", "Français")];

/// The cookies Tablo reads a visitor's language from, and `theme.js` their theme.
///
/// Tablo writes the language one under each panel's prefix, which `/` never receives, so the
/// page keeps its own copy at `/` and writes the panels' along with it.
const LOCALE_COOKIE: &str = "tablo_locale";
const THEME_COOKIE: &str = "tablo_theme";

/// `GET /` — the welcome page, rendered in the admin panel's document and catalog.
#[page("/")]
async fn welcome(cx: &Cx) -> Result<impl View> {
    let AdminPanel(admin) = context::<AdminPanel>(cx);
    let AppPanel(app) = context::<AppPanel>(cx);
    // Entering the admin panel gives the page its catalog and locale.
    let entered = admin.enter(cx);
    let t = |id: &'static str| tablo::t!(&entered, id);
    let admin_url = admin.url().unwrap_or_else(|| "/admin".to_string());
    let app_url = app.url().unwrap_or_else(|| "/app".to_string());
    let PartnerPanel(partners) = context::<PartnerPanel>(cx);
    let partners_url = partners.url().unwrap_or_else(|| "/partners".to_string());

    // The buttons post the panels' login forms, which check this token against its cookie.
    let csrf = tablo::csrf::ensure_token(cx);
    let role_name = |role: StaffRole| match role {
        StaffRole::Administrator => "home-role-administrator",
        StaffRole::ShopManager => "home-role-shop-manager",
        StaffRole::HrManager => "home-role-hr-manager",
        StaffRole::Viewer => "home-role-viewer",
    };
    // The grid of what each role reaches, read from the policies' own rules.
    let grant = |level: &'static str| {
        let label = match level {
            "full" => "home-grant-full",
            "read" => "home-grant-read",
            "yes" => "home-grant-yes",
            _ => "home-grant-none",
        };
        Grant {
            level,
            label: t(label),
        }
    };
    let grants = |role: StaffRole| {
        let mut grants: Vec<Grant> = Domain::ALL
            .into_iter()
            .map(|domain| match role.access(domain) {
                Access::Full => grant("full"),
                Access::Read => grant("read"),
                Access::None => grant("none"),
            })
            .collect();
        grants.push(grant(if role.sees_pay() { "yes" } else { "none" }));
        grants.push(grant(if role.administers() { "yes" } else { "none" }));
        grants
    };
    let mut columns = Domain::ALL.map(|domain| t(domain.label())).to_vec();
    columns.push(t("home-col-pay"));
    columns.push(t("home-col-audit"));
    let persona = |name, email, role| Persona {
        name,
        email,
        role: t(role_name(role)),
        grants: grants(role),
    };
    let legend = ["full", "read", "none"].map(grant);
    let mut staff = vec![persona(
        accounts::ADMIN_NAME,
        accounts::ADMIN_EMAIL,
        StaffRole::Administrator,
    )];
    staff.extend(
        accounts::STAFF
            .iter()
            .map(|account| persona(account.name, account.email, account.role)),
    );
    let member = Persona {
        name: accounts::ADMIN_NAME,
        email: accounts::ADMIN_EMAIL,
        role: t("home-role-member"),
        grants: Vec::new(),
    };
    let features = FEATURES
        .iter()
        .enumerate()
        .map(|(n, (title, body))| (format!("{:02}", n + 1), t(title), t(body)))
        .collect::<Vec<_>>();

    let locale = topcoat::l10n::try_locale(&entered).map(ToString::to_string);
    let languages = LANGUAGES.map(|(tag, name)| (tag, name, locale.as_deref() == Some(tag)));
    // `theme.js` checks the theme picked on click; the page checks the stored one. The panels
    // default to light.
    let stored = cookies(cx).get(THEME_COOKIE);
    let stored = stored.as_ref().map_or("light", |cookie| cookie.value());
    let themes = [
        ("light", tablo::ui::icons::SUN, t("tablo-shell-theme-light")),
        ("dark", tablo::ui::icons::MOON, t("tablo-shell-theme-dark")),
        (
            "system",
            tablo::ui::icons::MONITOR,
            t("tablo-shell-theme-system"),
        ),
    ]
    .map(|(name, data, label)| (name, data, label, name == stored));

    let title = t("home-document-title");
    // The document is the outer request's, which knows no locale, so the page states its own.
    let lang = t("home-lang");
    // The view renders after this function returns, so it translates through a clone it owns.
    let t = {
        let cx = entered.clone();
        move |id: &'static str| tablo::t!(&cx, id)
    };
    let admin_login = format!("{admin_url}/login");
    let app_login = format!("{app_url}/login");
    let sign_up = format!("{app_url}/register");
    let body = view! {
        topcoat::font::link(font: SCHIBSTED_GROTESK)
        topcoat::font::link(font: MARTIAN_MONO)
        <div lang=(lang) class="ld">
            <div class="ld-sheet">
                <header class="ld-mast">
                    <a class="ld-brand" href="/">
                        <img src=(LOGO_URL) width="32" height="32" alt="">
                        "Tablo"
                        <small>"demo"</small>
                    </a>
                    <nav class="ld-nav" aria-label=(t("home-nav"))>
                        <a class="ld-hide-sm" href=(SITE)>(t("home-site"))</a>
                        <a class="ld-hide-sm" href=(GUIDE)>(t("home-guide"))</a>
                        <div class="ld-seg" role="group" aria-label=(t("tablo-shell-language"))>
                            for (tag, name, current) in languages {
                                <form method="post" action="/locale">
                                    <input type="hidden" name=(tablo::csrf::FIELD_NAME) value=(tablo::csrf::mask(&csrf))>
                                    <input type="hidden" name="locale" value=(tag)>
                                    <button type="submit" title=(name) lang=(tag) aria-current=(current.then_some("true"))>
                                        (tag)
                                    </button>
                                </form>
                            }
                        </div>
                        <div class="ld-seg" role="radiogroup" aria-label=(t("tablo-shell-theme"))>
                            for (name, data, label, checked) in themes {
                                <button
                                    type="button"
                                    role="radio"
                                    data-theme-set=(name)
                                    aria-checked=(if checked { "true" } else { "false" })
                                    aria-label=(label.clone())
                                    title=(label)
                                >
                                    icon(data: data)
                                </button>
                            }
                        </div>
                        sign_in(
                            action: admin_login.clone(),
                            email: accounts::ADMIN_EMAIL,
                            csrf: csrf.clone(),
                            label: t("home-cta-admin"),
                            class: "ld-nav-cta ld-hide-sm",
                        )
                    </nav>
                </header>

                <main>
                    <section class="ld-row" aria-labelledby="home-title">
                        <div class="ld-num"><b>"01"</b></div>
                        <div class="ld-body">
                            <div class="ld-hero">
                                <p class="ld-label">(t("home-kicker"))</p>
                                <h1 class="ld-h1" id="home-title">
                                    (t("home-title-1"))
                                    <br>
                                    <span class="ld-red">(t("home-title-2"))</span>
                                </h1>
                            </div>
                            <div class="ld-hero-sub">
                                <div>
                                    <p class="ld-lede">(t("home-lead"))</p>
                                    <p class="ld-text">
                                        (t("home-password-note"))
                                        " "
                                        <code>(accounts::ADMIN_PASSWORD)</code>
                                        "."
                                    </p>
                                </div>
                                <div>
                                    <div class="ld-actions">
                                        sign_in(
                                            action: admin_login.clone(),
                                            email: accounts::ADMIN_EMAIL,
                                            csrf: csrf.clone(),
                                            label: t("home-cta-admin"),
                                            class: "ld-btn ld-btn-primary",
                                        )
                                        <a class="ld-btn" href=(GUIDE)>(t("home-cta-guide"))</a>
                                    </div>
                                    <p class="ld-meta">
                                        <span>(t("home-meta-panels"))</span>
                                        <span>(t("home-meta-database"))</span>
                                        <span>(t("home-meta-locales"))</span>
                                        <a href=(SOURCE)>"github.com/y-l-g/tablo-demo"</a>
                                    </p>
                                </div>
                            </div>
                        </div>
                    </section>

                    <section class="ld-row" aria-labelledby="home-admin">
                        <div class="ld-num">"02"</div>
                        <div class="ld-body">
                            <div class="ld-head">
                                <span id="home-admin">(t("home-admin-title")) " " <i>(admin_url.clone())</i></span>
                                <a href=(admin_login.clone())>(t("home-sign-in-page"))</a>
                            </div>
                            <div class="ld-pad-sm ld-split">
                                <p class="ld-text">(t("home-admin-body"))</p>
                                <p class="ld-legend" aria-hidden="true">
                                    for Grant { level, label } in legend {
                                        <span class="ld-grant" data-level=(level)>(label)</span>
                                    }
                                </p>
                            </div>
                            <table class="ld-table ld-table-stack">
                                <thead>
                                    <tr>
                                        <th scope="col">(t("home-col-role"))</th>
                                        for column in columns.clone() {
                                            <th scope="col" class="ld-grants">(column)</th>
                                        }
                                        <th scope="col"><span class="ld-hide">(t("home-col-sign-in"))</span></th>
                                    </tr>
                                </thead>
                                <tbody>
                                    for persona in staff {
                                        <tr>
                                            <td>who(role: persona.role.clone(), name: persona.name, email: persona.email)</td>
                                            for (column, Grant { level, label }) in columns.clone().into_iter().zip(persona.grants) {
                                                <td class="ld-grants" data-th=(column)>
                                                    <span class="ld-grant" data-level=(level)>(label)</span>
                                                </td>
                                            }
                                            <td class="ld-go">
                                                sign_in(
                                                    action: admin_login.clone(),
                                                    email: persona.email,
                                                    csrf: csrf.clone(),
                                                    label: t("home-sign-in"),
                                                    class: "ld-btn",
                                                )
                                            </td>
                                        </tr>
                                    }
                                </tbody>
                            </table>
                        </div>
                    </section>

                    <section class="ld-row" aria-labelledby="home-more">
                        <div class="ld-num">"03"</div>
                        <div class="ld-body">
                            <div class="ld-head">
                                <span id="home-more">(t("home-more-title"))</span>
                            </div>
                            <div class="ld-cells ld-cells-2">
                                <div class="ld-cell">
                                    <p class="ld-label">(app_url.clone())</p>
                                    <h2 class="ld-h3">(t("home-app-title"))</h2>
                                    <p class="ld-text">(t("home-app-body"))</p>
                                    <div class="ld-who-row">
                                        who(role: member.role.clone(), name: member.name, email: member.email)
                                        sign_in(
                                            action: app_login,
                                            email: member.email,
                                            csrf: csrf.clone(),
                                            label: t("home-sign-in"),
                                            class: "ld-btn",
                                        )
                                    </div>
                                    <p><a class="ld-link" href=(sign_up)>(t("home-sign-up"))</a></p>
                                </div>
                                <div class="ld-cell">
                                    <p class="ld-label">(partners_url.clone())</p>
                                    <h2 class="ld-h3">(t("home-partners-title"))</h2>
                                    <p class="ld-text">(t("home-partners-body"))</p>
                                    <p><a class="ld-link" href=(partners_url)>(t("home-partners-open"))</a></p>
                                </div>
                            </div>
                        </div>
                    </section>

                    <section class="ld-row" aria-labelledby="home-features">
                        <div class="ld-num">"04"</div>
                        <div class="ld-body">
                            <div class="ld-head">
                                <span id="home-features">(t("home-features-title"))</span>
                                <span>(t("home-features-note"))</span>
                            </div>
                            <div class="ld-cells ld-cells-4">
                                for (n, title, body) in features {
                                    <div class="ld-cell">
                                        <p class="ld-label"><span class="ld-red">(n)</span></p>
                                        <h3 class="ld-h3">(title)</h3>
                                        <p class="ld-text">(body)</p>
                                    </div>
                                }
                            </div>
                        </div>
                    </section>
                </main>

                <footer class="ld-foot">
                    <div class="ld-cells ld-cells-3">
                        <div class="ld-cell">
                            <h4>"Tablo"</h4>
                            <ul>
                                <li><a href=(SITE)>(t("home-site"))</a></li>
                                <li><a href=(GUIDE)>(t("home-guide"))</a></li>
                                <li><a href=(API)>(t("home-api"))</a></li>
                                <li><a href=(ATLAS)>"Atlas"</a></li>
                            </ul>
                        </div>
                        <div class="ld-cell">
                            <h4>(t("home-foot-source"))</h4>
                            <ul>
                                <li><a href=(SOURCE)>(t("home-this-demo")) <small>"tablo-demo"</small></a></li>
                                <li><a href=(TABLO)>"Tablo" <small>"y-l-g"</small></a></li>
                                <li><a href=(TOPCOAT)>"Topcoat" <small>"tokio-rs"</small></a></li>
                                <li><a href=(TOASTY)>"Toasty" <small>"tokio-rs"</small></a></li>
                            </ul>
                        </div>
                        <div class="ld-cell">
                            <h4>(t("home-foot-more"))</h4>
                            <ul>
                                <li><a href=(SAAS)>(t("home-saas"))</a></li>
                                <li><a href=(EXAMPLES)>(t("home-examples"))</a></li>
                            </ul>
                        </div>
                    </div>
                    <div class="ld-colophon">
                        <span>(t("home-built-with"))</span>
                        <span>"MIT"</span>
                    </div>
                </footer>
            </div>
        </div>
    };
    admin.document(cx, title, body).await
}

/// The language picked from the page's switch.
#[derive(Deserialize)]
struct LocaleForm {
    csrf_token: String,
    locale: String,
}

/// `POST /locale` — read the page and both panels in another language, then land back on `/`.
#[route(POST "/locale")]
async fn pick_locale(cx: &Cx, Form(form): Form<LocaleForm>) -> Result<(HeaderMap, SeeOther)> {
    tablo::csrf::verify(cx, Some(&form.csrf_token))?;
    if !LANGUAGES.iter().any(|(tag, _)| *tag == form.locale) {
        return Err(forbidden().into());
    }
    let AdminPanel(admin) = context::<AdminPanel>(cx);
    let AppPanel(app) = context::<AppPanel>(cx);
    let PartnerPanel(partners) = context::<PartnerPanel>(cx);
    // The panels' own copies too, which outrank this one under their prefixes. The cookie jar
    // keeps one cookie per name, so they go out as headers of their own.
    let paths = ["/".to_string()]
        .into_iter()
        .chain(admin.url())
        .chain(app.url())
        .chain(partners.url());
    let mut headers = HeaderMap::new();
    for path in paths {
        let cookie = Cookie::build((LOCALE_COOKIE, form.locale.clone()))
            .path(path)
            .http_only(true)
            .secure(true)
            .same_site(SameSite::Lax)
            .max_age(topcoat::cookie::time::Duration::days(365))
            .build();
        headers.append(SET_COOKIE, HeaderValue::try_from(cookie.to_string())?);
    }
    Ok((headers, see_other("/")))
}

/// One account a visitor may continue as: who it is, and what its role reaches.
struct Persona {
    name: &'static str,
    email: &'static str,
    role: String,
    grants: Vec<Grant>,
}

/// One cell of the roles grid: how far a role reaches, as `ledger.css` marks it, and its word.
struct Grant {
    level: &'static str,
    label: String,
}

/// The initials of `name`, such as `DU` for `Demo User`.
fn initials(name: &str) -> String {
    name.split_whitespace()
        .filter_map(|word| word.chars().next())
        .take(2)
        .collect()
}

/// An account's initials, role and name.
#[component]
async fn who(cx: &Cx, role: String, name: &'static str, email: &'static str) -> Result<impl View> {
    Ok(view! {
        <span class="ld-who">
            <span aria-hidden="true">(initials(name))</span>
            <span>
                <b>(role)</b>
                <small>(name)</small>
                <small>(email)</small>
            </span>
        </span>
    })
}

/// A button that signs in to a panel as `email`.
///
/// It posts the panel's own login form, with the CSRF token the page set, so a visitor signs in
/// the way the login page signs them in: throttle, two-factor and all.
#[component]
async fn sign_in(
    cx: &Cx,
    action: String,
    email: &'static str,
    csrf: String,
    label: String,
    class: &'static str,
) -> Result<impl View> {
    Ok(view! {
        <form method="post" action=(action)>
            <input type="hidden" name=(tablo::csrf::FIELD_NAME) value=(tablo::csrf::mask(&csrf))>
            <input type="hidden" name=(tablo::auth::LOGIN_FIELD) value=(email)>
            <input type="hidden" name=(tablo::auth::PASSWORD_FIELD) value=(accounts::ADMIN_PASSWORD)>
            <button type="submit" class=(class)>
                (label)
                icon(data: ARROW)
            </button>
        </form>
    })
}
