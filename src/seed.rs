//! The demo data.

use std::collections::HashMap;

use jiff::{Span, Timestamp, Zoned, civil::Date};
use rust_decimal::Decimal;
use serde_json::json;
use tablo::rich_text::from_text;
use toasty::Db;
use uuid::Uuid;

use crate::auth::create_user;
use crate::enums::{
    CountryCode, CurrencyCode, EmploymentType, ExpenseCategory, ExpenseStatus, LeaveStatus,
    LeaveType, OrderStatus, PaymentMethod, ProductType, ProjectStatus, StaffRole, TaskPriority,
    TaskStatus,
};
use crate::models::{
    Address, Author, Brand, Comment, Customer, Department, Employee, Expense, ExpenseLine,
    LeaveRequest, MetadataEntry, Order, OrderAddress, OrderItem, Payment, Post, PostCategory,
    PostTag, Product, ProductCategory, ProductCategoryLink, Project, Tag, Task, Team, TeamUser,
    Timesheet,
};
use crate::models::{cost, money, round};
use crate::panel::{UPLOAD_URL_PREFIX, upload_dir};

/// The demo administrator's name.
pub const ADMIN_NAME: &str = "Demo User";

/// The demo administrator's email address.
pub const ADMIN_EMAIL: &str = "admin@example.com";

/// The demo administrator's password.
pub const ADMIN_PASSWORD: &str = "password";

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

/// Fills an empty database with the demo's rows.
pub async fn seed(db: &mut Db) -> toasty::Result<()> {
    let mut rng = Rng::new(0x5eed);
    let now = Zoned::now();
    let today = now.date();

    let admin = create_user(
        db,
        ADMIN_NAME,
        ADMIN_EMAIL,
        ADMIN_PASSWORD,
        Some(StaffRole::Administrator),
    )
    .await?;
    let mut staff = Vec::new();
    for account in &STAFF {
        staff.push(
            create_user(
                db,
                account.name,
                account.email,
                ADMIN_PASSWORD,
                Some(account.role),
            )
            .await?,
        );
    }
    let teams = seed_teams(db).await?;
    let shots = upload_all(0, PRODUCT_SHOTS).await?;
    let covers = upload_all(PRODUCT_SHOTS.len(), POST_COVERS).await?;
    let (brands, customers) = seed_shop(db, &mut rng, &now).await?;
    let products = seed_products(db, &mut rng, &brands, &customers, &shots).await?;
    seed_orders(db, &mut rng, &now, &customers, &products).await?;
    seed_blog(db, &mut rng, &now, &customers, &covers).await?;
    seed_hr(db, &mut rng, &now, today).await?;

    // The demo's `team_user` pivot. The seeded administrator joins both teams, so
    // the app panel's switcher has something to switch, and the rest of the staff
    // the first; an account that signs up joins none until it registers one.
    for team in &teams {
        toasty::create!(TeamUser {
            team_id: team.id,
            user_id: admin.id,
        })
        .exec(db)
        .await?;
    }
    for user in &staff {
        toasty::create!(TeamUser {
            team_id: teams[0].id,
            user_id: user.id,
        })
        .exec(db)
        .await?;
    }
    Ok(())
}

/// Writes each of `images` to the upload directory and returns the paths the panel serves them
/// at. A file's name is fixed by its place, from `first` on, so each start writes over the files
/// the last one wrote.
async fn upload_all(first: usize, images: &[(&str, &[u8])]) -> toasty::Result<Vec<String>> {
    let dir = upload_dir();
    tokio::fs::create_dir_all(&dir).await?;
    let mut paths = Vec::new();
    for (offset, (name, bytes)) in images.iter().enumerate() {
        let file = format!("{}-{name}", Uuid::from_u128((first + offset) as u128 + 1));
        tokio::fs::write(dir.join(&file), bytes).await?;
        paths.push(format!("{UPLOAD_URL_PREFIX}/{file}"));
    }
    Ok(paths)
}

/// The teams the `app` panel switches between.
async fn seed_teams(db: &mut Db) -> toasty::Result<Vec<Team>> {
    let mut teams = Vec::new();
    for name in ["Acme Inc.", "Globex"] {
        teams.push(
            toasty::create!(Team {
                name: name.to_string(),
            })
            .exec(db)
            .await?,
        );
    }
    Ok(teams)
}

/// Brands with their addresses, customers with theirs, and product categories.
async fn seed_shop(
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
async fn seed_products(
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
async fn seed_orders(
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
        Product::filter(Product::fields().id().eq(product_id))
            .update()
            .units_sold(units)
            .exec(db)
            .await?;
    }
    Ok(())
}

/// Post categories, authors, their posts, and the comments on them.
async fn seed_blog(
    db: &mut Db,
    rng: &mut Rng,
    now: &Zoned,
    customers: &[Customer],
    covers: &[String],
) -> toasty::Result<()> {
    let mut categories = Vec::new();
    for name in POST_CATEGORIES {
        categories.push(
            toasty::create!(PostCategory {
                name: name.to_string(),
                slug: slug(name),
                description: Some(format!("Posts about {name}.")),
                is_visible: rng.boolean(85),
                seo_title: Some(name.to_string()),
                seo_description: Some(format!("{name} articles from the demo blog.")),
                created_at: days_ago(now, rng.below(700) as i64),
            })
            .exec(db)
            .await?,
        );
    }

    let mut tags = Vec::new();
    for name in TAGS {
        tags.push(
            toasty::create!(Tag {
                name: name.to_string(),
            })
            .exec(db)
            .await?,
        );
    }

    let mut authors = Vec::new();
    for index in 0..AUTHORS {
        let name = person(rng);
        let handle = slug(&name);
        let created_at = days_ago(now, rng.between(400, 720) as i64);
        authors.push(
            toasty::create!(Author {
                name: name.clone(),
                email: format!("{handle}.{}@example.com", index + 1),
                bio: Some(format!("{name} writes about building software.")),
                github_handle: Some(handle.clone()),
                twitter_handle: Some(handle),
                created_at,
                updated_at: created_at,
            })
            .exec(db)
            .await?,
        );
    }

    // About one post a week, the newest still drafts and one scheduled for next week.
    for (index, draft) in POSTS.iter().enumerate() {
        let age = ((POSTS.len() - index) * 7) as i64 - 4 + rng.below(3) as i64;
        let created_at = days_ago(now, age);
        let published_at = match POSTS.len() - index {
            1 => Some(today_back(-7)),
            2 => None,
            _ => Some(date_of(created_at)),
        };
        let category = categories
            .iter()
            .find(|category| category.name == draft.category)
            .unwrap_or(&categories[0]);
        let tag_count = rng.between(2, 4);
        let picked = rng.pick_distinct(TAGS, tag_count);
        let post = toasty::create!(Post {
            author_id: Some(rng.pick(&authors).id),
            post_category_id: Some(category.id),
            title: draft.title.to_string(),
            slug: slug(draft.title),
            content: from_text(draft.content),
            published_at,
            image: covers[index % covers.len()].clone(),
            seo_title: Some(draft.title.to_string()),
            seo_description: Some(draft.content.chars().take(120).collect()),
            created_at,
            updated_at: created_at,
        })
        .exec(db)
        .await?;

        for name in picked {
            let tag = tags
                .iter()
                .find(|tag| tag.name == name)
                .expect("every seeded tag is created");
            toasty::create!(PostTag {
                post_id: post.id,
                tag_id: tag.id,
            })
            .exec(db)
            .await?;
        }

        for _ in 0..rng.between(1, 4) {
            toasty::create!(Comment {
                customer_id: Some(rng.pick(customers).id),
                post_id: Some(post.id),
                title: Some(rng.pick(REPLY_TITLES).to_string()),
                content: Some(rng.pick(REPLIES).to_string()),
                is_visible: rng.boolean(85),
                created_at: days_ago(now, rng.below(age.max(1) as u64) as i64),
            })
            .exec(db)
            .await?;
        }
    }
    Ok(())
}

/// Departments, employees and everything the HR pages list.
async fn seed_hr(db: &mut Db, rng: &mut Rng, now: &Zoned, today: Date) -> toasty::Result<()> {
    let mut departments = Vec::new();
    for name in DEPARTMENT_NAMES {
        departments.push(
            toasty::create!(Department {
                name: name.to_string(),
                slug: slug(name),
                description: Some(format!("The {name} team.")),
                budget: money(rng.float(50_000.0, 500_000.0)),
                headcount_limit: rng.between(5, 40) as i64,
                color: Some(hex_color(rng)),
                is_active: true,
                created_at: days_ago(now, rng.below(700) as i64),
            })
            .exec(db)
            .await?,
        );
    }
    for name in SUBDEPARTMENTS {
        let parent = rng.pick(&departments);
        let (parent_id, parent_slug, parent_name) =
            (parent.id, parent.slug.clone(), parent.name.clone());
        departments.push(
            toasty::create!(Department {
                parent_id: Some(parent_id),
                name: name.to_string(),
                slug: format!("{parent_slug}-{}", slug(name)),
                description: Some(format!("{name} at {parent_name}.")),
                budget: money(rng.float(20_000.0, 200_000.0)),
                headcount_limit: rng.between(2, 15) as i64,
                color: Some(hex_color(rng)),
                is_active: rng.boolean(90),
                created_at: days_ago(now, rng.below(700) as i64),
            })
            .exec(db)
            .await?,
        );
    }

    let mut employees = Vec::new();
    for index in 0..EMPLOYEES {
        employees.push(create_employee(db, rng, &departments, index, None).await?);
    }
    // Salaries that stand out beside the rest of the workforce.
    for (offset, salary) in [
        168_000.0, 158_000.0, 45_000.0, 42_000.0, 41_000.0, 170_000.0,
    ]
    .into_iter()
    .enumerate()
    {
        let index = EMPLOYEES + offset as u64;
        employees.push(create_employee(db, rng, &departments, index, Some(salary)).await?);
    }

    for _ in 0..LEAVE_REQUESTS {
        let employee_id = rng.pick(&employees).id;
        let approver_id = rng.boolean(60).then(|| rng.pick(&employees).id);
        let status = *rng.pick(&[
            LeaveStatus::Pending,
            LeaveStatus::Approved,
            LeaveStatus::Rejected,
            LeaveStatus::Taken,
            LeaveStatus::Taken,
            LeaveStatus::Cancelled,
        ]);
        // Leave waiting for a decision or approved lies ahead, leave taken behind.
        let start_date = match status {
            LeaveStatus::Pending | LeaveStatus::Approved => {
                today_back(-(rng.between(3, 120) as i64))
            }
            LeaveStatus::Taken => today_back(rng.between(5, 400) as i64),
            _ => today_back(rng.below(400) as i64 - 60),
        };
        let end_date = start_date
            .checked_add(Span::new().days(rng.below(10) as i64))
            .unwrap_or(start_date);
        toasty::create!(LeaveRequest {
            employee_id,
            approver_id,
            kind: *rng.pick(&[
                LeaveType::Annual,
                LeaveType::Sick,
                LeaveType::Personal,
                LeaveType::Unpaid,
                LeaveType::Parental,
            ]),
            status,
            start_date,
            end_date,
            days_requested: (end_date
                .since(start_date)
                .map(|span| span.get_days())
                .unwrap_or(0)
                + 1) as f64,
            reason: rng.pick(LEAVE_REASONS).to_string(),
            reviewer_notes: (status != LeaveStatus::Pending)
                .then(|| rng.pick(REVIEW_NOTES).to_string()),
            reviewed_at: (status != LeaveStatus::Pending).then(Timestamp::now),
            created_at: days_ago(now, rng.below(400) as i64),
        })
        .exec(db)
        .await?;
    }

    let mut projects = Vec::new();
    for name in PROJECT_NAMES {
        let budget = rng.float(10_000.0, 500_000.0);
        let estimated_hours = round(rng.float(100.0, 2000.0));
        let start_date = today_back(rng.below(180) as i64);
        projects.push(
            toasty::create!(Project {
                department_id: Some(rng.pick(&departments).id),
                name: name.to_string(),
                slug: slug(name),
                description: Some(format!("{name} is a company-wide initiative.")),
                color: Some(hex_color(rng)),
                status: *rng.pick(&[
                    ProjectStatus::Planning,
                    ProjectStatus::Active,
                    ProjectStatus::OnHold,
                    ProjectStatus::Completed,
                    ProjectStatus::Cancelled,
                ]),
                priority: *rng.pick(&[
                    TaskPriority::Low,
                    TaskPriority::Medium,
                    TaskPriority::High,
                    TaskPriority::Critical,
                ]),
                budget: money(budget),
                spent: money(rng.float(0.0, budget * 1.2)),
                estimated_hours,
                actual_hours: round(rng.float(0.0, estimated_hours * 1.3)),
                start_date,
                end_date: rng
                    .boolean(70)
                    .then(|| today_back(-(rng.below(365) as i64))),
                plan: Some(
                    json!([{
                        "type": "milestone",
                        "data": { "title": "Kick-off", "target_date": start_date.to_string() }
                    }])
                    .into(),
                ),
                created_at: days_ago(now, rng.below(180) as i64),
            })
            .exec(db)
            .await?,
        );
    }
    // Budgets that stand out: one barely touched, two spent several times over.
    for (name, budget, spent) in [
        ("Warehouse Overhaul", 400_000.0, 50_000.0),
        ("Checkout Rewrite", 20_000.0, 95_000.0),
        ("Brand Refresh", 15_000.0, 45_000.0),
    ] {
        projects.push(
            toasty::create!(Project {
                department_id: Some(rng.pick(&departments).id),
                name: name.to_string(),
                slug: slug(name),
                status: ProjectStatus::Active,
                priority: TaskPriority::High,
                budget: money(budget),
                spent: money(spent),
                estimated_hours: 500.0,
                actual_hours: 250.0,
                start_date: today,
                created_at: days_ago(now, rng.below(180) as i64),
            })
            .exec(db)
            .await?,
        );
    }

    let mut tasks = Vec::new();
    for index in 0..TASKS {
        let project_id = rng.pick(&projects).id;
        let label_count = rng.between(1, 3);
        let labels = rng.pick_distinct(TASK_LABELS, label_count);
        tasks.push(
            toasty::create!(Task {
                project_id,
                assigned_to: rng.boolean(70).then(|| rng.pick(&employees).id),
                title: rng.pick(TASK_TITLES).to_string(),
                description: Some(rng.pick(TASK_DESCRIPTIONS).to_string()),
                status: *rng.pick(&[
                    TaskStatus::Backlog,
                    TaskStatus::Todo,
                    TaskStatus::InProgress,
                    TaskStatus::InReview,
                    TaskStatus::Completed,
                    TaskStatus::Cancelled,
                ]),
                priority: *rng.pick(&[
                    TaskPriority::Low,
                    TaskPriority::Medium,
                    TaskPriority::High,
                    TaskPriority::Critical,
                ]),
                estimated_hours: Some(round(rng.float(1.0, 40.0))),
                actual_hours: round(rng.float(0.0, 45.0)),
                due_date: rng.boolean(80).then(|| today_back(rng.below(120) as i64)),
                labels: labels.clone(),
                sort: index as i64,
                created_at: days_ago(now, rng.below(200) as i64),
                updated_at: Timestamp::now(),
            })
            .exec(db)
            .await?,
        );
    }

    for _ in 0..TIMESHEETS {
        let employee = rng.pick(&employees);
        let (employee_id, hourly_rate) = (employee.id, employee.hourly_rate);
        let project_id = rng.pick(&projects).id;
        let hours = round(rng.float(0.5, 9.0) * 2.0) / 2.0;
        let hourly_rate = hourly_rate.unwrap_or_else(|| money(rng.float(50.0, 200.0)));
        toasty::create!(Timesheet {
            employee_id,
            task_id: tasks
                .iter()
                .find(|task| task.project_id == project_id)
                .map(|task| task.id),
            project_id,
            date: today_back(rng.below(90) as i64),
            hours,
            minutes: rng.below(60) as i64,
            description: Some(rng.pick(TIMESHEET_NOTES).to_string()),
            is_billable: rng.boolean(80),
            hourly_rate,
            total_cost: cost(hours, hourly_rate),
            created_at: days_ago(now, rng.below(90) as i64),
        })
        .exec(db)
        .await?;
    }

    for index in 0..EXPENSES {
        let employee_id = rng.pick(&employees).id;
        let status = *rng.pick(&[
            ExpenseStatus::Draft,
            ExpenseStatus::Submitted,
            ExpenseStatus::Approved,
            ExpenseStatus::Rejected,
            ExpenseStatus::Reimbursed,
        ]);
        let created_at = days_ago(now, rng.below(300) as i64);
        let expense = toasty::create!(Expense {
            employee_id,
            project_id: rng.boolean(60).then(|| rng.pick(&projects).id),
            expense_number: format!("EXP-{:06}", index + 1),
            status,
            category: *rng.pick(&[
                ExpenseCategory::Travel,
                ExpenseCategory::Meals,
                ExpenseCategory::Supplies,
                ExpenseCategory::Equipment,
                ExpenseCategory::Software,
                ExpenseCategory::Other,
            ]),
            description: rng.pick(EXPENSE_DESCRIPTIONS).to_string(),
            total_amount: Decimal::ZERO,
            currency: "USD".to_string(),
            submitted_at: (status != ExpenseStatus::Draft).then(Timestamp::now),
            approved_at: matches!(status, ExpenseStatus::Approved | ExpenseStatus::Reimbursed)
                .then(Timestamp::now),
            approved_by: rng.boolean(30).then(|| rng.pick(&employees).id),
            notes: rng.boolean(30).then(|| rng.pick(REVIEW_NOTES).to_string()),
            created_at,
            updated_at: created_at,
        })
        .exec(db)
        .await?;

        let mut total = Decimal::ZERO;
        for _ in 0..rng.between(1, 5) {
            let quantity = rng.between(1, 6) as i64;
            let unit_price = money(rng.float(5.0, 400.0));
            let amount = Decimal::from(quantity) * unit_price;
            total += amount;
            toasty::create!(ExpenseLine {
                expense_id: expense.id,
                description: rng.pick(EXPENSE_LINES).to_string(),
                amount,
                quantity,
                unit_price,
                date: today_back(rng.below(120) as i64),
                created_at,
                updated_at: created_at,
            })
            .exec(db)
            .await?;
        }
        Expense::filter(Expense::fields().id().eq(expense.id))
            .update()
            .total_amount(total)
            .exec(db)
            .await?;
    }
    Ok(())
}

/// One employee of `departments`, at `salary` when the demo's anomalies set one.
async fn create_employee(
    db: &mut Db,
    rng: &mut Rng,
    departments: &[Department],
    index: u64,
    salary: Option<f64>,
) -> toasty::Result<Employee> {
    let name = person(rng);
    let employment_type = *rng.pick(&[
        EmploymentType::FullTime,
        EmploymentType::FullTime,
        EmploymentType::FullTime,
        EmploymentType::PartTime,
        EmploymentType::Contractor,
        EmploymentType::Intern,
    ]);
    let skill_count = rng.between(2, 6);
    let skills = rng.pick_distinct(SKILLS, skill_count);
    toasty::create!(Employee {
        department_id: Some(rng.pick(departments).id),
        name: name.clone(),
        email: format!("{}.{}@example.com", slug(&name), index + 1),
        phone: Some(phone(rng)),
        date_of_birth: Some(today_back(rng.between(9_000, 20_000) as i64)),
        hire_date: today_back(rng.below(1800) as i64),
        job_title: rng.pick(JOB_TITLES).to_string(),
        employment_type,
        salary: salary.map(money).or_else(|| match employment_type {
            EmploymentType::FullTime => Some(money(rng.float(40_000.0, 180_000.0))),
            EmploymentType::PartTime => Some(money(rng.float(20_000.0, 90_000.0))),
            _ => None,
        }),
        hourly_rate: match employment_type {
            EmploymentType::Contractor => Some(money(rng.float(50.0, 200.0))),
            EmploymentType::Intern => Some(money(rng.float(15.0, 30.0))),
            _ => None,
        },
        team_color: Some(hex_color(rng)),
        skills,
        metadata: vec![
            MetadataEntry {
                key: "certification".to_string(),
                value: rng.pick(CERTIFICATIONS).to_string(),
            },
            MetadataEntry {
                key: "office".to_string(),
                value: rng.pick(OFFICES).to_string(),
            },
        ],
        is_active: rng.boolean(90),
        created_at: days_ago(&Zoned::now(), rng.below(700) as i64),
    })
    .exec(db)
    .await
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

/// A small deterministic generator, so the demo data is the same on every run.
struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Self(seed)
    }

    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    /// A number below `n`.
    fn below(&mut self, n: u64) -> u64 {
        if n == 0 { 0 } else { self.next() % n }
    }

    /// A number in `lo..=hi`.
    fn between(&mut self, lo: u64, hi: u64) -> u64 {
        lo + self.below(hi.saturating_sub(lo) + 1)
    }

    /// A number in `lo..hi`.
    fn float(&mut self, lo: f64, hi: f64) -> f64 {
        let unit = (self.next() >> 11) as f64 / (1u64 << 53) as f64;
        lo + unit * (hi - lo)
    }

    /// Whether a `percent` share of answers are true.
    fn boolean(&mut self, percent: u64) -> bool {
        self.below(100) < percent
    }

    /// One of `items`.
    fn pick<'a, T>(&mut self, items: &'a [T]) -> &'a T {
        &items[self.below(items.len() as u64) as usize]
    }

    /// `count` of `items`, each drawn at most once.
    fn pick_distinct<T: Clone>(&mut self, items: &[T], count: u64) -> Vec<T> {
        let count = count.min(items.len() as u64);
        let mut out: Vec<T> = Vec::with_capacity(count as usize);
        let mut taken: Vec<usize> = Vec::new();
        for _ in 0..count {
            let mut index = self.below(items.len() as u64) as usize;
            while taken.contains(&index) {
                index = (index + 1) % items.len();
            }
            taken.push(index);
            out.push(items[index].clone());
        }
        out
    }
}

/// How many days ago the `index`th of `count` records was made, the oldest two years ago, so
/// that more of them fall in each month than in the one before.
fn growth_age(index: u64, count: u64) -> f64 {
    let left = 1.0 - index as f64 / count as f64;
    HISTORY_DAYS * left.powf(1.3)
}

/// A timestamp `hours` before `now`.
fn hours_ago(now: &Zoned, hours: f64) -> Timestamp {
    now.timestamp()
        .checked_sub(Span::new().seconds((hours * 3600.0) as i64))
        .unwrap_or_else(|_| now.timestamp())
}

/// A timestamp `days` before `now`.
fn days_ago(now: &Zoned, days: i64) -> Timestamp {
    now.checked_sub(Span::new().days(days))
        .unwrap_or_else(|_| now.clone())
        .timestamp()
}

/// The day `at` falls on, where the demo runs.
fn date_of(at: Timestamp) -> Date {
    at.to_zoned(jiff::tz::TimeZone::system()).date()
}

/// The date `days` before today, or after it when `days` is negative.
fn today_back(days: i64) -> Date {
    let today = Zoned::now().date();
    today.checked_sub(Span::new().days(days)).unwrap_or(today)
}

/// A URL-safe spelling of `text`.
fn slug(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut last_dash = true;
    for c in text.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
            last_dash = false;
        } else if !last_dash {
            out.push('-');
            last_dash = true;
        }
    }
    out.trim_end_matches('-').to_string()
}

/// A person's name.
fn person(rng: &mut Rng) -> String {
    format!("{} {}", rng.pick(FIRST_NAMES), rng.pick(LAST_NAMES))
}

/// A phone number.
fn phone(rng: &mut Rng) -> String {
    format!(
        "+1 ({:03}) {:03}-{:04}",
        rng.below(1000),
        rng.below(1000),
        rng.below(10_000)
    )
}

/// A hex colour.
fn hex_color(rng: &mut Rng) -> String {
    format!("#{:06x}", rng.below(0x100_0000))
}

/// How far back the shop's history reaches.
const HISTORY_DAYS: f64 = 720.0;

const CUSTOMERS: u64 = 600;
const ORDERS: u64 = 600;
const AUTHORS: u64 = 6;
const EMPLOYEES: u64 = 120;
const LEAVE_REQUESTS: u64 = 300;
const TASKS: u64 = 200;
const TIMESHEETS: u64 = 2000;
const EXPENSES: u64 = 250;

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

const POST_CATEGORIES: &[&str] = &[
    "Announcements",
    "Design",
    "Engineering",
    "Guides",
    "Performance",
    "Product",
    "Security",
    "Testing",
];

/// The tags the seeded posts carry.
pub const TAGS: &[&str] = &[
    "Rust",
    "Async",
    "CSS",
    "DevOps",
    "Testing",
    "API",
    "Security",
    "Performance",
    "Architecture",
    "Git",
    "Database",
    "Accessibility",
    "Deployment",
    "Design",
];

/// Embeds each named image of `assets/seed/{dir}`.
macro_rules! images {
    ($dir:literal: $($name:literal),* $(,)?) => {
        &[$(($name, include_bytes!(concat!("../assets/seed/", $dir, "/", $name)))),*]
    };
}

/// The product shots the seeded products show.
const PRODUCT_SHOTS: &[(&str, &[u8])] = images!("products":
    "product-01.webp", "product-02.webp", "product-03.webp", "product-04.webp",
    "product-05.webp", "product-06.webp", "product-07.webp", "product-08.webp",
    "product-09.webp", "product-10.webp", "product-11.webp", "product-12.webp",
    "product-13.webp", "product-14.webp", "product-15.webp", "product-16.webp",
);

/// The covers the seeded posts show.
const POST_COVERS: &[(&str, &[u8])] = images!("posts":
    "cover-01.webp", "cover-02.webp", "cover-03.webp", "cover-04.webp",
    "cover-05.webp", "cover-06.webp", "cover-07.webp", "cover-08.webp",
);

/// One blog post the seeder writes, newest last.
struct PostDraft {
    title: &'static str,
    category: &'static str,
    content: &'static str,
}

const POSTS: &[PostDraft] = &[
    PostDraft {
        title: "Shipping a Rust Admin Panel in a Weekend",
        category: "Product",
        content: "A list, a form and a detail page per table, declared next to the models they serve. Most of the weekend went into the data, not the screens.",
    },
    PostDraft {
        title: "Why Our Queues Got Faster",
        category: "Performance",
        content: "Slow work such as sending mail now leaves the request. A worker drains the queue, and the page answers before the mail is written.",
    },
    PostDraft {
        title: "Building Accessible Forms",
        category: "Design",
        content: "Label every input, describe every error, and keep the whole form reachable from the keyboard. Accessibility is a requirement, not a polish step.",
    },
    PostDraft {
        title: "Understanding Database Indexes",
        category: "Engineering",
        content: "An index trades write cost for read speed. Index the columns your filters and joins touch, and read the plan before you add another one.",
    },
    PostDraft {
        title: "A Field Guide to Code Review",
        category: "Guides",
        content: "Review the code, not the author. Keep changes small, say why, and answer within a day so the branch does not drift.",
    },
    PostDraft {
        title: "Designing APIs People Enjoy",
        category: "Engineering",
        content: "Name resources with nouns, answer with consistent errors, and paginate every list. Version before the first breaking change, not after.",
    },
    PostDraft {
        title: "Design Tokens Without the Ceremony",
        category: "Design",
        content: "Colours, radii and shadows live in one place. A theme is a handful of variables, and dark mode is the same handful again.",
    },
    PostDraft {
        title: "Testing Strategies That Scale",
        category: "Testing",
        content: "Many fast unit tests, fewer integration tests, and a handful of end-to-end runs that prove the whole path.",
    },
    PostDraft {
        title: "Deploying With Zero Downtime",
        category: "Engineering",
        content: "Run two environments, move traffic once the new one is healthy, and split every schema change across two releases.",
    },
    PostDraft {
        title: "What We Learned From a Year of Async Rust",
        category: "Engineering",
        content: "Hold no lock across an await, keep futures Send, and let the runtime do the scheduling. The rest is ordinary Rust.",
    },
    PostDraft {
        title: "Two-Factor Sign-In, Explained",
        category: "Security",
        content: "An authenticator app or a mailed code, then recovery codes for the day the phone is lost. Each step is a page of its own.",
    },
    PostDraft {
        title: "Charts That Answer a Question",
        category: "Product",
        content: "A chart earns its place when it answers something: is revenue growing, which week was quiet. Twelve weeks is usually enough to tell.",
    },
    PostDraft {
        title: "The Case for Boring Infrastructure",
        category: "Engineering",
        content: "One database, one binary, one container. Interesting problems should come from the product, not from the deploy.",
    },
    PostDraft {
        title: "Writing Release Notes People Read",
        category: "Announcements",
        content: "Lead with what changed for the reader, link the details, and keep the version number out of the first sentence.",
    },
    PostDraft {
        title: "Securing File Uploads",
        category: "Security",
        content: "Check the extension and the size before storing, serve files from signed links that expire, and never trust a filename.",
    },
    PostDraft {
        title: "Spring Release: Tabs, Tags and Charts",
        category: "Announcements",
        content: "Record pages split into tabs, tags became real lists, and dashboards gained line and bar charts over the last twelve weeks.",
    },
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

const REPLY_TITLES: &[&str] = &["Thanks", "Great write-up", "One question"];

const REPLIES: &[&str] = &[
    "This cleared up something I had been getting wrong for years.",
    "Do you have a follow-up on the migration path?",
    "Bookmarked. The section on indexes alone was worth the read.",
];

const DEPARTMENT_NAMES: &[&str] = &["Engineering", "Operations", "Marketing", "People"];

const SUBDEPARTMENTS: &[&str] = &["Frontend", "Backend", "DevOps", "QA", "Content", "SEO"];

/// The job titles the seeded employees hold, and the forms suggest.
pub const JOB_TITLES: &[&str] = &[
    "Software Engineer",
    "Senior Engineer",
    "Designer",
    "Product Manager",
    "Data Analyst",
    "Support Specialist",
    "Account Manager",
    "Technical Writer",
    "QA Engineer",
    "DevOps Engineer",
];

/// The skills the seeded employees claim, and the employee form suggests.
pub const SKILLS: &[&str] = &[
    "Rust",
    "TypeScript",
    "React",
    "Python",
    "SQL",
    "Docker",
    "Kubernetes",
    "AWS",
    "Figma",
    "Go",
];

const CERTIFICATIONS: &[&str] = &["AWS Certified", "PMP", "Scrum Master", "None"];

const OFFICES: &[&str] = &["New York", "San Francisco", "London", "Remote"];

const LEAVE_REASONS: &[&str] = &[
    "Family holiday booked in the spring.",
    "Recovering from a cold.",
    "Moving house this week.",
    "Conference attendance.",
    "Parental leave for a new arrival.",
];

const REVIEW_NOTES: &[&str] = &[
    "Approved, enjoy the time off.",
    "Cover arranged with the team.",
    "Please resubmit with a shorter window.",
];

const PROJECT_NAMES: &[&str] = &[
    "Atlas Migration",
    "Beacon Analytics",
    "Compass Billing",
    "Delta Onboarding",
    "Ember Design",
    "Fathom Search",
    "Granite Platform",
    "Harbor Support",
    "Ivory Reporting",
    "Jasper Mobile",
    "Keystone Identity",
    "Lumen Marketing",
    "Meridian Data",
    "Nimbus Infra",
    "Onyx Checkout",
    "Pillar Content",
    "Quartz Metrics",
    "Ridge Logistics",
    "Summit Portal",
    "Tide Payments",
];

const TASK_TITLES: &[&str] = &[
    "Draft the schema",
    "Wire the endpoint",
    "Review the copy",
    "Add the empty state",
    "Instrument the flow",
    "Backfill the table",
    "Tune the query",
    "Write the migration",
    "Polish the form",
    "Ship the changelog",
];

const TASK_DESCRIPTIONS: &[&str] = &[
    "Break the work into steps a reviewer can follow.",
    "Confirm the behaviour on a fresh database.",
    "Check the empty and error states before merging.",
];

/// The labels the seeded tasks carry, and the task form suggests.
pub const TASK_LABELS: &[&str] = &[
    "bug",
    "feature",
    "enhancement",
    "documentation",
    "design",
    "testing",
    "refactor",
    "urgent",
];

const TIMESHEET_NOTES: &[&str] = &[
    "Pairing on the importer.",
    "Reviewing pull requests.",
    "Writing tests for the new endpoint.",
    "Investigating a slow query.",
];

const EXPENSE_DESCRIPTIONS: &[&str] = &[
    "Client visit",
    "Team offsite",
    "Home office setup",
    "Conference ticket",
    "Software licence",
];

const EXPENSE_LINES: &[&str] = &[
    "Flight",
    "Hotel",
    "Taxi",
    "Meal",
    "Monitor",
    "Keyboard",
    "Licence",
    "Course fee",
];

const STREETS: &[&str] = &["Main St", "Oak Ave", "Cedar Lane", "Market St", "Hill Rd"];

/// The cities the seeded addresses are in, and the forms suggest.
pub const CITIES: &[&str] = &[
    "Portland", "Austin", "Denver", "Boston", "Seattle", "Chicago", "Toronto", "Dublin",
];

const STATES: &[&str] = &["CA", "TX", "CO", "MA", "WA", "IL", "ON", "D"];

const FIRST_NAMES: &[&str] = &[
    "Ada", "Alan", "Grace", "Linus", "Margaret", "Ken", "Barbara", "Dennis", "Radia", "Vint",
    "Anita", "Tim", "Sophie", "Marcus", "Elena", "Priya", "Tomas", "Nadia", "Omar", "Lena", "Hugo",
    "Ines", "Jonas", "Maya", "Nils", "Olga", "Pablo", "Quinn", "Rosa", "Sven",
];

const LAST_NAMES: &[&str] = &[
    "Lovelace",
    "Turing",
    "Hopper",
    "Torvalds",
    "Hamilton",
    "Thompson",
    "Liskov",
    "Ritchie",
    "Perlman",
    "Cerf",
    "Borg",
    "Berners-Lee",
    "Wilson",
    "Okafor",
    "Novak",
    "Silva",
    "Haddad",
    "Bergman",
    "Costa",
    "Dubois",
];
