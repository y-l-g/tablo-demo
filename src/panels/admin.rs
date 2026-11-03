//! The `admin` panel: every resource of the demo, for staff alone.

use tablo::{Auth, Panel};

use crate::accounts::DemoAuth;
use crate::audit::{Audit, AuditTrail};
use crate::config::Config;
use crate::home::Dashboard;
use crate::shared::uploads;
use crate::{blog, hr, projects, shop};

/// The admin panel: the home, the domains' dashboards, the audit trail, then each domain's
/// resources.
pub fn admin_panel(config: &Config) -> Panel {
    super::shared(Panel::new("admin"))
        .auth(Auth::custom(DemoAuth))
        .notifications()
        // Every write of every resource below leaves a row in the audit trail.
        .before_commit::<Audit>()
        .home::<Dashboard>()
        .page::<shop::ShopDashboard>()
        .page::<hr::HrDashboard>()
        .resource::<AuditTrail>()
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
