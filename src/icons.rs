//! One Lucide icon per sidebar entry, from the set `build.rs` stages.

use topcoat::icon::{IconData, iconify::iconify_icon};

pub const WELCOME: IconData = iconify_icon!("lucide:layout-dashboard");
pub const AUDIT_TRAIL: IconData = iconify_icon!("lucide:rotate-ccw-clock");
pub const SHOP_DASHBOARD: IconData = iconify_icon!("lucide:store");
pub const HR_DASHBOARD: IconData = iconify_icon!("lucide:briefcase");
pub const SETTINGS: IconData = iconify_icon!("lucide:settings");
pub const MAILBOX: IconData = iconify_icon!("lucide:mail");

pub const PRODUCTS: IconData = iconify_icon!("lucide:package");
pub const ORDERS: IconData = iconify_icon!("lucide:shopping-cart");
pub const CUSTOMERS: IconData = iconify_icon!("lucide:users");
pub const BRANDS: IconData = iconify_icon!("lucide:bookmark");
pub const PRODUCT_CATEGORIES: IconData = iconify_icon!("lucide:tag");
pub const COMMENTS: IconData = iconify_icon!("lucide:message-square");
pub const ORDER_ITEMS: IconData = iconify_icon!("lucide:clipboard-list");
pub const PAYMENTS: IconData = iconify_icon!("lucide:credit-card");
pub const ADDRESSES: IconData = iconify_icon!("lucide:map-pin");
pub const ORDER_ADDRESSES: IconData = iconify_icon!("lucide:truck");

pub const PROJECTS: IconData = iconify_icon!("lucide:folder");
pub const TASKS: IconData = iconify_icon!("lucide:list-checks");
pub const TIMESHEETS: IconData = iconify_icon!("lucide:clock");

pub const POSTS: IconData = iconify_icon!("lucide:file-text");
pub const POST_CATEGORIES: IconData = iconify_icon!("lucide:tags");
pub const AUTHORS: IconData = iconify_icon!("lucide:pen-line");

pub const EMPLOYEES: IconData = iconify_icon!("lucide:id-card");
pub const DEPARTMENTS: IconData = iconify_icon!("lucide:building");
pub const LEAVE_REQUESTS: IconData = iconify_icon!("lucide:calendar-days");
pub const EXPENSES: IconData = iconify_icon!("lucide:banknote");
pub const EXPENSE_LINES: IconData = iconify_icon!("lucide:receipt");
