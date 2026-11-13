//! The demo's brand on every panel: its name, a logo for light pages and one for dark ones,
//! served by the app itself, the theme each panel wears, and a favicon in that theme's primary.

use tablo::{Brand, Theme};
use topcoat::{
    Result,
    context::Cx,
    router::{
        HeaderMap, HeaderValue,
        header::{CACHE_CONTROL, CONTENT_TYPE},
        route,
    },
};

/// The URL of the logo shown on light pages.
pub const LOGO_URL: &str = "/brand/logo.svg";

/// The URL of the logo shown while a page renders dark.
pub const DARK_LOGO_URL: &str = "/brand/logo-dark.svg";

/// The demo's brand in `theme`: its name, an ink mark that turns light on a dark page, which
/// sits beside any theme's primary, and the mark filled with that primary as the favicon, so each
/// panel's tab tells which panel it is.
pub(super) fn brand(theme: Theme) -> Brand {
    Brand::new("Tablo Demo")
        .logo(LOGO_URL)
        .dark_logo(DARK_LOGO_URL)
        .favicon(favicon_url(theme))
        .theme(theme)
}

/// The URL of the favicon in `theme`'s primary; the ink logo for a theme without its own.
fn favicon_url(theme: Theme) -> &'static str {
    match theme {
        Theme::Jade => "/brand/favicon-jade.svg",
        Theme::Iris => "/brand/favicon-iris.svg",
        Theme::Amber => "/brand/favicon-amber.svg",
        _ => LOGO_URL,
    }
}

/// `GET /brand/favicon-jade.svg` — the mark in Jade's teal.
#[route(GET "/brand/favicon-jade.svg")]
async fn jade_favicon(cx: &Cx) -> Result<(HeaderMap, &'static str)> {
    svg(include_str!("../../assets/brand/favicon-jade.svg"))
}

/// `GET /brand/favicon-iris.svg` — the mark in Iris's violet.
#[route(GET "/brand/favicon-iris.svg")]
async fn iris_favicon(cx: &Cx) -> Result<(HeaderMap, &'static str)> {
    svg(include_str!("../../assets/brand/favicon-iris.svg"))
}

/// `GET /brand/favicon-amber.svg` — the mark in Amber's orange.
#[route(GET "/brand/favicon-amber.svg")]
async fn amber_favicon(cx: &Cx) -> Result<(HeaderMap, &'static str)> {
    svg(include_str!("../../assets/brand/favicon-amber.svg"))
}

/// `GET /brand/logo.svg` — the ink mark.
#[route(GET "/brand/logo.svg")]
async fn logo(cx: &Cx) -> Result<(HeaderMap, &'static str)> {
    svg(include_str!("../../assets/brand/logo.svg"))
}

/// `GET /brand/logo-dark.svg` — the light mark for a dark background.
#[route(GET "/brand/logo-dark.svg")]
async fn dark_logo(cx: &Cx) -> Result<(HeaderMap, &'static str)> {
    svg(include_str!("../../assets/brand/logo-dark.svg"))
}

/// `body` served as an SVG image a browser may cache for a day.
fn svg(body: &'static str) -> Result<(HeaderMap, &'static str)> {
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("image/svg+xml"));
    headers.insert(
        CACHE_CONTROL,
        HeaderValue::from_static("public, max-age=86400"),
    );
    Ok((headers, body))
}
