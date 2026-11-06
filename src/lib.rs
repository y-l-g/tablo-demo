//! The Tablo demo: a shop, a blog, HR and projects, run by staff from the `admin` panel, an `app`
//! panel customers sign up on, and a `partners` panel the demo signs in on its own page.
//!
//! Each business domain is a module that owns its tables, its resources and its dashboard:
//! [`shop`], [`blog`], [`hr`] and [`projects`]. Around them, [`accounts`] holds who signs in,
//! [`access`] what each staff role may do there, [`audit`] the row every write leaves, and
//! [`partners`] the sign-in the `partners` panel trusts. [`panels`] mounts it all, [`router`] serves the panels beside the public [`welcome`] page, and
//! [`seed`] fills the database on every start.

pub mod access;
pub mod accounts;
pub mod audit;
pub mod blog;
pub mod config;
pub mod home;
pub mod hr;
pub mod mailbox;
pub mod panels;
pub mod partners;
pub mod projects;
pub mod router;
pub mod seed;
pub mod shared;
pub mod shop;
pub mod welcome;

/// Every table the demo stores, for the database to create.
pub fn models() -> toasty::ModelSet {
    tablo::models!(
        accounts::User,
        accounts::Team,
        accounts::TeamUser,
        audit::AuditEntry,
        partners::PartnerSession,
        shop::Brand,
        shop::ProductCategory,
        shop::Product,
        shop::ProductCategoryLink,
        shop::Customer,
        shop::Address,
        shop::Order,
        shop::OrderItem,
        shop::OrderAddress,
        shop::Payment,
        blog::Author,
        blog::PostCategory,
        blog::Tag,
        blog::Post,
        blog::PostTag,
        blog::Comment,
        hr::Department,
        hr::Employee,
        hr::LeaveRequest,
        hr::Expense,
        hr::ExpenseLine,
        projects::Project,
        projects::Task,
        projects::Timesheet
    )
}
