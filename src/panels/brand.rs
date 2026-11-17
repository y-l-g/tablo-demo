//! The demo's brand on every panel: its name, Tablo's mark served by the app itself, and the
//! theme each panel wears.

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

/// The URL of Tablo's mark: the logo of every panel, its favicon, and the mark on `/`.
pub const LOGO_URL: &str = "/brand/logo.svg";

/// The demo's brand in `theme`: its name and Tablo's mark, as logo and favicon alike. The mark
/// keeps its own colours in every theme and scheme, so it reads as the same mark on the front
/// page, in each panel, and on Tablo's site.
pub(super) fn brand(theme: Theme) -> Brand {
    Brand::new("Tablo Demo")
        .logo(LOGO_URL)
        .favicon(LOGO_URL)
        .theme(theme)
}

/// `GET /brand/logo.svg` — Tablo's mark.
#[route(GET "/brand/logo.svg")]
async fn logo(cx: &Cx) -> Result<(HeaderMap, &'static str)> {
    svg(include_str!("../../assets/brand/logo.svg"))
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
