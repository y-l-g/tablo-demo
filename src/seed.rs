//! The demo data, mirroring the Filament demo's seeder.

use jiff::{Span, Timestamp, Zoned, civil::Date};
use serde_json::json;
use tablo::rich_text::from_text;
use toasty::Db;

use crate::auth::create_user;
use crate::enums::{
    CountryCode, CurrencyCode, EmploymentType, ExpenseCategory, ExpenseStatus, LeaveStatus,
    LeaveType, OrderStatus, PaymentMethod, ProductType, ProjectStatus, TaskPriority, TaskStatus,
};
use crate::models::round;
use crate::models::{
    Address, Author, Brand, Comment, Customer, Department, Employee, Expense, ExpenseLine,
    LeaveRequest, Order, OrderAddress, OrderItem, Payment, Post, PostCategory, Product,
    ProductCategory, ProductCategoryLink, Project, Task, Team, TeamUser, Timesheet,
};

/// The demo administrator's email address.
pub const ADMIN_EMAIL: &str = "admin@example.com";

/// The demo administrator's password.
pub const ADMIN_PASSWORD: &str = "password";

/// Fills an empty database with the demo's rows.
pub async fn seed(db: &mut Db) -> toasty::Result<()> {
    let mut rng = Rng::new(0x5eed);
    let now = Zoned::now();
    let today = now.date();

    let admin = create_user(db, "Demo User", ADMIN_EMAIL, ADMIN_PASSWORD).await?;
    let teams = seed_teams(db).await?;
    let (brands, customers) = seed_shop(db, &mut rng, &now).await?;
    let products = seed_products(db, &mut rng, &brands, &customers, today).await?;
    seed_orders(db, &mut rng, &now, &customers, &products).await?;
    seed_blog(db, &mut rng, &now, &customers, today).await?;
    seed_hr(db, &mut rng, &now, today).await?;

    // The demo's `team_user` pivot: every user belongs to every team.
    for team in &teams {
        toasty::create!(TeamUser {
            team_id: team.id,
            user_id: admin.id,
        })
        .exec(db)
        .await?;
    }
    Ok(())
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
            sort: Some(index as i64),
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
        let created_at = days_ago(now, 720 - (index as i64 * 720 / CUSTOMERS as i64));
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
    today: Date,
) -> toasty::Result<Vec<Product>> {
    let categories: Vec<ProductCategory> = ProductCategory::all().exec(db).await?;
    let mut products = Vec::new();
    for (index, name) in PRODUCTS.iter().enumerate() {
        let draft = ProductDraft {
            name,
            price: rng.float(8.0, 500.0),
            qty: rng.between(0, 200),
            index,
        };
        products.push(create_product(db, rng, brands, today, draft).await?);
    }
    for (offset, (name, price, qty)) in PRICE_ANOMALIES.iter().enumerate() {
        let draft = ProductDraft {
            name,
            price: *price,
            qty: *qty,
            index: PRODUCTS.len() + offset,
        };
        products.push(create_product(db, rng, brands, today, draft).await?);
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
struct ProductDraft<'a> {
    name: &'a str,
    price: f64,
    qty: u64,
    /// Its place in the demo's product list, which its SKU and barcode spell.
    index: usize,
}

/// One product of `brands`.
async fn create_product(
    db: &mut Db,
    rng: &mut Rng,
    brands: &[Brand],
    today: Date,
    draft: ProductDraft<'_>,
) -> toasty::Result<Product> {
    let ProductDraft {
        name,
        price,
        qty,
        index,
    } = draft;
    let brand = rng.pick(brands);
    let name = format!("{name} {}", rng.pick(PRODUCT_QUALIFIERS));
    let created_at = days_ago(&Zoned::now(), rng.below(700) as i64);
    toasty::create!(Product {
        brand_id: Some(brand.id),
        name: name.clone(),
        slug: Some(slug(&name)),
        sku: Some(format!("SKU-{:05}", index + 1)),
        barcode: Some(format!("{:013}", 100_000_000_000 + index as u64)),
        description: Some(format!("A {name} from {}.", brand.name)),
        qty,
        security_stock: rng.below(20),
        featured: rng.boolean(20),
        is_visible: rng.boolean(80),
        old_price: Some(round(price * 1.2)),
        price: Some(round(price)),
        cost: Some(round(price * rng.float(0.3, 0.6))),
        kind: Some(if rng.boolean(80) {
            ProductType::Deliverable
        } else {
            ProductType::Downloadable
        }),
        backorder: rng.boolean(30),
        requires_shipping: rng.boolean(75),
        published_at: Some(today),
        weight_value: Some(round(rng.float(0.1, 20.0))),
        height_value: Some(round(rng.float(1.0, 100.0))),
        width_value: Some(round(rng.float(1.0, 100.0))),
        depth_value: Some(round(rng.float(1.0, 100.0))),
        volume_value: Some(round(rng.float(0.1, 50.0))),
        created_at,
        updated_at: created_at,
    })
    .exec(db)
    .await
}

/// Orders, each with its lines, its payments and its shipping address.
async fn seed_orders(
    db: &mut Db,
    rng: &mut Rng,
    now: &Zoned,
    customers: &[Customer],
    products: &[Product],
) -> toasty::Result<()> {
    for index in 0..ORDERS {
        let created_at = days_ago(now, 720 - (index as i64 * 720 / ORDERS as i64));
        let order = toasty::create!(Order {
            customer_id: rng.boolean(85).then(|| rng.pick(customers).id),
            number: format!("OR-{:06}", index + 1),
            total_price: Some(round(rng.float(50.0, 3000.0))),
            status: *rng.pick(&[
                OrderStatus::New,
                OrderStatus::Processing,
                OrderStatus::Shipped,
                OrderStatus::Delivered,
                OrderStatus::Delivered,
                OrderStatus::Cancelled,
            ]),
            shipping_price: Some(round(rng.float(0.0, 40.0))),
            shipping_method: Some(rng.pick(SHIPPING_METHODS).to_string()),
            created_at,
            updated_at: created_at,
        })
        .exec(db)
        .await?;

        let address = AddressFields::draw(rng);
        address.create_for_order(db, order.id).await?;

        for _ in 0..rng.between(1, 3) {
            toasty::create!(Payment {
                order_id: order.id,
                reference: format!("PAY-{:08}", rng.below(100_000_000)),
                provider: rng.pick(&["stripe", "paypal"]).to_string(),
                method: *rng.pick(&[
                    PaymentMethod::CreditCard,
                    PaymentMethod::BankTransfer,
                    PaymentMethod::Paypal,
                    PaymentMethod::ApplePay,
                    PaymentMethod::GooglePay,
                ]),
                amount: round(rng.float(20.0, 900.0)),
                currency: CurrencyCode::Usd,
                created_at,
                updated_at: created_at,
            })
            .exec(db)
            .await?;
        }

        for sort in 0..rng.between(1, 8) {
            let product = rng.pick(products);
            toasty::create!(OrderItem {
                sort: sort as u32,
                order_id: Some(order.id),
                product_id: Some(product.id),
                qty: rng.between(1, 5) as i64,
                unit_price: round(product.price.unwrap_or(25.0)),
                created_at,
                updated_at: created_at,
            })
            .exec(db)
            .await?;
        }
    }
    Ok(())
}

/// Post categories, authors, their posts, and the comments on them.
async fn seed_blog(
    db: &mut Db,
    rng: &mut Rng,
    now: &Zoned,
    customers: &[Customer],
    today: Date,
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

    for author_index in 0..AUTHORS {
        let name = person(rng);
        let handle = slug(&name);
        let created_at = days_ago(now, rng.below(700) as i64);
        let author = toasty::create!(Author {
            name: name.clone(),
            email: format!("{handle}.{author_index}@example.com"),
            bio: Some(format!("{name} writes about building software.")),
            github_handle: Some(handle.clone()),
            twitter_handle: Some(handle),
            created_at,
            updated_at: created_at,
        })
        .exec(db)
        .await?;

        for (template_index, template) in POST_TEMPLATES.iter().enumerate() {
            let title = format!(
                "{} {}",
                template.title,
                author_index * POST_TEMPLATES.len() as u64 + template_index as u64
            );
            let created_at = days_ago(now, rng.below(720) as i64);
            let tag_count = rng.between(2, 5);
            let tags = rng.pick_distinct(TAGS, tag_count);
            let post = toasty::create!(Post {
                author_id: Some(author.id),
                post_category_id: Some(rng.pick(&categories).id),
                title: title.clone(),
                slug: slug(&title),
                content: from_text(template.content),
                published_at: rng.boolean(80).then_some(today),
                tags: tags.join(","),
                seo_title: Some(title),
                seo_description: Some(template.content.chars().take(120).collect()),
                created_at,
                updated_at: created_at,
            })
            .exec(db)
            .await?;

            for _ in 0..rng.between(1, 4) {
                toasty::create!(Comment {
                    customer_id: Some(rng.pick(customers).id),
                    post_id: Some(post.id),
                    title: Some(rng.pick(REPLY_TITLES).to_string()),
                    content: Some(rng.pick(REPLIES).to_string()),
                    is_visible: rng.boolean(85),
                    created_at: days_ago(now, rng.below(400) as i64),
                })
                .exec(db)
                .await?;
            }
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
                budget: round(rng.float(50_000.0, 500_000.0)),
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
                budget: round(rng.float(20_000.0, 200_000.0)),
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
        let start_date = today_back(rng.below(400) as i64);
        let end_date = start_date
            .checked_add(Span::new().days(rng.below(10) as i64))
            .unwrap_or(start_date);
        let status = *rng.pick(&[
            LeaveStatus::Pending,
            LeaveStatus::Approved,
            LeaveStatus::Rejected,
            LeaveStatus::Taken,
            LeaveStatus::Cancelled,
        ]);
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
        let budget = round(rng.float(10_000.0, 500_000.0));
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
                budget,
                spent: round(rng.float(0.0, budget * 1.2)),
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
    // Budgets that stand out on the burn-rate chart.
    for (offset, (budget, spent)) in [
        (400_000.0, 50_000.0),
        (20_000.0, 95_000.0),
        (15_000.0, 45_000.0),
    ]
    .into_iter()
    .enumerate()
    {
        let name = format!("{} Overhaul {}", PROJECT_NAMES[offset], offset + 1);
        projects.push(
            toasty::create!(Project {
                department_id: Some(rng.pick(&departments).id),
                name: name.clone(),
                slug: slug(&name),
                status: ProjectStatus::Active,
                priority: TaskPriority::High,
                budget,
                spent,
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
                title: format!("{} {}", rng.pick(TASK_TITLES), index + 1),
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
        let hourly_rate = hourly_rate.unwrap_or_else(|| round(rng.float(50.0, 200.0)));
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
            total_cost: round(hours * hourly_rate),
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
            total_amount: 0.0,
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

        let mut total = 0.0;
        for _ in 0..rng.between(1, 5) {
            let quantity = rng.between(1, 6) as i64;
            let unit_price = round(rng.float(5.0, 400.0));
            let amount = round(quantity as f64 * unit_price);
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
            .total_amount(round(total))
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
        salary: salary.or_else(|| match employment_type {
            EmploymentType::FullTime => Some(round(rng.float(40_000.0, 180_000.0))),
            EmploymentType::PartTime => Some(round(rng.float(20_000.0, 90_000.0))),
            _ => None,
        }),
        hourly_rate: match employment_type {
            EmploymentType::Contractor => Some(round(rng.float(50.0, 200.0))),
            EmploymentType::Intern => Some(round(rng.float(15.0, 30.0))),
            _ => None,
        },
        team_color: Some(hex_color(rng)),
        skills,
        metadata: Some(
            json!({
                "certification": rng.pick(CERTIFICATIONS),
                "office": rng.pick(OFFICES),
            })
            .into(),
        ),
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

/// A timestamp `days` before `now`.
fn days_ago(now: &Zoned, days: i64) -> Timestamp {
    now.checked_sub(Span::new().days(days))
        .unwrap_or_else(|_| now.clone())
        .timestamp()
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

const CUSTOMERS: u64 = 600;
const ORDERS: u64 = 600;
const AUTHORS: u64 = 8;
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

const SHIPPING_METHODS: &[&str] = &["Standard", "Express", "Pickup", "International"];

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

const TAGS: &[&str] = &[
    "Laravel",
    "PHP",
    "JavaScript",
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
    "Design Patterns",
];

/// One blog post body the seeder draws from.
struct PostTemplate {
    title: &'static str,
    content: &'static str,
}

const POST_TEMPLATES: &[PostTemplate] = &[
    PostTemplate {
        title: "Getting Started with Queues",
        content: "Queues defer slow work such as sending mail, so a request stays fast. \
                  Configure a driver, write a job, and let a worker drain the queue.",
    },
    PostTemplate {
        title: "Building Accessible Forms",
        content: "Label every input, describe every error, and keep the whole form reachable \
                  from the keyboard. Accessibility is a requirement, not a polish step.",
    },
    PostTemplate {
        title: "Understanding Database Indexing",
        content: "An index trades write cost for read speed. Index the columns your filters \
                  and joins touch, and check the plan before you add another one.",
    },
    PostTemplate {
        title: "A Guide to Code Review",
        content: "Review the code, not the author. Keep changes small, say why, and answer \
                  within a day so the branch does not drift.",
    },
    PostTemplate {
        title: "Designing RESTful APIs",
        content: "Name resources with nouns, answer with consistent errors, and paginate \
                  every list. Version before the first breaking change, not after.",
    },
    PostTemplate {
        title: "The Art of Writing Clean CSS",
        content: "Prefer classes to ids, keep selectors shallow, and let design tokens carry \
                  the values that repeat.",
    },
    PostTemplate {
        title: "Testing Strategies That Scale",
        content: "Many fast unit tests, fewer integration tests, and a handful of end-to-end \
                  runs that prove the whole path.",
    },
    PostTemplate {
        title: "Deploying with Zero Downtime",
        content: "Run two environments, move traffic once the new one is healthy, and split \
                  every schema change across two releases.",
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

const JOB_TITLES: &[&str] = &[
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

const SKILLS: &[&str] = &[
    "PHP",
    "Laravel",
    "JavaScript",
    "TypeScript",
    "React",
    "Vue.js",
    "Python",
    "SQL",
    "Docker",
    "AWS",
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

const TASK_LABELS: &[&str] = &[
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

const CITIES: &[&str] = &[
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
