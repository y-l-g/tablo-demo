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
const BOOK: IconData = iconify_icon!("lucide:book-open");
const USER_PLUS: IconData = iconify_icon!("lucide:user-plus");

/// What the demo shows, a tile each: its icon, its title and the line under it.
const FEATURES: [(IconData, &str, &str); 8] = [
    (
        iconify_icon!("lucide:layers"),
        "home-feature-resources",
        "home-feature-resources-body",
    ),
    (
        iconify_icon!("lucide:table"),
        "home-feature-tables",
        "home-feature-tables-body",
    ),
    (
        iconify_icon!("lucide:square-pen"),
        "home-feature-forms",
        "home-feature-forms-body",
    ),
    (
        iconify_icon!("lucide:zap"),
        "home-feature-actions",
        "home-feature-actions-body",
    ),
    (
        iconify_icon!("lucide:chart-column"),
        "home-feature-dashboards",
        "home-feature-dashboards-body",
    ),
    (
        iconify_icon!("lucide:shield-check"),
        "home-feature-auth",
        "home-feature-auth-body",
    ),
    (
        iconify_icon!("lucide:search"),
        "home-feature-search",
        "home-feature-search-body",
    ),
    (
        iconify_icon!("lucide:languages"),
        "home-feature-i18n",
        "home-feature-i18n-body",
    ),
];

/// The panels' locales, each named in its own language.
const LANGUAGES: [(&str, &str); 2] = [("en", "English"), ("fr", "Français")];

/// The cookies Tablo reads a visitor's language from, and `theme.js` their theme.
///
/// Tablo writes the language one under each panel's prefix, which `/` never receives, so the
/// page keeps its own copy at `/` and writes the panels' along with it.
const LOCALE_COOKIE: &str = "tablo_locale";
const THEME_COOKIE: &str = "tablo_theme";

/// One option of the page's language and theme switches.
const SEGMENT: &str = "grid h-7 min-w-7 place-items-center rounded-md px-1.5 text-xs text-muted-foreground transition-colors hover:text-foreground";

/// A switch of the page's header.
const SWITCH: &str =
    "flex rounded-lg border border-border bg-card/80 p-0.5 shadow-xs backdrop-blur";

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
        StaffRole::Administrator => (
            "home-role-administrator",
            "home-role-administrator-about",
            "bg-primary/10 text-primary",
        ),
        StaffRole::ShopManager => (
            "home-role-shop-manager",
            "home-role-shop-manager-about",
            "bg-warning/15 text-warning",
        ),
        StaffRole::HrManager => (
            "home-role-hr-manager",
            "home-role-hr-manager-about",
            "bg-success/15 text-success",
        ),
        StaffRole::Viewer => (
            "home-role-viewer",
            "home-role-viewer-about",
            "bg-muted text-muted-foreground",
        ),
    };
    let persona = |name, email, role| {
        let (role, about, tone) = staff_role(role);
        Persona {
            name,
            email,
            role: t(role),
            about: t(about),
            tone,
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
        tone: "bg-info/15 text-info",
    }];
    let features = FEATURES.map(|(data, title, body)| (data, t(title), t(body)));

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
    let admin_login = format!("{admin_url}/login");
    let sign_up = format!("{app_url}/register");
    let body = view! {
        <div lang=(lang) class="relative isolate min-h-screen overflow-hidden bg-background text-foreground">
            <div
                aria-hidden="true"
                class="pointer-events-none absolute inset-x-0 top-0 -z-10 h-[40rem] bg-[radial-gradient(55%_60%_at_50%_0%,color-mix(in_oklch,var(--primary)_24%,transparent),transparent)]"
            ></div>
            <div
                aria-hidden="true"
                class="pointer-events-none absolute inset-x-0 top-0 -z-10 h-[40rem] bg-[linear-gradient(to_right,var(--border)_1px,transparent_1px),linear-gradient(to_bottom,var(--border)_1px,transparent_1px)] bg-[size:3rem_3rem] [mask-image:radial-gradient(50%_60%_at_50%_0%,black,transparent)]"
            ></div>
            <div class="mx-auto flex w-full max-w-5xl flex-col px-6 py-6">
                <header class="flex flex-wrap items-center justify-between gap-3">
                    <a href="/" class="flex items-center gap-2.5 text-sm font-semibold">
                        <span class="grid size-8 place-items-center rounded-lg bg-primary font-bold text-primary-foreground shadow-sm">
                            "T"
                        </span>
                        "Tablo"
                    </a>
                    <div class="flex flex-wrap items-center gap-2">
                        <div role="group" aria-label=(t("tablo-shell-language")) class=(SWITCH)>
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
                        <div role="radiogroup" aria-label=(t("tablo-shell-theme")) class=(SWITCH)>
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
                    </div>
                </header>

                <main>
                    <section class="mx-auto mt-16 max-w-3xl text-center sm:mt-24">
                        <span class="inline-flex items-center gap-2 rounded-full border border-border bg-card/80 px-3 py-1 text-xs font-medium text-muted-foreground shadow-xs backdrop-blur">
                            <span class="size-1.5 rounded-full bg-primary"></span>
                            (t("home-eyebrow"))
                        </span>
                        <h1 class="mt-6 text-4xl font-semibold tracking-tight text-balance sm:text-5xl">
                            (t("home-title"))
                        </h1>
                        <p class="mx-auto mt-5 max-w-2xl text-base text-muted-foreground text-pretty sm:text-lg">
                            (t("home-lead"))
                            " "
                            (t("home-password-note"))
                            " "
                            <code class="rounded bg-muted px-1 font-mono text-foreground">(seed::ADMIN_PASSWORD)</code>
                            "."
                        </p>
                        <div class="mt-8 flex flex-wrap items-center justify-center gap-3">
                            <form method="post" action=(admin_login)>
                                <input type="hidden" name=(tablo::csrf::FIELD_NAME) value=(csrf.clone())>
                                <input type="hidden" name=(tablo::auth::LOGIN_FIELD) value=(seed::ADMIN_EMAIL)>
                                <input type="hidden" name=(tablo::auth::PASSWORD_FIELD) value=(seed::ADMIN_PASSWORD)>
                                <button
                                    type="submit"
                                    class="inline-flex h-11 items-center gap-2 rounded-lg bg-primary px-5 text-sm font-medium text-primary-foreground shadow-sm transition-colors hover:bg-primary/90 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background"
                                >
                                    (t("home-cta-admin"))
                                    icon(data: ARROW)
                                </button>
                            </form>
                            <a
                                href=(GUIDE)
                                class="inline-flex h-11 items-center gap-2 rounded-lg border border-border bg-card px-5 text-sm font-medium shadow-xs transition-colors hover:bg-muted"
                            >
                                icon(data: BOOK)
                                (t("home-cta-guide"))
                            </a>
                        </div>
                    </section>

                    <div class="mt-20 grid items-start gap-6 lg:grid-cols-5">
                        panel(
                            path: admin_url,
                            title: t("home-admin-title"),
                            body: t("home-admin-body"),
                            personas: admin_personas,
                            csrf: csrf.clone(),
                            open: t("home-sign-in-page"),
                            sign_up: None,
                            span: "lg:col-span-3",
                        )
                        panel(
                            path: app_url,
                            title: t("home-app-title"),
                            body: t("home-app-body"),
                            personas: app_personas,
                            csrf: csrf.clone(),
                            open: t("home-sign-in-page"),
                            sign_up: Some((sign_up, t("home-sign-up"))),
                            span: "lg:col-span-2",
                        )
                    </div>

                    <section class="mt-20">
                        <h2 class="text-center text-xs font-semibold uppercase tracking-widest text-muted-foreground">
                            (t("home-features-title"))
                        </h2>
                        <div class="mt-6 grid gap-px overflow-hidden rounded-2xl border border-border bg-border shadow-xs sm:grid-cols-2 lg:grid-cols-4">
                            for (data, title, body) in features {
                                <div class="bg-card p-5">
                                    <span class="grid size-9 place-items-center rounded-lg bg-primary/10 text-primary">
                                        icon(data: data)
                                    </span>
                                    <h3 class="mt-4 text-sm font-semibold">(title)</h3>
                                    <p class="mt-1 text-sm text-muted-foreground text-pretty">(body)</p>
                                </div>
                            }
                        </div>
                    </section>
                </main>

                <footer class="mt-20 flex flex-wrap items-center justify-between gap-4 border-t border-border py-6 text-sm text-muted-foreground">
                    <p>(t("home-built-with"))</p>
                    <nav class="flex gap-5">
                        <a href=(GUIDE) class="hover:text-foreground">(t("home-guide"))</a>
                        <a href=(TABLO) class="hover:text-foreground">"Tablo"</a>
                        <a href=(SOURCE) class="hover:text-foreground">(t("home-source"))</a>
                    </nav>
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

/// One account a visitor may continue as: who it is, what its role lets it do, and the tint of
/// its avatar.
struct Persona {
    name: &'static str,
    email: &'static str,
    role: String,
    about: String,
    tone: &'static str,
}

/// The initials of `name`, such as `DU` for `Demo User`.
fn initials(name: &str) -> String {
    name.split_whitespace()
        .filter_map(|word| word.chars().next())
        .take(2)
        .collect()
}

/// One panel: its path, what it is, and a button per account that signs in to it, with a link
/// to its sign-up page when it has one.
///
/// Each button posts the panel's own login form, with the CSRF token the page set, so a visitor
/// signs in the way the login page signs them in: throttle, two-factor and all.
#[component]
async fn panel(
    path: String,
    title: String,
    body: String,
    personas: Vec<Persona>,
    csrf: String,
    open: String,
    sign_up: Option<(String, String)>,
    span: &'static str,
) -> Result<impl View> {
    let login = format!("{path}/login");
    Ok(view! {
        <section class=(class!("flex flex-col rounded-2xl border border-border bg-card p-2 shadow-sm", span))>
            <div class="px-3 pt-3 pb-4">
                <div class="flex flex-wrap items-center gap-2">
                    <h2 class="text-base font-semibold">(title)</h2>
                    <code class="rounded-md bg-muted px-1.5 py-0.5 font-mono text-xs text-muted-foreground">(path.clone())</code>
                    <a href=(path) class="ml-auto text-sm font-medium text-primary underline-offset-4 hover:underline">(open)</a>
                </div>
                <p class="mt-1 text-sm text-muted-foreground text-pretty">(body)</p>
            </div>
            <ul class="flex flex-col gap-0.5">
                for persona in personas {
                    <li>
                        <form method="post" action=(login.clone())>
                            <input type="hidden" name=(tablo::csrf::FIELD_NAME) value=(csrf.clone())>
                            <input type="hidden" name=(tablo::auth::LOGIN_FIELD) value=(persona.email)>
                            <input type="hidden" name=(tablo::auth::PASSWORD_FIELD) value=(seed::ADMIN_PASSWORD)>
                            <button
                                type="submit"
                                class="group flex w-full items-center gap-3 rounded-xl px-3 py-2.5 text-left transition-colors hover:bg-muted focus-visible:bg-muted focus-visible:outline-none"
                            >
                                <span class=(class!("grid size-10 shrink-0 place-items-center rounded-full text-sm font-semibold", persona.tone))>
                                    (initials(persona.name))
                                </span>
                                <span class="min-w-0 flex-1">
                                    <span class="flex flex-wrap items-baseline gap-x-2">
                                        <span class="text-sm font-medium">(persona.role)</span>
                                        <span class="text-xs text-muted-foreground">(persona.name)</span>
                                    </span>
                                    <span class="mt-0.5 block text-xs text-muted-foreground text-pretty">(persona.about)</span>
                                </span>
                                <span class="text-muted-foreground transition group-hover:translate-x-0.5 group-hover:text-foreground">
                                    icon(data: ARROW)
                                </span>
                            </button>
                        </form>
                    </li>
                }
            </ul>
            if let Some((href, label)) = sign_up {
                <a
                    href=(href)
                    class="m-1 mt-2 flex items-center justify-center gap-2 rounded-xl border border-dashed border-border px-3 py-3 text-sm font-medium text-muted-foreground transition-colors hover:border-primary/40 hover:text-foreground"
                >
                    icon(data: USER_PLUS)
                    (label)
                </a>
            }
        </section>
    })
}
