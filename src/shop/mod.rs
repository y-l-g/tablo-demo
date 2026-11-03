//! The shop: products, their brands and categories, the customers who buy them, and the orders
//! they place.
//!
//! Its resources form the sidebar's Shop group. A customer's or a brand's addresses, and an
//! order's lines, payments and shipping address, are tabs on their owner's page, so their
//! resources have no sidebar entry of their own.

mod address;
mod brand;
mod category;
mod customer;
mod dashboard;
mod models;
mod order;
mod product;

pub use address::{AddressResource, CITIES};
pub use brand::BrandResource;
pub use category::ProductCategoryResource;
pub use customer::{CustomerResource, customer_name};
pub use dashboard::{RevenueWidget, ShopDashboard};
pub use models::{
    Address, Brand, CountryCode, CurrencyCode, Customer, Order, OrderAddress, OrderItem,
    OrderStatus, Payment, PaymentMethod, Product, ProductCategory, ProductCategoryLink,
    ProductType,
};
pub use order::{
    OrderAddressResource, OrderItemResource, OrderResource, PaymentResource, PipelineWidget,
};
pub use product::ProductResource;
