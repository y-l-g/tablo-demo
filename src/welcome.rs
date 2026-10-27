//! The public page at `/`: the two panels the demo mounts, and an account to sign in to each.

use serde::Deserialize;
use tablo::PanelHandle;
use topcoat::{
    Result,
    context::{Cx, context},
    cookie::{Cookie, Cookies, SameSite, cookies},
    icon::{IconData, icon, iconify::iconify_icon},
    router::{
        HeaderMap, HeaderValue,
        content::Form,
        error::{SeeOther, forbidden, see_other},
        header::SET_COOKIE,
        page, route,
    },
    view::{View, class, component, view},
};

use crate::{enums::StaffRole, seed};

/// The mounted admin panel, whose document, translations and URLs the page borrows.
pub struct AdminPanel(pub PanelHandle);

/// The mounted app panel, which the page links to.
pub struct AppPanel(pub PanelHandle);

const TABLO: &str = "https://github.com/y-l-g/tablo";
const GUIDE: &str = "https://y-l.fr/tablo/nightly/guide/";
const SOURCE: &str = "https://github.com/y-l-g/tablo-demo";

const ARROW: IconData = iconify_icon!("lucide:arrow-right");

/// The panels' locales, each named in its own language.
const LANGUAGES: [(&str, &str); 2] = [("en", "English"), ("fr", "Français")];

/// The cookies Tablo reads a visitor's language from, and `theme.js` their theme.
///
/// Tablo writes the language one under each panel's prefix, which `/` never receives, so the
/// page keeps its own copy at `/` and writes the panels' along with it.
const LOCALE_COOKIE: &str = "tablo_locale";
const THEME_COOKIE: &str = "tablo_theme";

/// One option of the page's language and theme switches.
const SEGMENT: &str = "grid h-7 min-w-7 place-items-center rounded-[5px] px-1.5 text-xs text-muted-foreground transition-colors hover:text-foreground";

/// `GET /` — the welcome page, rendered in the admin panel's document and catalog.
#[page("/")]
async fn welcome(cx: &Cx) -> Result<impl View> {
    let AdminPanel(admin) = context::<AdminPanel>(cx);
    let AppPanel(app) = context::<AppPanel>(cx);
    // Entering the admin panel gives the page its catalog and locale.
    let outer = cx;
    let entered = admin.enter(cx);
    let cx = &entered;
    let t = |id: &'static str| tablo::t!(cx, id);
    let admin_url = admin.url().unwrap_or_else(|| "/admin".to_string());
    let app_url = app.url().unwrap_or_else(|| "/app".to_string());

    // The buttons post the panels' login forms, which check this token against its cookie.
    let csrf = tablo::csrf::ensure_token(outer);
    let staff_role = |role: StaffRole| match role {
        StaffRole::Administrator => ("home-role-administrator", "home-role-administrator-about"),
        StaffRole::ShopManager => ("home-role-shop-manager", "home-role-shop-manager-about"),
        StaffRole::HrManager => ("home-role-hr-manager", "home-role-hr-manager-about"),
        StaffRole::Viewer => ("home-role-viewer", "home-role-viewer-about"),
    };
    let persona = |name, email, role| {
        let (role, about) = staff_role(role);
        Persona {
            name,
            email,
            role: t(role),
            about: t(about),
        }
    };
    let mut admin_personas = vec![persona(
        seed::ADMIN_NAME,
        seed::ADMIN_EMAIL,
        StaffRole::Administrator,
    )];
    admin_personas.extend(
        seed::STAFF
            .iter()
            .map(|account| persona(account.name, account.email, account.role)),
    );
    let app_personas = vec![Persona {
        name: seed::ADMIN_NAME,
        email: seed::ADMIN_EMAIL,
        role: t("home-role-member"),
        about: t("home-role-member-about"),
    }];

    let locale = topcoat::l10n::try_locale(cx).map(ToString::to_string);
    let languages = LANGUAGES.map(|(tag, name)| (tag, name, locale.as_deref() == Some(tag)));
    // `theme.js` checks the theme picked on click; the page checks the stored one. The panels
    // default to light.
    let stored = cookies(outer).get(THEME_COOKIE);
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
    let body = view! {
        <div lang=(lang) class="min-h-screen bg-background text-foreground">
            <div class="mx-auto flex w-full max-w-2xl flex-col px-6 py-8 sm:py-12">
                <header class="flex flex-wrap items-center justify-end gap-2">
                    <div
                        role="group"
                        aria-label=(t("tablo-shell-language"))
                        class="flex rounded-md border border-border p-0.5"
                    >
                        for (tag, name, current) in languages {
                            <form method="post" action="/locale">
                                <input type="hidden" name=(tablo::csrf::FIELD_NAME) value=(csrf.clone())>
                                <input type="hidden" name="locale" value=(tag)>
                                <button
                                    type="submit"
                                    title=(name)
                                    aria-current=(current.then_some("true"))
                                    class=(class!(SEGMENT, "uppercase aria-[current=true]:bg-muted aria-[current=true]:text-foreground"))
                                >
                                    (tag)
                                </button>
                            </form>
                        }
                    </div>
                    <div
                        role="radiogroup"
                        aria-label=(t("tablo-shell-theme"))
                        class="flex rounded-md border border-border p-0.5"
                    >
                        for (name, data, label, checked) in themes {
                            <button
                                type="button"
                                role="radio"
                                data-theme-set=(name)
                                aria-checked=(if checked { "true" } else { "false" })
                                aria-label=(label.clone())
                                title=(label)
                                class=(class!(SEGMENT, "aria-checked:bg-muted aria-checked:text-foreground"))
                            >
                                icon(data: data)
                            </button>
                        }
                    </div>
                </header>

                <main class="mt-16 sm:mt-24">
                    <h1 class="text-3xl font-semibold tracking-tight">(t("home-title"))</h1>
                    <p class="mt-3 text-muted-foreground text-pretty">
                        (t("home-lead"))
                        " "
                        (t("home-password-note"))
                        " "
                        <code class="font-mono text-foreground">(seed::ADMIN_PASSWORD)</code>
                        "."
                    </p>
                    panel(
                        path: admin_url,
                        body: t("home-admin-body"),
                        personas: admin_personas,
                        csrf: csrf.clone(),
                        open: t("home-sign-in-page"),
                    )
                    panel(
                        path: app_url,
                        body: t("home-app-body"),
                        personas: app_personas,
                        csrf: csrf.clone(),
                        open: t("home-sign-in-page"),
                    )
                </main>

                <footer class="mt-16 flex flex-wrap gap-x-5 gap-y-2 border-t border-border pt-6 text-sm text-muted-foreground">
                    <a href=(GUIDE) class="hover:text-foreground">(t("home-guide"))</a>
                    <a href=(TABLO) class="hover:text-foreground">"Tablo"</a>
                    <a href=(SOURCE) class="hover:text-foreground">(t("home-source"))</a>
                </footer>
            </div>
        </div>
    };
    admin.document(outer, title, body).await
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
    // The panels' own copies too, which outrank this one under their prefixes. The cookie jar
    // keeps one cookie per name, so the three go out as headers of their own.
    let paths = ["/".to_string()]
        .into_iter()
        .chain(admin.url())
        .chain(app.url());
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

/// One account a visitor may continue as: who it is, and what its role lets it do.
struct Persona {
    name: &'static str,
    email: &'static str,
    role: String,
    about: String,
}

/// One panel: its path, what it is, and a button per account that signs in to it.
///
/// Each button posts the panel's own login form, with the CSRF token the page set, so a visitor
/// signs in the way the login page signs them in: throttle, two-factor and all.
#[component]
async fn panel(
    path: String,
    body: String,
    personas: Vec<Persona>,
    csrf: String,
    open: String,
) -> Result<impl View> {
    let login = format!("{path}/login");
    Ok(view! {
        <section class="mt-12">
            <div class="flex items-baseline justify-between gap-4">
                <h2 class="font-mono text-sm font-medium">(path.clone())</h2>
                <a href=(path) class="text-sm text-muted-foreground hover:text-foreground">(open)</a>
            </div>
            <p class="mt-1 text-sm text-muted-foreground text-pretty">(body)</p>
            <ul class="mt-4 divide-y divide-border overflow-hidden rounded-lg border border-border">
                for persona in personas {
                    <li>
                        <form method="post" action=(login.clone())>
                            <input type="hidden" name=(tablo::csrf::FIELD_NAME) value=(csrf.clone())>
                            <input type="hidden" name=(tablo::auth::LOGIN_FIELD) value=(persona.email)>
                            <input type="hidden" name=(tablo::auth::PASSWORD_FIELD) value=(seed::ADMIN_PASSWORD)>
                            <button
                                type="submit"
                                class="group flex w-full items-center gap-4 px-4 py-3 text-left transition-colors hover:bg-muted/50 focus-visible:bg-muted/50 focus-visible:outline-none"
                            >
                                <span class="min-w-0 flex-1">
                                    <span class="flex flex-wrap items-baseline gap-x-2 text-sm">
                                        <span class="font-medium">(persona.role)</span>
                                        <span class="text-xs text-muted-foreground">(persona.name)</span>
                                    </span>
                                    <span class="mt-0.5 block text-xs text-muted-foreground">(persona.about)</span>
                                </span>
                                <span class="text-muted-foreground transition-transform group-hover:translate-x-0.5 group-hover:text-foreground">
                                    icon(data: ARROW)
                                </span>
                            </button>
                        </form>
                    </li>
                }
            </ul>
        </section>
    })
}
