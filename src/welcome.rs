//! The public page at `/`: what the demo is, the two panels it mounts, and a tour of the
//! features worth trying, each linked to the page that shows it and the code that declares it.

use tablo::{PanelHandle, Resource, db::db};
use topcoat::{
    Result,
    context::{Cx, context},
    icon::{IconData, icon, iconify::iconify_icon},
    router::page,
    view::{Length, View, class, component, view},
};

use crate::{
    audit::AuditTrail,
    dashboards::ShopDashboard,
    enums::{LeaveStatus, OrderStatus, StaffRole},
    hr::{EmployeeResource, LeaveRequestResource, TaskResource},
    models::{Customer, Employee, Order, Product},
    seed,
    shop::{CustomerResource, OrderResource},
};

/// The mounted admin panel, whose document, translations and URLs the page borrows.
pub struct AdminPanel(pub PanelHandle);

/// The mounted app panel, which the page links to.
pub struct AppPanel(pub PanelHandle);

/// Where the demo's source is read, one file per tour stop.
const SOURCE: &str = "https://github.com/y-l-g/tablo-demo/blob/main";
const TABLO: &str = "https://github.com/y-l-g/tablo";
const GUIDE: &str = "https://y-l.fr/tablo/nightly/guide/";

const ADMIN_ICON: IconData = iconify_icon!("lucide:shield-check");
const APP_ICON: IconData = iconify_icon!("lucide:users-round");
const ARROW: IconData = iconify_icon!("lucide:arrow-right");
const CODE: IconData = iconify_icon!("lucide:code-xml");
const GITHUB: IconData = iconify_icon!("lucide:github");
const BOOK: IconData = iconify_icon!("lucide:book-open");
const RESET: IconData = iconify_icon!("lucide:rotate-ccw");

/// One feature to try: where it is, what to do there, and the file that declares it.
struct Stop {
    icon: IconData,
    title: String,
    body: String,
    href: Option<String>,
    source: &'static str,
}

/// `GET /` — the welcome page, rendered in the admin panel's document and catalog.
#[page("/")]
async fn welcome(cx: &Cx) -> Result<impl View> {
    let AdminPanel(admin) = context::<AdminPanel>(cx);
    let AppPanel(app) = context::<AppPanel>(cx);
    // Entering the admin panel gives the page its catalog, locale and resource URLs.
    let outer = cx;
    let entered = admin.enter(cx);
    let cx = &entered;
    let t = |id: &'static str| tablo::t!(cx, id);
    let admin_url = admin.url().unwrap_or_else(|| "/admin".to_string());
    let app_url = app.url().unwrap_or_else(|| "/app".to_string());

    let mut db = db(cx)?;
    let figures = [
        (
            Product::all().exec(&mut db).await?.len(),
            t("home-figure-products"),
        ),
        (
            Order::all().exec(&mut db).await?.len(),
            t("home-figure-orders"),
        ),
        (
            Customer::all().exec(&mut db).await?.len(),
            t("home-figure-customers"),
        ),
        (
            Employee::all().exec(&mut db).await?.len(),
            t("home-figure-employees"),
        ),
    ];

    let admin_stops = vec![
        Stop {
            icon: iconify_icon!("lucide:truck"),
            title: t("home-try-orders"),
            body: t("home-try-orders-body"),
            href: filtered::<OrderResource>(cx, OrderStatus::Processing.value()),
            source: "src/shop.rs",
        },
        Stop {
            icon: iconify_icon!("lucide:calendar-check"),
            title: t("home-try-leave"),
            body: t("home-try-leave-body"),
            href: filtered::<LeaveRequestResource>(cx, LeaveStatus::Pending.value()),
            source: "src/hr.rs",
        },
        Stop {
            icon: iconify_icon!("lucide:kanban"),
            title: t("home-try-tasks"),
            body: t("home-try-tasks-body"),
            href: tablo::url::resource::<TaskResource>(cx),
            source: "src/hr.rs",
        },
        Stop {
            icon: iconify_icon!("lucide:toggle-right"),
            title: t("home-try-conditional"),
            body: t("home-try-conditional-body"),
            href: tablo::url::create::<EmployeeResource>(cx),
            source: "src/hr.rs",
        },
        Stop {
            icon: iconify_icon!("lucide:archive-restore"),
            title: t("home-try-trash"),
            body: t("home-try-trash-body"),
            href: tablo::url::resource::<CustomerResource>(cx),
            source: "src/shop.rs",
        },
        Stop {
            icon: iconify_icon!("lucide:chart-no-axes-combined"),
            title: t("home-try-dashboards"),
            body: t("home-try-dashboards-body"),
            href: tablo::url::page::<ShopDashboard>(cx),
            source: "src/dashboards.rs",
        },
        Stop {
            icon: crate::icons::AUDIT_TRAIL,
            title: t("home-try-audit"),
            body: t("home-try-audit-body"),
            href: tablo::url::page::<AuditTrail>(cx),
            source: "src/audit.rs",
        },
        Stop {
            icon: iconify_icon!("lucide:shield-check"),
            title: t("home-try-roles"),
            body: t("home-try-roles-body"),
            href: tablo::url::resource::<EmployeeResource>(cx),
            source: "src/roles.rs",
        },
    ];
    let app_stops = vec![
        Stop {
            icon: iconify_icon!("lucide:user-plus"),
            title: t("home-try-sign-up"),
            body: t("home-try-sign-up-body"),
            href: Some(format!("{app_url}/register")),
            source: "src/auth.rs",
        },
        Stop {
            icon: iconify_icon!("lucide:key-round"),
            title: t("home-try-two-factor"),
            body: t("home-try-two-factor-body"),
            href: Some(app_url.clone()),
            source: "src/panel.rs",
        },
        Stop {
            icon: iconify_icon!("lucide:arrow-left-right"),
            title: t("home-try-teams"),
            body: t("home-try-teams-body"),
            href: Some(app_url.clone()),
            source: "src/auth.rs",
        },
        Stop {
            icon: iconify_icon!("lucide:languages"),
            title: t("home-try-language"),
            body: t("home-try-language-body"),
            href: Some(app_url.clone()),
            source: "locales/fr.ftl",
        },
    ];

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

    let title = t("home-document-title");
    // The document is the outer request's, which knows no locale, so the page states its own.
    let lang = t("home-lang");
    // The view renders after this function returns, so it translates through a clone it owns.
    let t = {
        let cx = entered.clone();
        move |id: &'static str| tablo::t!(&cx, id)
    };
    let body = view! {
        <div lang=(lang) class="flex min-h-screen flex-col bg-background text-foreground">
            <header class="border-b border-border">
                <nav class="mx-auto flex w-full max-w-6xl items-center gap-2 px-6 py-4">
                    <span class="flex items-center gap-2.5 font-semibold tracking-tight">
                        <span
                            class="grid size-7 place-items-center rounded-md bg-primary text-sm text-primary-foreground"
                        >
                            "T"
                        </span>
                        "Tablo Demo"
                    </span>
                    <a href=(GUIDE) class=(class!("ml-auto", tablo::ui::button_variants(tablo::ui::ButtonVariant::Ghost, tablo::ui::ButtonSize::Sm)))>
                        icon(data: BOOK)
                        (t("home-guide"))
                    </a>
                    <a href=(TABLO) class=(class!(tablo::ui::button_variants(tablo::ui::ButtonVariant::Ghost, tablo::ui::ButtonSize::Sm)))>
                        icon(data: GITHUB)
                        "GitHub"
                    </a>
                </nav>
            </header>

            <main class="mx-auto w-full max-w-6xl flex-1 px-6">
                <section class="py-16 sm:py-24">
                    <p
                        class="inline-flex items-center gap-2 rounded-full border border-border px-3 py-1 text-xs text-muted-foreground"
                    >
                        icon(data: RESET)
                        (t("home-reset-note"))
                    </p>
                    <h1
                        class="mt-6 max-w-3xl text-4xl font-semibold tracking-tight text-balance sm:text-5xl"
                    >
                        (t("home-title"))
                    </h1>
                    <p class="mt-5 max-w-2xl text-lg text-muted-foreground text-pretty">
                        (t("home-lead"))
                    </p>
                    <dl
                        class="mt-10 grid max-w-2xl grid-cols-2 gap-px overflow-hidden rounded-lg border border-border bg-border sm:grid-cols-4"
                    >
                        for (count, label) in figures {
                            <div class="bg-background px-4 py-3">
                                <dt class="text-xs text-muted-foreground">(label)</dt>
                                <dd class="mt-1 text-2xl font-semibold tabular-nums">
                                    (count.to_string())
                                </dd>
                            </div>
                        }
                    </dl>
                </section>

                <section class="grid gap-6 pb-16 md:grid-cols-2">
                    panel_card(
                        icon_data: ADMIN_ICON,
                        title: t("home-admin-title"),
                        path: admin_url.clone(),
                        body: t("home-admin-body"),
                        points: vec![
                            t("home-admin-point-tables"),
                            t("home-admin-point-actions"),
                            t("home-admin-point-forms"),
                            t("home-admin-point-dashboards"),
                            t("home-admin-point-roles"),
                        ],
                        personas: admin_personas,
                        csrf: csrf.clone(),
                        sign_in: t("home-sign-in-as"),
                        note: t("home-password-note"),
                        cta: t("home-admin-open"),
                    )
                    panel_card(
                        icon_data: APP_ICON,
                        title: t("home-app-title"),
                        path: app_url.clone(),
                        body: t("home-app-body"),
                        points: vec![
                            t("home-app-point-sign-up"),
                            t("home-app-point-two-factor"),
                            t("home-app-point-teams"),
                            t("home-app-point-profile"),
                        ],
                        personas: app_personas,
                        csrf: csrf,
                        sign_in: t("home-sign-in-as"),
                        note: t("home-app-note"),
                        cta: t("home-app-open"),
                    )
                </section>

                <section class="border-t border-border py-16">
                    <h2 class="text-2xl font-semibold tracking-tight">
                        (t("home-tour-title"))
                    </h2>
                    <p class="mt-2 max-w-2xl text-muted-foreground">
                        (t("home-tour-lead"))
                    </p>
                    tour(heading: t("home-tour-admin"), stops: admin_stops)
                    tour(heading: t("home-tour-app"), stops: app_stops)
                </section>
            </main>

            <footer class="border-t border-border">
                <div
                    class="mx-auto flex w-full max-w-6xl flex-wrap items-center gap-x-6 gap-y-2 px-6 py-6 text-sm text-muted-foreground"
                >
                    <p>(t("home-footer"))</p>
                    <a
                        href="https://github.com/y-l-g/tablo-demo"
                        class="ml-auto hover:text-foreground"
                    >
                        (t("home-footer-source"))
                    </a>
                </div>
            </footer>
        </div>
    };
    admin.document(outer, title, body).await
}

/// `R`'s list filtered to `status`, the link a dashboard tile gives.
fn filtered<R: Resource>(cx: &Cx, status: &str) -> Option<String> {
    tablo::url::resource::<R>(cx).map(|url| format!("{url}?f.status={status}"))
}

/// One account a visitor may continue as: who it is, and what its role lets it do.
struct Persona {
    name: &'static str,
    email: &'static str,
    role: String,
    about: String,
}

/// One panel: what it is, what it shows, and a button per account that signs in to it.
///
/// Each button posts the panel's own login form, with the CSRF token the page set, so a visitor
/// signs in the way the login page signs them in: throttle, two-factor and all.
#[component]
async fn panel_card(
    icon_data: IconData,
    title: String,
    path: String,
    body: String,
    points: Vec<String>,
    personas: Vec<Persona>,
    csrf: String,
    sign_in: String,
    note: String,
    cta: String,
) -> Result<impl View> {
    let login = format!("{path}/login");
    Ok(view! {
        <article class="flex flex-col rounded-xl border border-border bg-card p-6 text-card-foreground shadow-xs">
            <div class="flex items-center gap-3">
                <span class="grid size-10 place-items-center rounded-lg bg-muted text-foreground">
                    icon(data: icon_data, size: Length::rem(1.25))
                </span>
                <div>
                    <h2 class="font-semibold tracking-tight">(title)</h2>
                    <p class="font-mono text-xs text-muted-foreground">(path.clone())</p>
                </div>
            </div>
            <p class="mt-4 text-sm text-muted-foreground">(body)</p>
            <ul class="mt-4 grid gap-2 text-sm">
                for point in points {
                    <li class="flex gap-2">
                        <span class="mt-2 size-1.5 shrink-0 rounded-full bg-primary"></span>
                        (point)
                    </li>
                }
            </ul>
            <h3 class="mt-6 text-sm font-medium">(sign_in)</h3>
            <ul class="mt-3 grid gap-2">
                for persona in personas {
                    <li>
                        <form method="post" action=(login.clone())>
                            <input type="hidden" name=(tablo::csrf::FIELD_NAME) value=(csrf.clone())>
                            <input type="hidden" name=(tablo::auth::LOGIN_FIELD) value=(persona.email)>
                            <input type="hidden" name=(tablo::auth::PASSWORD_FIELD) value=(seed::ADMIN_PASSWORD)>
                            <button
                                type="submit"
                                class="group flex w-full items-center gap-3 rounded-lg border border-border px-3 py-2.5 text-left transition-colors hover:bg-muted/60 focus-visible:outline-2 focus-visible:outline-ring"
                            >
                                <span class="grid size-8 shrink-0 place-items-center rounded-full bg-muted text-xs font-medium">
                                    (initials(persona.name))
                                </span>
                                <span class="min-w-0 flex-1">
                                    <span class="flex flex-wrap items-baseline gap-x-2 text-sm">
                                        <span class="font-medium">(persona.name)</span>
                                        <span class="text-xs text-muted-foreground">(persona.role)</span>
                                    </span>
                                    <span class="block text-xs text-muted-foreground">(persona.about)</span>
                                </span>
                                <span class="text-muted-foreground transition-transform group-hover:translate-x-0.5 group-hover:text-foreground">
                                    icon(data: ARROW)
                                </span>
                            </button>
                        </form>
                    </li>
                }
            </ul>
            <p class="mt-4 flex flex-wrap items-center gap-x-3 gap-y-1 text-xs text-muted-foreground">
                <span>
                    (note)
                    " "
                    <code class="font-mono text-foreground">(seed::ADMIN_PASSWORD)</code>
                </span>
                <a href=(path) class="ml-auto inline-flex items-center gap-1 font-medium text-foreground hover:underline">
                    (cta)
                    icon(data: ARROW)
                </a>
            </p>
        </article>
    })
}

/// The first letter of each of a name's first two words, for an avatar.
fn initials(name: &str) -> String {
    name.split_whitespace()
        .take(2)
        .filter_map(|word| word.chars().next())
        .collect()
}

/// A group of tour stops, each a link to its page beside a link to its source.
#[component]
async fn tour(heading: String, stops: Vec<Stop>) -> Result<impl View> {
    Ok(view! {
        <h3 class="mt-10 text-sm font-medium text-muted-foreground">(heading)</h3>
        <ul class="mt-4 grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
            for stop in stops {
                <li class="group relative flex flex-col rounded-lg border border-border p-4 transition-colors hover:bg-muted/50">
                    <span class="text-muted-foreground group-hover:text-foreground">
                        icon(data: stop.icon, size: Length::rem(1.25))
                    </span>
                    if let Some(href) = stop.href {
                        <a href=(href) class="mt-3 font-medium after:absolute after:inset-0">
                            (stop.title)
                        </a>
                    } else {
                        <p class="mt-3 font-medium">(stop.title)</p>
                    }
                    <p class="mt-1 flex-1 text-sm text-muted-foreground">(stop.body)</p>
                    <a
                        href=(format!("{SOURCE}/{}", stop.source))
                        class="relative z-10 mt-3 inline-flex items-center gap-1.5 self-start font-mono text-xs text-muted-foreground hover:text-foreground"
                    >
                        icon(data: CODE)
                        (stop.source)
                    </a>
                </li>
            }
        </ul>
    })
}
