//! The `partners` panel: the blog's authors and categories, to read, for an account the demo signs
//! in on its own page.

use tablo::{Auth, Page, PageDef, Panel, auth};
use topcoat::{
    Result,
    context::Cx,
    icon::iconify::iconify_icon,
    view::{View, view},
};

use crate::accounts::SignedIn;
use crate::blog;
use crate::partners::PartnerSignIn;

/// The partner panel, whose users the host app names. The account it admits reads and changes
/// nothing else: [`access`](crate::access) holds it to reading.
pub fn partners_panel() -> Panel {
    super::shared(Panel::new("partners"))
        .brand(super::brand::partner_brand())
        .auth(Auth::external(PartnerSignIn))
        .home::<PartnerHome>()
        .resource::<blog::PostCategoryResource>()
        .resource::<blog::AuthorResource>()
}

/// Who the host app says is signed in, and where that came from.
pub struct PartnerHome;

impl Page for PartnerHome {
    fn declare() -> PageDef {
        PageDef::new()
            .label(tablo::text!("page-partner-home"))
            .icon(iconify_icon!("lucide:layout-dashboard"))
            .navigation_order(-2)
    }

    async fn render(cx: &Cx) -> Result<impl View> {
        let name = auth::user::<SignedIn>(cx).map(|signed| signed.user.name.clone());
        let welcome = tablo::t!(cx, "copy-partner-welcome", name = name.unwrap_or_default());
        Ok(view! {
            tablo::ui::page(
                tablo::ui::page_header(
                    tablo::ui::page_title((tablo::t!(cx, "page-partner-home")))
                    tablo::ui::page_description((welcome))
                )
                tablo::ui::page_content(
                    <p class="text-sm text-pretty">(tablo::t!(cx, "copy-partner-body"))</p>
                )
            )
        })
    }
}
