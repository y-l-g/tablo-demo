//! Who may do what in the `admin` panel: each staff role's access to each part of the business.
//!
//! | Role          | Shop | Blog | HR   | Projects | Pay | Audit log   |
//! | ------------- | ---- | ---- | ---- | -------- | --- | ----------- |
//! | Administrator | full | full | full | full     | yes | yes         |
//! | Shop manager  | full | full | none | read     | no  | no          |
//! | HR manager    | none | read | full | full     | yes | no          |
//! | Viewer        | read | read | read | read     | no  | no          |
//!
//! The welcome page draws this table from the same functions.

use tablo::{Ability, Policy, auth};
use topcoat::context::Cx;

use crate::accounts::{SignedIn, StaffRole};

/// The part of the business a resource belongs to, as its sidebar group names it.
#[derive(Debug, Clone, Copy)]
pub enum Domain {
    Shop,
    Blog,
    Hr,
    Projects,
}

impl Domain {
    /// Every domain, in the sidebar's order.
    pub const ALL: [Domain; 4] = [Domain::Shop, Domain::Blog, Domain::Hr, Domain::Projects];

    /// The message that names the domain, its sidebar group's.
    pub fn label(self) -> &'static str {
        match self {
            Domain::Shop => "navigation-shop",
            Domain::Blog => "navigation-blog",
            Domain::Hr => "navigation-hr",
            Domain::Projects => "navigation-projects",
        }
    }
}

/// How far a role reaches into a domain.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Access {
    None,
    Read,
    Full,
}

impl StaffRole {
    /// How far the role reaches into `domain`.
    pub fn access(self, domain: Domain) -> Access {
        use {Access::*, Domain::*, StaffRole::*};
        match (self, domain) {
            (Administrator, _) => Full,
            (ShopManager, Shop | Blog) => Full,
            (ShopManager, Projects) => Read,
            (ShopManager, Hr) => None,
            (HrManager, Hr | Projects) => Full,
            (HrManager, Blog) => Read,
            (HrManager, Shop) => None,
            (Viewer, _) => Read,
        }
    }

    /// Whether the role sees what employees are paid.
    pub fn sees_pay(self) -> bool {
        matches!(self, StaffRole::Administrator | StaffRole::HrManager)
    }

    /// Whether the role reads the audit log and acts as a colleague.
    pub fn administers(self) -> bool {
        self == StaffRole::Administrator
    }
}

/// The signed-in user's staff role, `None` for a customer or a visitor.
pub fn role(cx: &Cx) -> Option<StaffRole> {
    auth::user::<SignedIn>(cx).and_then(|signed_in| signed_in.user.role)
}

/// The policy of a resource in `domain`: every ability under full access, the reads under read
/// access, and nothing otherwise, the sidebar entry included.
pub fn policy<M: 'static>(domain: Domain) -> impl Policy<M> {
    move |cx: &Cx, ability: Ability<'_, M>| match role(cx).map(|role| role.access(domain)) {
        Some(Access::Full) => true,
        Some(Access::Read) => ability.is_read(),
        Some(Access::None) | None => false,
    }
}

/// Whether the user may open a domain at all, for the pages and widgets that read it.
pub fn reaches(cx: &Cx, domain: Domain) -> bool {
    role(cx).is_some_and(|role| role.access(domain) != Access::None)
}

/// Whether the user may change the records of a domain, which only full access allows.
pub fn may_write(cx: &Cx, domain: Domain) -> bool {
    role(cx).is_some_and(|role| role.access(domain) == Access::Full)
}

/// Whether the user sees what employees are paid: administrators and HR managers.
pub fn sees_pay(cx: &Cx) -> bool {
    role(cx).is_some_and(StaffRole::sees_pay)
}

/// Whether the user is an administrator, who alone reads the audit log and acts as a colleague.
pub fn is_administrator(cx: &Cx) -> bool {
    role(cx).is_some_and(StaffRole::administers)
}
