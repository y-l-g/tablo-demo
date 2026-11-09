//! The `admin` panel: every resource of the demo, for staff alone.

use tablo::{Auth, Panel, when};
use topcoat::icon::iconify::iconify_icon;

use crate::access;
use crate::accounts::{self, DemoAuth, StaffPage};
use crate::config::Config;
use crate::home::Dashboard;
use crate::shared::uploads;
use crate::{blog, hr, projects, shop};

/// The admin panel: the home, the domains' dashboards, the staff an administrator acts as, the
/// audit log, then each domain's resources.
pub fn admin_panel(config: &Config) -> Panel {
    super::shared(Panel::new("admin"))
        .auth(Auth::custom(DemoAuth).impersonation(accounts::impersonation()))
        // An address's "Show on map" lands on OpenStreetMap, which a form may follow only to an
        // origin the policy's `form-action` names.
        .content_security_policy_source("form-action", shop::MAP_ORIGIN)
        .notifications()
        // Every write of every resource below leaves an entry naming the fields it changed, and
        // a write made while acting as a colleague names the administrator behind it.
        .audit_log(|log| {
            log.icon(iconify_icon!("lucide:rotate-ccw-clock"))
                .navigation_order(-1)
                .policy(when(access::is_administrator))
        })
        .home::<Dashboard>()
        .page::<shop::ShopDashboard>()
        .page::<hr::HrDashboard>()
        .page::<StaffPage>()
        // The sidebar sorts its entries by navigation order, ties keeping the order they are
        // registered in, and a group sits where its first entry lands; registering the domains
        // one after the other keeps their groups in this order.
        .resource::<shop::ProductResource>()
        .resource::<shop::OrderResource>()
        .resource::<shop::CustomerResource>()
        .resource::<shop::BrandResource>()
        .resource::<shop::ProductCategoryResource>()
        .resource::<shop::OrderItemResource>()
        .resource::<shop::OrderAddressResource>()
        .resource::<shop::PaymentResource>()
        .resource::<shop::AddressResource>()
        .resource::<hr::EmployeeResource>()
        .resource::<hr::DepartmentResource>()
        .resource::<hr::LeaveRequestResource>()
        .resource::<hr::ExpenseResource>()
        .resource::<hr::ExpenseLineResource>()
        .resource::<projects::ProjectResource>()
        .resource::<projects::TaskResource>()
        .resource::<projects::TimesheetResource>()
        .resource::<blog::PostResource>()
        .resource::<blog::PostCategoryResource>()
        .resource::<blog::AuthorResource>()
        .resource::<blog::TagResource>()
        .resource::<blog::CommentResource>()
        .uploads_dir(uploads::URL_PREFIX, config.upload_dir.clone())
}
