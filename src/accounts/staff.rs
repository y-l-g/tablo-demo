//! Acting as a colleague: the administrator opens the `admin` panel as another member of staff sees
//! it, to check what their role reaches before they report it.
//!
//! Tablo's `Impersonate` action needs a resource over the panel's user type, and [`SignedIn`] is no
//! model, so [`StaffPage`] renders `auth::impersonate_button` for each colleague instead.

use std::time::Duration;

use tablo::{Impersonation, Page, PageDef, auth, db::db};
use topcoat::{
    Result,
    context::Cx,
    icon::iconify::iconify_icon,
    view::{View, view},
};

use super::{SignedIn, StaffRole, User};
use crate::access;

/// Who may act as whom on the `admin` panel: an administrator as any other member of staff, for
/// half an hour. Another administrator reaches nothing more, so acting as one shows nothing new.
pub fn impersonation() -> Impersonation<SignedIn> {
    Impersonation::new(|_cx, actor: &SignedIn, target: &SignedIn| {
        actor.user.role == Some(StaffRole::Administrator)
            && target
                .user
                .role
                .is_some_and(|role| role != StaffRole::Administrator)
    })
    .lifetime(Duration::from_secs(30 * 60))
}

/// The staff an administrator may act as, each with a button that does.
pub struct StaffPage;

impl Page for StaffPage {
    fn declare() -> PageDef {
        PageDef::new()
            .slug("staff")
            .label(tablo::text!("page-staff"))
            .icon(iconify_icon!("lucide:users"))
            .navigation_order(-1)
            // The rule refuses everyone else anyway; the page only spares them a list of buttons
            // that would answer 403.
            .access(|cx| {
                access::is_administrator(cx) && auth::impersonator::<SignedIn>(cx).is_none()
            })
    }

    async fn render(cx: &Cx) -> Result<impl View> {
        let mut db = db(cx)?;
        let mut staff: Vec<User> = User::all().exec(&mut db).await?;
        staff.retain(|user| {
            user.role
                .is_some_and(|role| role != StaffRole::Administrator)
        });
        staff.sort_by_key(|user| user.id);
        let rows: Vec<_> = staff
            .into_iter()
            .map(|user| {
                let role = tablo::t!(cx, user.role.map(role_label).unwrap_or_default());
                let (name, email) = (user.name.clone(), user.email.clone());
                // The button asks the panel's rule, so it reads the colleague as the rule does.
                let colleague = SignedIn {
                    user,
                    teams: Vec::new(),
                };
                (name, email, role, auth::impersonate_button(cx, &colleague))
            })
            .collect();
        Ok(view! {
            tablo::ui::page(
                tablo::ui::page_header(
                    tablo::ui::page_title((tablo::t!(cx, "page-staff")))
                    tablo::ui::page_description((tablo::t!(cx, "page-staff-description")))
                )
                tablo::ui::page_content(
                    tablo::ui::table(
                        tablo::ui::table_header(
                            tablo::ui::table_row(
                                tablo::ui::table_head((tablo::t!(cx, "staff-name")))
                                tablo::ui::table_head((tablo::t!(cx, "staff-email")))
                                tablo::ui::table_head((tablo::t!(cx, "staff-role")))
                                tablo::ui::table_head()
                            )
                        )
                        tablo::ui::table_body(
                            for (name, email, role, button) in rows {
                                tablo::ui::table_row(
                                    tablo::ui::table_cell(<span class="font-medium">(name)</span>)
                                    tablo::ui::table_cell((email))
                                    tablo::ui::table_cell((role))
                                    tablo::ui::table_cell(<div class="flex justify-end">(button)</div>)
                                )
                            }
                        )
                    )
                )
            )
        })
    }
}

/// The message naming `role`, as the welcome page names it.
fn role_label(role: StaffRole) -> &'static str {
    match role {
        StaffRole::Administrator => "home-role-administrator",
        StaffRole::ShopManager => "home-role-shop-manager",
        StaffRole::HrManager => "home-role-hr-manager",
        StaffRole::Viewer => "home-role-viewer",
    }
}
