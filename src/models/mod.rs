//! The demo's tables, one Toasty model each.

pub mod audit;
pub mod blog;
pub mod hr;
pub mod shop;
pub mod staff;

use jiff::Timestamp;
use rust_decimal::{Decimal, prelude::ToPrimitive};
use toasty::Deferred;

use crate::enums::CountryCode;

pub use audit::AuditEntry;
pub use blog::{Author, Post, PostCategory, PostTag, Tag};
pub use hr::{
    Department, Employee, Expense, ExpenseLine, LeaveRequest, MetadataEntry, Project, Task,
    Timesheet,
};
pub use shop::{
    Brand, Customer, Order, OrderAddress, OrderItem, Payment, Product, ProductCategory,
    ProductCategoryLink,
};
pub use staff::{Team, TeamUser, User};

/// A postal address, owned by the customer or the brand whose column names it.
///
/// Toasty has no polymorphic relation, so `customer_id` and `brand_id` are two
/// nullable keys, one per owner.
#[derive(Debug, Clone, toasty::Model, tablo::ModelColumns)]
pub struct Address {
    #[key]
    #[auto]
    pub id: u64,
    #[index]
    pub customer_id: Option<u64>,
    #[belongs_to(key = customer_id, references = id)]
    pub customer: Deferred<Option<Customer>>,
    #[index]
    pub brand_id: Option<u64>,
    #[belongs_to(key = brand_id, references = id)]
    pub brand: Deferred<Option<Brand>>,
    pub country: Option<CountryCode>,
    pub street: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub zip: Option<String>,
    #[default(Timestamp::now())]
    pub created_at: Timestamp,
    #[update(Timestamp::now())]
    pub updated_at: Timestamp,
}

/// A customer's review of a post or a product.
///
/// Toasty has no polymorphic relation, so each of the two nullable keys below
/// links a comment to one owner, and each side keeps its own relation.
#[derive(Debug, Clone, toasty::Model, tablo::ModelColumns)]
pub struct Comment {
    #[key]
    #[auto]
    pub id: u64,
    #[index]
    pub customer_id: Option<u64>,
    #[belongs_to(key = customer_id, references = id)]
    pub customer: Deferred<Option<Customer>>,
    #[index]
    pub post_id: Option<u64>,
    #[belongs_to(key = post_id, references = id)]
    pub post: Deferred<Option<Post>>,
    #[index]
    pub product_id: Option<u64>,
    #[belongs_to(key = product_id, references = id)]
    pub product: Deferred<Option<Product>>,
    pub title: Option<String>,
    pub content: Option<String>,
    #[default(false)]
    pub is_visible: bool,
    #[default(Timestamp::now())]
    pub created_at: Timestamp,
    #[update(Timestamp::now())]
    pub updated_at: Timestamp,
}

/// A measure rounded to the two decimals a value of its kind holds.
pub fn round(amount: f64) -> f64 {
    (amount * 100.0).round() / 100.0
}

/// A money amount from a float, rounded to the two decimals a price holds.
pub fn money(amount: f64) -> Decimal {
    Decimal::from_f64_retain(amount)
        .unwrap_or_default()
        .round_dp(2)
}

/// A money amount as a float, for the dashboards' figures and charts.
pub fn float(amount: Decimal) -> f64 {
    amount.to_f64().unwrap_or_default()
}

/// What `hours` of work at `rate` an hour cost, rounded to the cent.
pub fn cost(hours: f64, rate: Decimal) -> Decimal {
    (Decimal::from_f64_retain(hours).unwrap_or_default() * rate).round_dp(2)
}

/// Five characters that tell a replica's slug or number apart from its original's.
pub fn replica_suffix() -> String {
    const DIGITS: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let mut n = Timestamp::now().as_nanosecond().unsigned_abs();
    (0..5)
        .map(|_| {
            let digit = DIGITS[(n % 36) as usize] as char;
            n /= 36;
            digit
        })
        .collect()
}
