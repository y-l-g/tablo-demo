//! The projects' data: projects, their tasks, and the time booked to them.

use jiff::{Timestamp, Zoned, civil::Date};
use serde_json::json;
use toasty::Db;

use super::random::{Rng, days_ago, hex_color, money, round, slug, today_back};
use crate::hr::{Department, Employee};
use crate::projects::{
    Project, ProjectStatus, TASK_LABELS, Task, TaskPriority, TaskStatus, Timesheet,
};

/// How many tasks and timesheets the projects hold.
const TASKS: u64 = 200;
const TIMESHEETS: u64 = 2000;

/// Projects, their tasks, and the time employees booked to them.
pub(super) async fn seed_projects(
    db: &mut Db,
    rng: &mut Rng,
    now: &Zoned,
    today: Date,
    departments: &[Department],
    employees: &[Employee],
) -> toasty::Result<Vec<Project>> {
    let mut projects = Vec::new();
    for name in PROJECT_NAMES {
        let budget = rng.float(10_000.0, 500_000.0);
        let estimated_hours = round(rng.float(100.0, 2000.0));
        let start_date = today_back(rng.below(180) as i64);
        projects.push(
            toasty::create!(Project {
                department_id: Some(rng.pick(departments).id),
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
                department_id: Some(rng.pick(departments).id),
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
                assigned_to: rng.boolean(70).then(|| rng.pick(employees).id),
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
        let employee = rng.pick(employees);
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
            total_cost: Timesheet::cost(hours, hourly_rate),
            created_at: days_ago(now, rng.below(90) as i64),
        })
        .exec(db)
        .await?;
    }
    Ok(projects)
}

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

const TIMESHEET_NOTES: &[&str] = &[
    "Pairing on the importer.",
    "Reviewing pull requests.",
    "Writing tests for the new endpoint.",
    "Investigating a slow query.",
];
