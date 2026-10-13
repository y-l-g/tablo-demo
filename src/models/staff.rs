//! The people who sign in and the teams they belong to.

use jiff::Timestamp;
use toasty::Deferred;

/// A user of either panel.
#[derive(Debug, Clone, toasty::Model, tablo::ModelColumns)]
pub struct User {
    #[key]
    #[auto]
    pub id: u64,
    pub name: String,
    #[unique]
    pub email: String,
    /// When the address was confirmed. Tablo ships no verification flow, so the
    /// demo only carries the column.
    pub email_verified_at: Option<Timestamp>,
    /// An Argon2id hash in PHC string format.
    pub password: String,
    pub remember_token: Option<String>,
    #[default(Timestamp::now())]
    pub created_at: Timestamp,
    #[update(Timestamp::now())]
    pub updated_at: Timestamp,
    #[has_many(pair = user)]
    pub team_users: Deferred<Vec<TeamUser>>,
    #[has_many(via = team_users.team)]
    pub teams: Deferred<Vec<Team>>,
}

/// The `app` panel's tenant: a team a user acts for.
#[derive(Debug, Clone, toasty::Model, tablo::ModelColumns)]
pub struct Team {
    #[key]
    #[auto]
    pub id: uuid::Uuid,
    pub name: String,
    #[default(Timestamp::now())]
    pub created_at: Timestamp,
    #[update(Timestamp::now())]
    pub updated_at: Timestamp,
    #[has_many(pair = team)]
    pub team_users: Deferred<Vec<TeamUser>>,
    #[has_many(via = team_users.user)]
    pub users: Deferred<Vec<User>>,
}

/// One user's membership of one team, where Laravel's `team_user` pivot lives.
#[derive(Debug, Clone, toasty::Model, tablo::ModelColumns)]
#[key(team_id, user_id)]
pub struct TeamUser {
    #[index]
    pub team_id: uuid::Uuid,
    #[belongs_to(key = team_id, references = id)]
    pub team: Deferred<Team>,
    #[index]
    pub user_id: u64,
    #[belongs_to(key = user_id, references = id)]
    pub user: Deferred<User>,
}
