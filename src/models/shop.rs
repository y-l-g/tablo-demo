//! The shop: brands, categories, products, customers, orders and payments.

use jiff::{Timestamp, civil::Date};
use toasty::Deferred;

use super::{Address, Comment};
use crate::enums::{CountryCode, CurrencyCode, OrderStatus, PaymentMethod, ProductType};

/// A maker of products.
#[derive(Debug, Clone, toasty::Model, tablo::ModelColumns)]
pub struct Brand {
    #[key]
    #[auto]
    pub id: u64,
    pub name: String,
    #[unique]
    pub slug: String,
    pub website: Option<String>,
    pub description: Option<String>,
    #[default(0)]
    pub position: u16,
    #[default(false)]
    pub is_visible: bool,
    pub seo_title: Option<String>,
    pub seo_description: Option<String>,
    /// The brand's place in the list's manual order.
    pub sort: Option<i64>,
    #[default(Timestamp::now())]
    pub created_at: Timestamp,
    #[update(Timestamp::now())]
    pub updated_at: Timestamp,
    #[has_many]
    pub products: Deferred<Vec<Product>>,
    #[has_many]
    pub addresses: Deferred<Vec<Address>>,
}

/// A product category, which may sit under another one.
#[derive(Debug, Clone, toasty::Model, tablo::ModelColumns)]
pub struct ProductCategory {
    #[key]
    #[auto]
    pub id: u64,
    #[index]
    pub parent_id: Option<u64>,
    #[belongs_to(key = parent_id, references = id)]
    pub parent: Deferred<Option<ProductCategory>>,
    pub name: String,
    #[unique]
    pub slug: String,
    pub description: Option<String>,
    #[default(0)]
    pub position: u16,
    #[default(false)]
    pub is_visible: bool,
    pub seo_title: Option<String>,
    pub seo_description: Option<String>,
    #[default(Timestamp::now())]
    pub created_at: Timestamp,
    #[update(Timestamp::now())]
    pub updated_at: Timestamp,
    #[has_many(pair = parent)]
    pub children: Deferred<Vec<ProductCategory>>,
    #[has_many(pair = category)]
    pub links: Deferred<Vec<ProductCategoryLink>>,
    #[has_many(via = links.product)]
    pub products: Deferred<Vec<Product>>,
}

/// A product.
#[derive(Debug, Clone, toasty::Model, tablo::ModelColumns)]
pub struct Product {
    #[key]
    #[auto]
    pub id: u64,
    #[index]
    pub brand_id: Option<u64>,
    #[belongs_to(key = brand_id, references = id)]
    pub brand: Deferred<Option<Brand>>,
    pub name: String,
    #[unique]
    pub slug: Option<String>,
    #[unique]
    pub sku: Option<String>,
    #[unique]
    pub barcode: Option<String>,
    pub description: Option<String>,
    #[default(0)]
    pub qty: u64,
    #[default(0)]
    pub security_stock: u64,
    #[default(false)]
    pub featured: bool,
    #[default(false)]
    pub is_visible: bool,
    pub old_price: Option<f64>,
    pub price: Option<f64>,
    pub cost: Option<f64>,
    #[column("type")]
    pub kind: Option<ProductType>,
    #[default(false)]
    pub backorder: bool,
    #[default(false)]
    pub requires_shipping: bool,
    pub published_at: Option<Date>,
    pub seo_title: Option<String>,
    pub seo_description: Option<String>,
    pub weight_value: Option<f64>,
    #[default("kg".to_string())]
    pub weight_unit: String,
    pub height_value: Option<f64>,
    #[default("cm".to_string())]
    pub height_unit: String,
    pub width_value: Option<f64>,
    #[default("cm".to_string())]
    pub width_unit: String,
    pub depth_value: Option<f64>,
    #[default("cm".to_string())]
    pub depth_unit: String,
    pub volume_value: Option<f64>,
    #[default("l".to_string())]
    pub volume_unit: String,
    /// The product's image, where Spatie's media library holds a collection.
    #[default(String::new())]
    pub image: String,
    #[default(Timestamp::now())]
    pub created_at: Timestamp,
    #[update(Timestamp::now())]
    pub updated_at: Timestamp,
    #[has_many]
    pub links: Deferred<Vec<ProductCategoryLink>>,
    #[has_many(via = links.category)]
    pub categories: Deferred<Vec<ProductCategory>>,
    #[has_many]
    pub comments: Deferred<Vec<Comment>>,
}

/// Files a product under a category: the join model of the two.
#[derive(Debug, Clone, toasty::Model, tablo::ModelColumns)]
#[key(product_id, product_category_id)]
pub struct ProductCategoryLink {
    #[index]
    pub product_id: u64,
    #[belongs_to(key = product_id, references = id)]
    pub product: Deferred<Product>,
    #[index]
    pub product_category_id: u64,
    #[belongs_to(key = product_category_id, references = id)]
    pub category: Deferred<ProductCategory>,
}

/// Someone who buys from the shop.
#[derive(Debug, Clone, toasty::Model, tablo::ModelColumns)]
pub struct Customer {
    #[key]
    #[auto]
    pub id: u64,
    pub name: String,
    #[unique]
    pub email: String,
    pub photo: Option<String>,
    pub phone: Option<String>,
    pub birthday: Option<Date>,
    /// Set by a delete, which hides the row instead of removing it.
    pub deleted_at: Option<Timestamp>,
    #[default(Timestamp::now())]
    pub created_at: Timestamp,
    #[update(Timestamp::now())]
    pub updated_at: Timestamp,
    #[has_many]
    pub addresses: Deferred<Vec<Address>>,
    #[has_many]
    pub comments: Deferred<Vec<Comment>>,
    #[has_many]
    pub orders: Deferred<Vec<Order>>,
}

/// One line of an order.
#[derive(Debug, Clone, toasty::Model, tablo::ModelColumns)]
pub struct OrderItem {
    #[key]
    #[auto]
    pub id: u64,
    /// The line's place in the order, which the form's repeater sets.
    #[default(0)]
    pub sort: u32,
    #[index]
    pub order_id: Option<u64>,
    #[belongs_to(key = order_id, references = id)]
    pub order: Deferred<Option<Order>>,
    #[index]
    pub product_id: Option<u64>,
    #[belongs_to(key = product_id, references = id)]
    pub product: Deferred<Option<Product>>,
    pub qty: i64,
    pub unit_price: f64,
    #[default(Timestamp::now())]
    pub created_at: Timestamp,
    #[update(Timestamp::now())]
    pub updated_at: Timestamp,
}

/// Where an order ships.
#[derive(Debug, Clone, toasty::Model, tablo::ModelColumns)]
pub struct OrderAddress {
    #[key]
    #[auto]
    pub id: u64,
    #[index]
    pub order_id: Option<u64>,
    #[belongs_to(key = order_id, references = id)]
    pub order: Deferred<Option<Order>>,
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

/// A customer's order.
#[derive(Debug, Clone, toasty::Model, tablo::ModelColumns)]
pub struct Order {
    #[key]
    #[auto]
    pub id: u64,
    #[index]
    pub customer_id: Option<u64>,
    #[belongs_to(key = customer_id, references = id)]
    pub customer: Deferred<Option<Customer>>,
    #[unique]
    pub number: String,
    pub total_price: Option<f64>,
    #[default(OrderStatus::New)]
    pub status: OrderStatus,
    pub shipping_price: Option<f64>,
    pub shipping_method: Option<String>,
    pub notes: Option<String>,
    /// Set by a delete, which hides the row instead of removing it.
    pub deleted_at: Option<Timestamp>,
    #[default(Timestamp::now())]
    pub created_at: Timestamp,
    #[update(Timestamp::now())]
    pub updated_at: Timestamp,
    #[has_one]
    pub address: Deferred<Option<OrderAddress>>,
    #[has_many]
    pub items: Deferred<Vec<OrderItem>>,
    #[has_many]
    pub payments: Deferred<Vec<Payment>>,
}

/// A payment against an order.
#[derive(Debug, Clone, toasty::Model, tablo::ModelColumns)]
pub struct Payment {
    #[key]
    #[auto]
    pub id: u64,
    #[index]
    pub order_id: u64,
    #[belongs_to(key = order_id, references = id)]
    pub order: Deferred<Order>,
    pub reference: String,
    pub provider: String,
    pub method: PaymentMethod,
    pub amount: f64,
    pub currency: CurrencyCode,
    #[default(Timestamp::now())]
    pub created_at: Timestamp,
    #[update(Timestamp::now())]
    pub updated_at: Timestamp,
}
