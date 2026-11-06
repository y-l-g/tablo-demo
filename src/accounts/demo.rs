//! The demo's accounts: the seed creates them, the login hint names the administrator's, and
//! the welcome page signs in as each.

use super::StaffRole;

/// The demo administrator's name.
pub const ADMIN_NAME: &str = "Demo User";

/// The demo administrator's email address.
pub const ADMIN_EMAIL: &str = "admin@example.com";

/// The demo administrator's password.
pub const ADMIN_PASSWORD: &str = "password";

/// The account the host app's sign-in page signs in to the `partners` panel: the seeded viewer,
/// who changes nothing wherever they read.
pub const PARTNER_EMAIL: &str = "viewer@example.com";

/// A seeded member of staff besides the administrator, who signs in with the same password.
pub struct StaffAccount {
    pub name: &'static str,
    pub email: &'static str,
    pub role: StaffRole,
}

/// The staff the welcome page signs in as, one per role but the administrator's, whose account
/// is [`ADMIN_EMAIL`]'s.
pub const STAFF: [StaffAccount; 3] = [
    StaffAccount {
        name: "Sam Rivera",
        email: "shop@example.com",
        role: StaffRole::ShopManager,
    },
    StaffAccount {
        name: "Jordan Blake",
        email: "hr@example.com",
        role: StaffRole::HrManager,
    },
    StaffAccount {
        name: "Robin Ellis",
        email: "viewer@example.com",
        role: StaffRole::Viewer,
    },
];
