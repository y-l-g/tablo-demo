//! The appearance drawer: a button in the corner of every panel page that opens a sheet to pick
//! the color scheme, one of Tablo's themes and the sidebar's frame.
//!
//! The scheme is Tablo's own `tablo_theme` cookie, which `theme.js` switches in place. The theme
//! and the frame are cookies of the demo's, shared by the three panels: a pick posts to
//! [`pick_appearance`], which stores it and sends the visitor back to the page, rendered in it
//! by [`visitor_look`] with the drawer still open. The drawer opens and closes through the
//! browser's own `commandfor` buttons: a script in a panel page's body never runs, since Topcoat's
//! runtime streams it in.

use serde::Deserialize;
use tablo::{
    Banner, Look, SidebarFrame, Theme,
    ui::{ButtonSize, ButtonVariant, button_variants, icons},
};
use topcoat::{
    Result,
    context::Cx,
    cookie::{Cookie, Cookies, SameSite, cookies},
    icon::{IconData, icon, iconify::iconify_icon},
    router::{
        HeaderMap, HeaderValue,
        content::Form,
        error::{SeeOther, forbidden, see_other},
        header::SET_COOKIE,
        request::original_uri,
        route,
    },
    view::{View, component, view},
};

const THEME_COOKIE: &str = "tablo_demo_theme";
const SIDEBAR_COOKIE: &str = "tablo_demo_sidebar";
/// Set by a pick for the page it returns to, which opens the drawer and drops the cookie.
const OPEN_COOKIE: &str = "tablo_demo_appearance";
/// Tablo's scheme cookie; the panels start light without it.
const SCHEME_COOKIE: &str = "tablo_theme";

const PALETTE: IconData = iconify_icon!("lucide:palette");

/// A theme as the drawer offers it: its cookie value, the theme, and the colors its preview
/// paints, copied from Tablo's `theme.css`.
struct ThemeChoice {
    name: &'static str,
    theme: Theme,
    light: Swatch,
    dark: Swatch,
}

/// A theme's preview colors in one scheme: the page, the sidebar and the primary.
struct Swatch {
    page: &'static str,
    sidebar: &'static str,
    primary: &'static str,
}

const THEMES: [ThemeChoice; 4] = [
    ThemeChoice {
        name: "jade",
        theme: Theme::Jade,
        light: Swatch {
            page: "oklch(0.995 0.0015 170)",
            sidebar: "oklch(0.962 0.006 172)",
            primary: "oklch(0.5 0.105 168)",
        },
        dark: Swatch {
            page: "oklch(0.185 0.007 200)",
            sidebar: "oklch(0.15 0.006 200)",
            primary: "oklch(0.77 0.125 168)",
        },
    },
    ThemeChoice {
        name: "amber",
        theme: Theme::Amber,
        light: Swatch {
            page: "oklch(0.985 0 0)",
            sidebar: "oklch(1 0 0)",
            primary: "oklch(0.666 0.179 58.3)",
        },
        dark: Swatch {
            page: "oklch(0.141 0.005 286)",
            sidebar: "oklch(0.21 0.006 286)",
            primary: "oklch(0.769 0.188 70)",
        },
    },
    ThemeChoice {
        name: "graphite",
        theme: Theme::Graphite,
        light: Swatch {
            page: "oklch(0.99 0 0)",
            sidebar: "oklch(0.2 0 0)",
            primary: "oklch(0.2 0 0)",
        },
        dark: Swatch {
            page: "oklch(0.16 0 0)",
            sidebar: "oklch(0.12 0 0)",
            primary: "oklch(0.96 0 0)",
        },
    },
    ThemeChoice {
        name: "iris",
        theme: Theme::Iris,
        light: Swatch {
            page: "oklch(0.982 0.005 285)",
            sidebar: "oklch(1 0 0)",
            primary: "oklch(0.52 0.21 280)",
        },
        dark: Swatch {
            page: "oklch(0.165 0.02 282)",
            sidebar: "oklch(0.2 0.024 282)",
            primary: "oklch(0.73 0.15 282)",
        },
    },
];

/// The sidebar frames by cookie value, with their labels.
const FRAMES: [(&str, SidebarFrame, &str); 3] = [
    ("ruled", SidebarFrame::Ruled, "appearance-sidebar-ruled"),
    ("inset", SidebarFrame::Inset, "appearance-sidebar-inset"),
    (
        "floating",
        SidebarFrame::Floating,
        "appearance-sidebar-floating",
    ),
];

/// The value of the cookie `name` on the request, if any.
fn cookie(cx: &Cx, name: &str) -> Option<String> {
    cookies(cx)
        .get(name)
        .map(|cookie| cookie.value().to_owned())
}

/// The theme and frame the visitor picked in the drawer, over the panel's own.
pub(super) fn visitor_look(cx: &Cx) -> Look {
    let theme = cookie(cx, THEME_COOKIE)
        .and_then(|name| THEMES.into_iter().find(|choice| choice.name == name))
        .map(|choice| choice.theme);
    let frame = cookie(cx, SIDEBAR_COOKIE)
        .and_then(|name| FRAMES.into_iter().find(|(value, ..)| *value == name))
        .map(|(_, frame, _)| frame);
    let look = theme.map_or_else(Look::default, |theme| Look::default().theme(theme));
    frame.map_or(look, |frame| look.sidebar(frame))
}

/// The drawer, shown on every page a panel's shell frames.
///
/// It rides on the banner seam because Tablo asks a banner of exactly those pages, and leaves it
/// out of the frames a creatable choice's dialog opens; it is fixed, so it takes no room above
/// the page.
pub struct Appearance;

impl Banner for Appearance {
    async fn render(cx: &Cx) -> Result<Option<impl View>> {
        let t = |id: &'static str| tablo::t!(cx, id);
        let reopen = cookie(cx, OPEN_COOKIE).is_some();
        if reopen {
            cookies(cx).remove(Cookie::build(OPEN_COOKIE).path("/"));
        }
        let scheme = cookie(cx, SCHEME_COOKIE).unwrap_or_else(|| "light".to_owned());
        let modes = [
            ("light", icons::SUN, t("tablo-shell-theme-light")),
            ("dark", icons::MOON, t("tablo-shell-theme-dark")),
            ("system", icons::MONITOR, t("tablo-shell-theme-system")),
        ]
        .map(|(name, data, label)| (name, data, label, name == scheme));
        let theme = Theme::current(cx);
        let themes = THEMES.map(|choice| {
            let checked = Some(choice.theme) == theme;
            (choice.name, swatch_style(&choice), checked)
        });
        let frame = SidebarFrame::current(cx);
        let frames = FRAMES.map(|(name, value, label)| (name, t(label), Some(value) == frame));
        let csrf = tablo::csrf::ensure_token(cx);
        let back = original_uri(cx)
            .path_and_query()
            .map_or_else(|| "/".to_owned(), ToString::to_string);
        let close = button_variants(ButtonVariant::Ghost, ButtonSize::Icon);
        let reset = button_variants(ButtonVariant::Outline, ButtonSize::Md);

        Ok(Some(view! {
            <button
                type="button"
                commandfor="appearance"
                command="show-modal"
                aria-label=(t("appearance-open"))
                title=(t("appearance-open"))
                class="fixed right-4 bottom-4 z-40 inline-flex size-12 items-center justify-center rounded-full bg-primary text-primary-foreground shadow-lg ring-1 ring-black/10 transition-transform outline-none hover:scale-105 focus-visible:ring-[3px] focus-visible:ring-ring/50 motion-reduce:transition-none [&_svg]:size-5"
            >
                icon(data: PALETTE)
            </button>
            <dialog
                id="appearance"
                open=(reopen)
                aria-labelledby="appearance-title"
                aria-describedby="appearance-description"
                class="fixed inset-0 z-50 size-full max-h-none max-w-none overflow-hidden bg-black/40 text-foreground opacity-0 open:flex open:opacity-100 starting:open:opacity-0 [transition:opacity_200ms_ease-out,display_200ms_allow-discrete,overlay_200ms_allow-discrete] backdrop:bg-transparent motion-reduce:transition-none"
            >
                // The shade around the sheet closes it, as Escape does.
                <button
                    type="button"
                    commandfor="appearance"
                    command="close"
                    tabindex="-1"
                    aria-hidden="true"
                    class="absolute inset-0 cursor-default"
                ></button>
                <form
                    method="post"
                    action="/appearance"
                    class="relative ml-auto flex h-full w-full max-w-sm translate-x-full flex-col gap-6 overflow-y-auto border-l border-border bg-card p-6 text-card-foreground shadow-xl [transition:translate_200ms_ease-out] in-[[open]]:translate-x-0 starting:in-[[open]]:translate-x-full motion-reduce:transition-none"
                >
                    <input type="hidden" name=(tablo::csrf::FIELD_NAME) value=(csrf)>
                    <input type="hidden" name="back" value=(back)>
                    <div class="flex items-start justify-between gap-4">
                        <div class="grid gap-1">
                            <h2 id="appearance-title" class="text-base font-semibold">
                                (t("appearance-title"))
                            </h2>
                            <p id="appearance-description" class="text-sm text-muted-foreground">
                                (t("appearance-description"))
                            </p>
                        </div>
                        <button
                            type="button"
                            commandfor="appearance"
                            command="close"
                            aria-label=(t("appearance-close"))
                            class=(close)
                        >
                            icon(data: icons::X)
                        </button>
                    </div>

                    <section class="grid gap-2" aria-labelledby="appearance-mode">
                        <h3 id="appearance-mode" class="text-sm font-medium">(t("appearance-mode"))</h3>
                        <div role="radiogroup" aria-labelledby="appearance-mode" class="grid grid-cols-3 gap-1 rounded-lg bg-muted p-1">
                            for (name, data, label, checked) in modes {
                                <button
                                    type="button"
                                    role="radio"
                                    data-theme-set=(name)
                                    aria-checked=(if checked { "true" } else { "false" })
                                    class="inline-flex h-8 items-center justify-center gap-1.5 rounded-md text-sm text-muted-foreground transition-colors outline-none hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring aria-checked:bg-background aria-checked:text-foreground aria-checked:shadow-xs [&_svg]:size-4"
                                >
                                    icon(data: data)
                                    (label)
                                </button>
                            }
                        </div>
                    </section>

                    <section class="grid gap-2" aria-labelledby="appearance-theme">
                        <h3 id="appearance-theme" class="text-sm font-medium">(t("appearance-theme"))</h3>
                        <div class="grid grid-cols-2 gap-2">
                            for (name, style, checked) in themes {
                                <button
                                    type="submit"
                                    name="theme"
                                    value=(name)
                                    aria-pressed=(if checked { "true" } else { "false" })
                                    style=(style)
                                    class=(PICK)
                                >
                                    <span aria-hidden="true" class="flex h-14 overflow-hidden rounded-md border border-border">
                                        <span class="w-1/3 bg-(--sw-sidebar) dark:bg-(--sw-sidebar-dark)"></span>
                                        <span class="flex flex-1 flex-col gap-1.5 bg-(--sw-page) p-2 dark:bg-(--sw-page-dark)">
                                            <span class="h-2 w-3/4 rounded-full bg-(--sw-primary) dark:bg-(--sw-primary-dark)"></span>
                                            <span class="h-1.5 w-full rounded-full bg-(--sw-primary) opacity-20 dark:bg-(--sw-primary-dark)"></span>
                                            <span class="h-1.5 w-2/3 rounded-full bg-(--sw-primary) opacity-20 dark:bg-(--sw-primary-dark)"></span>
                                        </span>
                                    </span>
                                    <span class="px-0.5 text-sm font-medium capitalize">(name)</span>
                                </button>
                            }
                        </div>
                    </section>

                    <section class="grid gap-2" aria-labelledby="appearance-sidebar">
                        <h3 id="appearance-sidebar" class="text-sm font-medium">(t("appearance-sidebar"))</h3>
                        <div class="grid grid-cols-3 gap-2">
                            for (name, label, checked) in frames {
                                <button
                                    type="submit"
                                    name="sidebar"
                                    value=(name)
                                    aria-pressed=(if checked { "true" } else { "false" })
                                    class=(PICK)
                                >
                                    frame_preview(name: name)
                                    <span class="px-0.5 text-sm font-medium">(label)</span>
                                </button>
                            }
                        </div>
                    </section>

                    <div class="mt-auto grid gap-2 border-t border-border pt-4">
                        <button type="submit" name="reset" value="1" class=(reset)>
                            (t("appearance-reset"))
                        </button>
                        <p class="text-xs text-muted-foreground">(t("appearance-note"))</p>
                    </div>
                </form>
            </dialog>
        }))
    }
}

/// A pick from the drawer: a theme, a frame, or neither to go back to the panel's own.
#[derive(Deserialize)]
struct Pick {
    csrf_token: String,
    back: String,
    theme: Option<String>,
    sidebar: Option<String>,
}

/// `POST /appearance` — store a pick from the drawer, then reopen it on the page it came from.
#[route(POST "/appearance")]
async fn pick_appearance(cx: &Cx, Form(pick): Form<Pick>) -> Result<(HeaderMap, SeeOther)> {
    tablo::csrf::verify(cx, Some(&pick.csrf_token))?;
    // Back to a page of this site only: `//host` and `/\host` leave it.
    let local = pick.back.starts_with('/') && !pick.back[1..].starts_with(['/', '\\']);
    if !local {
        return Err(forbidden().into());
    }
    let theme_known = pick
        .theme
        .as_deref()
        .is_none_or(|value| THEMES.iter().any(|choice| choice.name == value));
    let frame_known = pick
        .sidebar
        .as_deref()
        .is_none_or(|value| FRAMES.iter().any(|(name, ..)| *name == value));
    if !theme_known || !frame_known {
        return Err(forbidden().into());
    }
    // A theme brings its own frame, so picking one drops the frame picked before; a pick of
    // neither is the reset, which drops both.
    let (theme, sidebar) = match (pick.theme, pick.sidebar) {
        (Some(theme), _) => (Some(theme), None),
        (None, Some(sidebar)) => (cookie(cx, THEME_COOKIE), Some(sidebar)),
        (None, None) => (None, None),
    };
    let mut headers = HeaderMap::new();
    for (name, value) in [
        (THEME_COOKIE, theme),
        (SIDEBAR_COOKIE, sidebar),
        (OPEN_COOKIE, Some("1".to_owned())),
    ] {
        let mut cookie = Cookie::build((name, value.clone().unwrap_or_default()))
            .path("/")
            .http_only(true)
            .same_site(SameSite::Lax);
        cookie = match value {
            Some(_) => cookie.max_age(topcoat::cookie::time::Duration::days(365)),
            None => cookie.max_age(topcoat::cookie::time::Duration::ZERO),
        };
        headers.append(
            SET_COOKIE,
            HeaderValue::try_from(cookie.build().to_string())?,
        );
    }
    Ok((headers, see_other(pick.back)))
}

/// A choice in the drawer: a preview over its name, ringed in the primary once it is the one the
/// page wears.
const PICK: &str = "grid gap-1.5 rounded-lg border border-border p-1.5 text-left transition-colors outline-none hover:bg-foreground/5 focus-visible:ring-2 focus-visible:ring-ring aria-pressed:border-primary aria-pressed:ring-1 aria-pressed:ring-primary";

/// The custom properties a theme's preview paints with, in both schemes.
fn swatch_style(choice: &ThemeChoice) -> String {
    let ThemeChoice { light, dark, .. } = choice;
    format!(
        "--sw-page:{};--sw-sidebar:{};--sw-primary:{};--sw-page-dark:{};--sw-sidebar-dark:{};--sw-primary-dark:{}",
        light.page, light.sidebar, light.primary, dark.page, dark.sidebar, dark.primary,
    )
}

/// A sketch of the frame `name`, in the colors of the theme the page wears.
#[component]
async fn frame_preview(cx: &Cx, name: &'static str) -> Result<impl View> {
    let (outer, sidebar, page) = match name {
        // The sidebar flush with the edge, ruled off from the page.
        "ruled" => (
            "bg-background",
            "w-1/3 border-r border-sidebar-border bg-sidebar",
            "flex-1 bg-background",
        ),
        // The page a sheet inset in the sidebar's color.
        "inset" => (
            "bg-sidebar",
            "w-1/3 bg-sidebar",
            "my-1 mr-1 flex-1 rounded-sm bg-background shadow-xs ring-1 ring-sidebar-border",
        ),
        // The sidebar a card floating beside the page.
        _ => (
            "bg-background",
            "my-1 ml-1 w-1/3 rounded-sm border border-sidebar-border bg-sidebar shadow-xs",
            "flex-1 bg-background",
        ),
    };
    Ok(view! {
        <span aria-hidden="true" class=(format!("flex h-12 overflow-hidden rounded-md border border-border {outer}"))>
            <span class=(sidebar)></span>
            <span class=(page)></span>
        </span>
    })
}
