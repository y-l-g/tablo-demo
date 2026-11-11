//! Who signs in: the demo's users and teams, read through Tablo's authenticator.
//!
//! [`password_auth`] signs staff in to the `admin` panel and anyone in to the `app` panel, where an
//! account also signs up, resets its password, verifies its address, edits its profile, and
//! creates and renames its teams. What a staff role may do once in is [`access`](crate::access)',
//! and [`StaffPage`] lets the administrator act as each other member of staff.

mod demo;
mod models;
mod sign_in;
mod staff;
mod teams;

pub use demo::{ADMIN_EMAIL, ADMIN_NAME, ADMIN_PASSWORD, PARTNER_EMAIL, STAFF, StaffAccount};
pub use models::{StaffRole, Team, TeamUser, User};
pub use sign_in::{DemoSignUp, SignUp, SignedIn, password_auth};
pub use staff::{StaffPage, impersonation};
pub use teams::{DemoTenantRegistrar, NewTeam};
