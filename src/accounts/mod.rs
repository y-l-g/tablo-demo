//! Who signs in: the demo's users and teams, read through Tablo's authenticator.
//!
//! [`DemoAuth`] signs staff in to the `admin` panel and anyone in to the `app` panel, where an
//! account also signs up, resets its password, verifies its address, edits its profile, and
//! creates and renames its teams. What a staff role may do once in is [`access`](crate::access)'.

mod demo;
mod models;
mod profile;
mod sign_in;
mod teams;

pub use demo::{ADMIN_EMAIL, ADMIN_NAME, ADMIN_PASSWORD, STAFF, StaffAccount};
pub use models::{StaffRole, Team, TeamUser, User};
pub use profile::{DemoResetter, DemoVerifier};
pub use sign_in::{DemoAuth, SignUp, SignedIn};
pub use teams::{DemoTenantRegistrar, NewTeam};
