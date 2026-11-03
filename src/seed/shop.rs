//! The shop's data: brands, customers and their addresses, categories, products and orders.

use std::collections::HashMap;

use jiff::Zoned;
use rust_decimal::Decimal;
use tablo::rich_text::from_text;
use toasty::Db;

use super::random::{
    Rng, date_of, days_ago, growth_age, hours_ago, money, person, phone, round, slug, today_back,
};
use crate::blog::Comment;
use crate::shop::{
    Address, Brand, CITIES, CountryCode, CurrencyCode, Customer, Order, OrderAddress, OrderItem,
    OrderStatus, Payment, PaymentMethod, Product, ProductCategory, ProductCategoryLink,
    ProductType,
};

/// How many customers and orders the shop holds.
const CUSTOMERS: u64 = 600;
const ORDERS: u64 = 600;

/// Brands with their addresses, customers with theirs, and product categories.
pub(super) async fn seed_shop(
    db: &mut Db,
    rng: &mut Rng,
    now: &Zoned,
) -> toasty::Result<(Vec<Brand>, Vec<Customer>)> {
    let mut brands = Vec::new();
    for (index, name) in BRANDS.iter().enumerate() {
        let brand = toasty::create!(Brand {
            name: name.to_string(),
            slug: slug(name),
            website: Some(format!("https://{}.example.com", slug(name))),
            description: Some(from_text(&format!("{name} makes everyday goods."))),
            position: index as u16,
            is_visible: true,
            sort: index as i64,
            created_at: days_ago(now, rng.below(700) as i64),
        })
        .exec(db)
        .await?;
        for _ in 0..rng.between(1, 3) {
            let address = AddressFields::draw(rng);
            address.create_for(db, Some(brand.id), None).await?;
        }
        brands.push(brand);
    }

    let mut customers = Vec::new();
    for index in 0..CUSTOMERS {
        let name = person(rng);
        let created_at = hours_ago(now, growth_age(index, CUSTOMERS) * 24.0);
        let customer = toasty::create!(Customer {
            name: name.clone(),
            email: format!("{}.{}@example.com", slug(&name), index + 1),
            phone: Some(phone(rng)),
            birthday: Some(today_back(rng.below(9000) as i64)),
            created_at,
            updated_at: created_at,
        })
        .exec(db)
        .await?;
        for _ in 0..rng.between(1, 3) {
            let address = AddressFields::draw(rng);
            address.create_for(db, None, Some(customer.id)).await?;
        }
        customers.push(customer);
    }

    for (index, name) in PRODUCT_CATEGORIES.iter().enumerate() {
        let parent = toasty::create!(ProductCategory {
            name: name.to_string(),
            slug: slug(name),
            description: Some(format!("Everything we sell in {name}.")),
            position: index as u16,
            is_visible: true,
            created_at: days_ago(now, rng.below(700) as i64),
        })
        .exec(db)
        .await?;
        for name in rng.pick_distinct(SUBCATEGORIES, 3) {
            toasty::create!(ProductCategory {
                parent_id: Some(parent.id),
                name: name.to_string(),
                slug: format!("{}-{}", parent.slug, slug(name)),
                position: rng.below(10) as u16,
                is_visible: true,
                created_at: days_ago(now, rng.below(700) as i64),
            })
            .exec(db)
            .await?;
        }
    }
    Ok((brands, customers))
}

/// Products, filed under three to six categories each.
pub(super) async fn seed_products(
    db: &mut Db,
    rng: &mut Rng,
    brands: &[Brand],
    customers: &[Customer],
    shots: &[String],
) -> toasty::Result<Vec<Product>> {
    let categories: Vec<ProductCategory> = ProductCategory::all().exec(db).await?;
    let mut products = Vec::new();
    for (index, name) in PRODUCTS.iter().enumerate() {
        let draft = ProductDraft {
            name: format!("{} {name}", rng.pick(PRODUCT_QUALIFIERS)),
            price: 9.0 + 290.0 * rng.float(0.0, 1.0).powi(2),
            qty: rng.between(0, 200),
            index,
            image: shots[index % shots.len()].clone(),
        };
        products.push(create_product(db, rng, brands, shots, draft).await?);
    }
    for (offset, (name, price, qty)) in PRICE_ANOMALIES.iter().enumerate() {
        let draft = ProductDraft {
            name: name.to_string(),
            price: *price,
            qty: *qty,
            index: PRODUCTS.len() + offset,
            image: shots[(PRODUCTS.len() + offset) % shots.len()].clone(),
        };
        products.push(create_product(db, rng, brands, shots, draft).await?);
    }

    for product in &products {
        let count = rng.between(3, 6);
        for category in rng.pick_distinct(&categories, count) {
            toasty::create!(ProductCategoryLink {
                product_id: product.id,
                product_category_id: category.id,
            })
            .exec(db)
            .await?;
        }
        for _ in 0..rng.between(1, 3) {
            toasty::create!(Comment {
                customer_id: Some(rng.pick(customers).id),
                product_id: Some(product.id),
                title: Some(rng.pick(REVIEW_TITLES).to_string()),
                content: Some(from_text(rng.pick(REVIEWS))),
                is_visible: rng.boolean(80),
                created_at: days_ago(&Zoned::now(), rng.below(700) as i64),
            })
            .exec(db)
            .await?;
        }
    }
    Ok(products)
}

/// What one seeded product is made of.
struct ProductDraft {
    name: String,
    price: f64,
    qty: u64,
    /// Its place in the demo's product list, which its SKU and barcode spell.
    index: usize,
    image: String,
}

/// One product of `brands`.
async fn create_product(
    db: &mut Db,
    rng: &mut Rng,
    brands: &[Brand],
    shots: &[String],
    draft: ProductDraft,
) -> toasty::Result<Product> {
    let ProductDraft {
        name,
        price,
        qty,
        index,
        image,
    } = draft;
    let brand = rng.pick(brands);
    let created_at = days_ago(&Zoned::now(), rng.below(700) as i64);
    let is_visible = rng.boolean(85);
    let gallery_size = rng.below(4);
    let gallery = rng.pick_distinct(shots, gallery_size);
    toasty::create!(Product {
        brand_id: Some(brand.id),
        name: name.clone(),
        slug: Some(slug(&name)),
        sku: Some(format!("SKU-{:05}", index + 1)),
        barcode: Some(format!("{:013}", 100_000_000_000 + index as u64)),
        description: Some(from_text(&format!(
            "The {name}, made by {} to last for years of everyday use.",
            brand.name
        ))),
        qty,
        security_stock: rng.below(20),
        featured: rng.boolean(20),
        is_visible,
        old_price: Some(money(price * 1.2)),
        price: Some(money(price)),
        cost: Some(money(price * rng.float(0.3, 0.6))),
        kind: Some(if rng.boolean(80) {
            ProductType::Deliverable
        } else {
            ProductType::Downloadable
        }),
        backorder: rng.boolean(30),
        requires_shipping: rng.boolean(75),
        published_at: is_visible.then(|| date_of(created_at)),
        weight_value: Some(round(rng.float(0.1, 20.0))),
        height_value: Some(round(rng.float(1.0, 100.0))),
        width_value: Some(round(rng.float(1.0, 100.0))),
        depth_value: Some(round(rng.float(1.0, 100.0))),
        volume_value: Some(round(rng.float(0.1, 50.0))),
        image,
        gallery,
        created_at,
        updated_at: created_at,
    })
    .exec(db)
    .await
}

/// Orders, each with its lines, its payments and its shipping address. The shop grows: each
/// month brings more orders than the one before, and an order moves along the pipeline as it
/// ages, a few stuck where the pipeline flags them.
pub(super) async fn seed_orders(
    db: &mut Db,
    rng: &mut Rng,
    now: &Zoned,
    customers: &[Customer],
    products: &[Product],
) -> toasty::Result<()> {
    let mut sold: HashMap<u64, u64> = HashMap::new();
    for index in 0..ORDERS {
        let age = growth_age(index, ORDERS) + rng.float(0.0, 1.0);
        let created_at = hours_ago(now, age * 24.0);
        let status = if rng.boolean(4) {
            OrderStatus::Cancelled
        } else if age < 2.0 || (age < 9.0 && rng.boolean(4)) {
            OrderStatus::New
        } else if age < 5.0 || (age < 20.0 && rng.boolean(3)) {
            OrderStatus::Processing
        } else if age < 10.0 {
            OrderStatus::Shipped
        } else {
            OrderStatus::Delivered
        };
        let (shipping_method, shipping_price) = *rng.pick(SHIPPING_METHODS);

        let mut lines = Vec::new();
        for _ in 0..rng.between(1, 4) {
            let product = rng.pick(products);
            let qty = if rng.boolean(75) {
                1
            } else {
                rng.between(2, 3) as i64
            };
            lines.push((product.id, qty, product.price.unwrap_or(Decimal::from(25))));
        }
        let subtotal: Decimal = lines
            .iter()
            .map(|(_, qty, price)| Decimal::from(*qty) * price)
            .sum();
        let shipping = money(shipping_price);
        let total = subtotal + shipping;

        let order = toasty::create!(Order {
            customer_id: rng.boolean(95).then(|| rng.pick(customers).id),
            number: format!("OR-{:06}", index + 1),
            total_price: Some(total),
            status,
            shipping_price: Some(shipping),
            shipping_method: Some(shipping_method.to_string()),
            created_at,
            updated_at: created_at,
        })
        .exec(db)
        .await?;

        let address = AddressFields::draw(rng);
        address.create_for_order(db, order.id).await?;

        if matches!(status, OrderStatus::Shipped | OrderStatus::Delivered) {
            for (product_id, qty, _) in &lines {
                *sold.entry(*product_id).or_default() += *qty as u64;
            }
        }

        for (sort, (product_id, qty, unit_price)) in lines.into_iter().enumerate() {
            toasty::create!(OrderItem {
                sort: sort as u32,
                order_id: Some(order.id),
                product_id: Some(product_id),
                qty,
                unit_price,
                created_at,
                updated_at: created_at,
            })
            .exec(db)
            .await?;
        }

        if matches!(status, OrderStatus::New | OrderStatus::Cancelled) {
            continue;
        }
        let (provider, method) = if rng.boolean(70) {
            (
                "stripe",
                *rng.pick(&[
                    PaymentMethod::CreditCard,
                    PaymentMethod::CreditCard,
                    PaymentMethod::ApplePay,
                    PaymentMethod::GooglePay,
                ]),
            )
        } else {
            ("paypal", PaymentMethod::Paypal)
        };
        toasty::create!(Payment {
            order_id: order.id,
            reference: format!("PAY-{:08}", rng.below(100_000_000)),
            provider: provider.to_string(),
            method,
            amount: total,
            currency: CurrencyCode::Usd,
            created_at,
            updated_at: created_at,
        })
        .exec(db)
        .await?;
    }
    for (product_id, units) in sold {
        Product::filter_by_id(product_id)
            .update()
            .units_sold(units)
            .exec(db)
            .await?;
    }
    Ok(())
}

/// The five fields an address is made of, wherever it is stored.
struct AddressFields {
    country: Option<CountryCode>,
    street: Option<String>,
    city: Option<String>,
    state: Option<String>,
    zip: Option<String>,
}

impl AddressFields {
    /// Draws one address.
    fn draw(rng: &mut Rng) -> Self {
        Self {
            country: Some(*rng.pick(&[
                CountryCode::Us,
                CountryCode::Gb,
                CountryCode::De,
                CountryCode::Fr,
                CountryCode::Ca,
                CountryCode::Au,
                CountryCode::Nl,
                CountryCode::Br,
                CountryCode::Jp,
                CountryCode::In,
            ])),
            street: Some(format!("{} {}", rng.below(9999), rng.pick(STREETS))),
            city: Some(rng.pick(CITIES).to_string()),
            state: Some(rng.pick(STATES).to_string()),
            zip: Some(format!("{:05}", rng.below(100_000))),
        }
    }

    /// Stores this address for `brand` or `customer`.
    async fn create_for(
        self,
        db: &mut Db,
        brand_id: Option<u64>,
        customer_id: Option<u64>,
    ) -> toasty::Result<Address> {
        toasty::create!(Address {
            brand_id,
            customer_id,
            country: self.country,
            street: self.street,
            city: self.city,
            state: self.state,
            zip: self.zip,
        })
        .exec(db)
        .await
    }

    /// Stores this address as `order`'s shipping address.
    async fn create_for_order(self, db: &mut Db, order_id: u64) -> toasty::Result<OrderAddress> {
        toasty::create!(OrderAddress {
            order_id: Some(order_id),
            country: self.country,
            street: self.street,
            city: self.city,
            state: self.state,
            zip: self.zip,
        })
        .exec(db)
        .await
    }
}

const BRANDS: &[&str] = &[
    "Atlas",
    "Bramble",
    "Copperfield",
    "Driftwood",
    "Everline",
    "Fernway",
    "Granite",
    "Harborlight",
    "Ironwood",
    "Juniper",
    "Kestrel",
    "Lantern",
    "Meridian",
    "Northwind",
    "Orchard",
    "Pinecrest",
    "Quarry",
    "Riverstone",
    "Sablewood",
    "Thornbury",
];

const PRODUCTS: &[&str] = &[
    "Backpack",
    "Baking Mat",
    "Belt",
    "Blanket",
    "Bluetooth Speaker",
    "Bottle Opener",
    "Candle Set",
    "Card Holder",
    "Carafe",
    "Charging Pad",
    "Chef Knife",
    "Coaster Set",
    "Coffee Mug",
    "Cutting Board",
    "Desk Lamp",
    "Desk Organizer",
    "Duffel Bag",
    "Flashlight",
    "French Press",
    "Garden Kit",
    "Hammock",
    "Headphones",
    "Hiking Boots",
    "Hoodie",
    "Jump Rope",
    "Kitchen Scale",
    "Laptop Stand",
    "Lunch Box",
    "Notebook",
    "Phone Mount",
    "Pillow",
    "Running Shoes",
    "Serving Board",
    "Skillet",
    "Socks",
    "Sunglasses",
    "T-Shirt",
    "Thermos",
    "Tote Bag",
    "Travel Mug",
    "Tumbler",
    "Umbrella",
    "Vase",
    "Wallet",
    "Water Bottle",
    "Wine Tote",
    "Wristwatch",
    "Yoga Mat",
];

const PRODUCT_QUALIFIERS: &[&str] = &[
    "Classic",
    "Everyday",
    "Compact",
    "Signature",
    "Trail",
    "Studio",
    "Heritage",
    "Alpine",
];

const PRICE_ANOMALIES: &[(&str, f64, u64)] = &[
    ("Summit Pack", 450.0, 85),
    ("Alpine Duffel", 480.0, 60),
    ("Clearance Bottle", 12.0, 3),
    ("Clearance Towel", 8.0, 2),
    ("Studio Lamp", 250.0, 70),
];

const PRODUCT_CATEGORIES: &[&str] = &[
    "Accessories",
    "Apparel",
    "Bags",
    "Bath",
    "Bedroom",
    "Camping",
    "Cleaning",
    "Electronics",
    "Fitness",
    "Garden",
    "Kitchen",
    "Lighting",
    "Office",
    "Outdoor",
    "Pets",
    "Stationery",
    "Storage",
    "Tools",
    "Toys",
    "Wellness",
];

const SUBCATEGORIES: &[&str] = &[
    "New arrivals",
    "Best sellers",
    "On sale",
    "Gift ideas",
    "Limited edition",
    "Essentials",
];

/// The ways an order ships, and what each costs.
const SHIPPING_METHODS: &[(&str, f64)] = &[
    ("Standard", 4.99),
    ("Standard", 4.99),
    ("Express", 14.99),
    ("Pickup", 0.0),
    ("International", 24.99),
];

const REVIEW_TITLES: &[&str] = &[
    "Exactly right",
    "Better than expected",
    "Good value",
    "Well made",
];

const REVIEWS: &[&str] = &[
    "Arrived quickly and matches the photos.",
    "Solid build, and the finish has held up after months of use.",
    "Does the job for the price. Would buy again.",
    "Slightly smaller than I expected, but I kept it.",
];

const STREETS: &[&str] = &["Main St", "Oak Ave", "Cedar Lane", "Market St", "Hill Rd"];

const STATES: &[&str] = &["CA", "TX", "CO", "MA", "WA", "IL", "ON", "D"];
