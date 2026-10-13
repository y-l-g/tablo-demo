//! The demo's tables, one Toasty model each.

pub mod blog;
pub mod hr;
pub mod shop;
pub mod staff;

use jiff::Timestamp;
use toasty::Deferred;

use crate::enums::CountryCode;

pub use blog::{Author, Post, PostCategory};
pub use hr::{Department, Employee, Expense, ExpenseLine, LeaveRequest, Project, Task, Timesheet};
pub use shop::{
    Brand, Customer, Order, OrderAddress, OrderItem, Payment, Product, ProductCategory,
    ProductCategoryLink,
};
pub use staff::{Team, TeamUser, User};

/// A postal address, owned by the customer or the brand whose column names it.
///
/// The Filament demo links addresses through a polymorphic `addressables` pivot;
/// `customer_id` and `brand_id` are the two nullable keys that replace it.
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
/// The Filament demo links comments through a polymorphic `commentable`; the
/// two nullable keys below replace it, so each side keeps its own relation.
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

/// How a money column renders: two decimals behind a dollar sign.
pub fn money(amount: &f64) -> String {
    format!("${amount:.2}")
}

/// How a whole-dollar amount renders.
pub fn dollars(amount: f64) -> String {
    format!("${amount:.0}")
}

/// How a nullable money column renders: a dash stands in for nothing stored.
pub fn maybe_money(amount: &Option<f64>) -> String {
    amount.map_or_else(|| "—".to_string(), |amount| money(&amount))
}
