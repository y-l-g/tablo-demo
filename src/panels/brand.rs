//! The demo's brand on every panel: its name, a logo for light pages and one for dark ones,
//! served by the app itself, and the theme each panel wears.

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

/// The URL of the logo shown on light pages, which is also the favicon.
pub const LOGO_URL: &str = "/brand/logo.svg";

/// The URL of the logo shown while a page renders dark.
pub const DARK_LOGO_URL: &str = "/brand/logo-dark.svg";

/// The demo's brand in `theme`: its name, and an ink mark that turns light on a dark page, which
/// sits beside any theme's primary.
pub(super) fn brand(theme: Theme) -> Brand {
    Brand::new("Tablo Demo")
        .logo(LOGO_URL)
        .dark_logo(DARK_LOGO_URL)
        .favicon(LOGO_URL)
        .theme(theme)
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
