//! The demo's accounts, one per staff role, and the teams the `app` panel switches between.

use jiff::Timestamp;
use tablo::auth::hash_password;
use toasty::Db;

use crate::accounts::{
    ADMIN_EMAIL, ADMIN_NAME, ADMIN_PASSWORD, STAFF, StaffRole, Team, TeamUser, User,
};

/// The staff accounts, the administrator first, then one per other role, each signing in with
/// the administrator's password.
pub(super) async fn seed_staff(db: &mut Db) -> toasty::Result<Vec<User>> {
    let mut staff = vec![
        create_user(
            db,
            ADMIN_NAME,
            ADMIN_EMAIL,
            ADMIN_PASSWORD,
            StaffRole::Administrator,
        )
        .await?,
    ];
    for account in &STAFF {
        staff.push(
            create_user(
                db,
                account.name,
                account.email,
                ADMIN_PASSWORD,
                account.role,
            )
            .await?,
        );
    }
    Ok(staff)
}

/// Creates a seeded account, hashed the way the login checks it. `role` is what opens the
/// `admin` panel, and only the seed grants one.
async fn create_user(
    db: &mut Db,
    name: &str,
    email: &str,
    password: &str,
    role: StaffRole,
) -> toasty::Result<User> {
    toasty::create!(User {
        name: name.to_string(),
        email: email.to_string(),
        role: Some(role),
        // The seeded account is trusted, so its address counts as verified.
        email_verified_at: Some(Timestamp::now()),
        password: hash_password(password)
            .await
            .expect("hash the demo password"),
    })
    .exec(db)
    .await
}

/// The teams the `app` panel switches between.
pub(super) async fn seed_teams(db: &mut Db) -> toasty::Result<Vec<Team>> {
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

/// The demo's `team_user` pivot. The administrator, first of `staff`, joins every team, so the
/// app panel's switcher has something to switch, and the rest of the staff the first; an account
/// that signs up joins none until it registers one.
pub(super) async fn seed_memberships(
    db: &mut Db,
    teams: &[Team],
    staff: &[User],
) -> toasty::Result<()> {
    let Some((admin, others)) = staff.split_first() else {
        return Ok(());
    };
    for team in teams {
        toasty::create!(TeamUser {
            team_id: team.id,
            user_id: admin.id,
        })
        .exec(db)
        .await?;
    }
    for user in others {
        toasty::create!(TeamUser {
            team_id: teams[0].id,
            user_id: user.id,
        })
        .exec(db)
        .await?;
    }
    Ok(())
}
