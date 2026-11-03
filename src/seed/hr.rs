//! HR's data: departments, employees, their leave and their expense claims.

use jiff::{Span, Timestamp, Zoned};
use rust_decimal::Decimal;
use toasty::Db;

use super::random::{Rng, days_ago, hex_color, money, person, phone, slug, today_back};
use crate::hr::{
    Department, Employee, EmploymentType, Expense, ExpenseCategory, ExpenseLine, ExpenseStatus,
    JOB_TITLES, LeaveRequest, LeaveStatus, LeaveType, MetadataEntry, SKILLS,
};
use crate::projects::Project;

/// How many employees, leave requests and expense claims the company holds.
const EMPLOYEES: u64 = 120;
const LEAVE_REQUESTS: u64 = 300;
const EXPENSES: u64 = 250;

/// Departments, their employees, and the leave the employees asked for.
pub(super) async fn seed_people(
    db: &mut Db,
    rng: &mut Rng,
    now: &Zoned,
) -> toasty::Result<(Vec<Department>, Vec<Employee>)> {
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
            days_requested: LeaveRequest::days_between(start_date, end_date),
            reason: rng.pick(LEAVE_REASONS).to_string(),
            reviewer_notes: (status != LeaveStatus::Pending)
                .then(|| rng.pick(REVIEW_NOTES).to_string()),
            reviewed_at: (status != LeaveStatus::Pending).then(Timestamp::now),
            created_at: days_ago(now, rng.below(400) as i64),
        })
        .exec(db)
        .await?;
    }
    Ok((departments, employees))
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

/// Expense claims, each with the lines it adds up.
pub(super) async fn seed_expenses(
    db: &mut Db,
    rng: &mut Rng,
    now: &Zoned,
    employees: &[Employee],
    projects: &[Project],
) -> toasty::Result<()> {
    for index in 0..EXPENSES {
        let employee_id = rng.pick(employees).id;
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
            project_id: rng.boolean(60).then(|| rng.pick(projects).id),
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
            approved_by: rng.boolean(30).then(|| rng.pick(employees).id),
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
            let amount = ExpenseLine::amount_of(quantity, unit_price);
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
        Expense::filter_by_id(expense.id)
            .update()
            .total_amount(total)
            .exec(db)
            .await?;
    }
    Ok(())
}

const DEPARTMENT_NAMES: &[&str] = &["Engineering", "Operations", "Marketing", "People"];

const SUBDEPARTMENTS: &[&str] = &["Frontend", "Backend", "DevOps", "QA", "Content", "SEO"];

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
