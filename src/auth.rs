//! Who signs in: the demo's user table, read through Tablo's authenticator.

use jiff::Timestamp;
use tablo::{
    Authenticator, FieldErrors, Membership, PanelUser, Registrar,
    auth::{hash_password, new_password_errors, verify_password},
    db::db,
};
use toasty::Db;
use topcoat::context::Cx;

use crate::models::{Team, TeamUser, User};

/// The signed-in user and the teams they may act for.
#[derive(Debug)]
pub struct SignedIn {
    pub user: User,
    pub teams: Vec<Membership>,
}

impl PanelUser for SignedIn {
    fn user_id(&self) -> String {
        self.user.id.to_string()
    }

    fn display_name(&self) -> &str {
        &self.user.name
    }

    /// The account's email, shown under the name in the sidebar's user menu.
    fn subtitle(&self) -> Option<&str> {
        Some(&self.user.email)
    }

    /// The address the account signs in with, and the one a mailed link goes to.
    fn email(&self) -> Option<&str> {
        Some(&self.user.email)
    }

    /// Every account may open the panel, as `User::canAccessPanel` answers.
    fn can_access_panel(&self) -> bool {
        true
    }

    fn tenants(&self) -> &[Membership] {
        &self.teams
    }
}

/// Reads the demo's users and creates them from the sign-up page.
pub struct DemoAuth;

impl DemoAuth {
    /// Loads `user` with every team, as `User::getTenants` answers.
    ///
    /// Runs on `ex` rather than on a fresh handle: sign-up calls this while its
    /// transaction holds the connection, and a second one would wait forever.
    async fn signed_in(user: User, ex: &mut dyn toasty::Executor) -> topcoat::Result<SignedIn> {
        let teams = Team::all()
            .exec(&mut *ex)
            .await?
            .into_iter()
            .map(|team| Membership::new(team.id, team.name))
            .collect();
        Ok(SignedIn { user, teams })
    }

    /// Finds the account whose email address is `email`.
    async fn by_email(cx: &Cx, email: &str) -> topcoat::Result<Option<User>> {
        let mut db = tablo::db::db(cx);
        Ok(User::filter(User::fields().email().eq(email.to_string()))
            .first()
            .exec(&mut db)
            .await?)
    }
}

impl Authenticator for DemoAuth {
    type User = SignedIn;

    async fn verify(
        &self,
        cx: &Cx,
        login: &str,
        password: &str,
    ) -> topcoat::Result<Option<SignedIn>> {
        let user = Self::by_email(cx, login).await?;
        // An unknown address costs the same work as a wrong password.
        if !verify_password(password, user.as_ref().map(|user| user.password.as_str())).await {
            return Ok(None);
        }
        match user {
            Some(user) => {
                let mut db = tablo::db::db(cx);
                Ok(Some(Self::signed_in(user, &mut db).await?))
            }
            None => Ok(None),
        }
    }

    async fn find_by_id(&self, cx: &Cx, id: &str) -> topcoat::Result<Option<SignedIn>> {
        let Ok(id) = id.parse::<u64>() else {
            return Ok(None);
        };
        let mut db = tablo::db::db(cx);
        let user = User::filter(User::fields().id().eq(id))
            .first()
            .exec(&mut db)
            .await?;
        match user {
            Some(user) => Ok(Some(Self::signed_in(user, &mut db).await?)),
            None => Ok(None),
        }
    }
}

/// What the `app` panel's sign-up page asks for.
#[derive(tablo::FormInput)]
pub struct SignUp {
    pub name: String,
    #[form(email)]
    pub email: String,
    #[form(password)]
    pub password: String,
    #[form(password)]
    pub password_confirmation: String,
}

impl Registrar for DemoAuth {
    type Input = SignUp;
    type User = SignedIn;

    async fn validate(
        &self,
        _cx: &Cx,
        sign_up: &SignUp,
        ex: &mut dyn toasty::Executor,
    ) -> topcoat::Result<FieldErrors> {
        let mut errors = new_password_errors(&sign_up.password, &sign_up.password_confirmation);
        let taken = User::filter(User::fields().email().eq(sign_up.email.clone()))
            .first()
            .exec(ex)
            .await?;
        if taken.is_some() {
            errors.add("email", tablo::text!("error-email-taken"));
        }
        Ok(errors)
    }

    async fn register(
        &self,
        _cx: &Cx,
        sign_up: SignUp,
        ex: &mut dyn toasty::Executor,
    ) -> topcoat::Result<SignedIn> {
        let user = toasty::create!(User {
            name: sign_up.name,
            email: sign_up.email,
            password: hash_password(&sign_up.password).await?,
        })
        .exec(&mut *ex)
        .await?;
        Self::signed_in(user, ex).await
    }
}

/// Creates the demo's administrator, hashed the way the login checks it.
pub async fn create_user(
    db: &mut Db,
    name: &str,
    email: &str,
    password: &str,
) -> toasty::Result<User> {
    toasty::create!(User {
        name: name.to_string(),
        email: email.to_string(),
        // The seeded account is trusted, as the Filament demo's factory trusts it.
        email_verified_at: Some(Timestamp::now()),
        password: hash_password(password)
            .await
            .expect("hash the demo password"),
    })
    .exec(db)
    .await
}

/// Resets a password, for the app panel's "Forgot your password?" page.
pub struct DemoResetter;

impl tablo::auth::PasswordResetter for DemoResetter {
    type User = SignedIn;

    async fn find_by_email(&self, cx: &Cx, email: &str) -> topcoat::Result<Option<SignedIn>> {
        let mut handle = db(cx);
        let user = User::filter(User::fields().email().eq(email.trim()))
            .first()
            .exec(&mut handle)
            .await?;
        let Some(user) = user else {
            return Ok(None);
        };
        Ok(Some(DemoAuth::signed_in(user, &mut handle).await?))
    }

    async fn reset_password(
        &self,
        _cx: &Cx,
        user_id: &str,
        password: &str,
        ex: &mut dyn toasty::Executor,
    ) -> topcoat::Result<()> {
        let id: u64 = user_id
            .parse()
            .map_err(|_| topcoat::Error::msg("the account's id is not a number"))?;
        let password = hash_password(password).await?;
        User::filter(User::fields().id().eq(id))
            .update()
            .password(password)
            .exec(ex)
            .await?;
        Ok(())
    }
}

/// Records that an account proved it reads its address.
pub struct DemoVerifier;
impl tablo::auth::EmailVerifier for DemoVerifier {
    type User = SignedIn;

    fn is_verified(&self, user: &SignedIn) -> bool {
        user.user.email_verified_at.is_some()
    }

    async fn store_verified_email(
        &self,
        _cx: &Cx,
        user_id: &str,
        email: &str,
        ex: &mut dyn toasty::Executor,
    ) -> topcoat::Result<()> {
        let id: u64 = user_id
            .parse()
            .map_err(|_| topcoat::Error::msg("the account's id is not a number"))?;
        User::filter(User::fields().id().eq(id))
            .update()
            .email(email.to_string())
            .email_verified_at(Some(Timestamp::now()))
            .exec(ex)
            .await?;
        Ok(())
    }
}

/// The name of the team a signed-in user creates.
#[derive(tablo::FormInput)]
pub struct NewTeam {
    pub name: String,
}

/// Creates a team for a signed-in user, as the Filament demo's tenant
/// registration does.
pub struct DemoTenantRegistrar;

impl tablo::auth::TenantRegistrar for DemoTenantRegistrar {
    type User = SignedIn;
    type Input = NewTeam;

    async fn validate(
        &self,
        _cx: &Cx,
        _user: &SignedIn,
        input: &Self::Input,
        _ex: &mut dyn toasty::Executor,
    ) -> topcoat::Result<FieldErrors> {
        let mut errors = FieldErrors::new();
        if input.name.trim().is_empty() {
            errors.add("name", tablo::text!("error-team-name-required"));
        }
        Ok(errors)
    }

    async fn register(
        &self,
        _cx: &Cx,
        user: &SignedIn,
        input: Self::Input,
        ex: &mut dyn toasty::Executor,
    ) -> topcoat::Result<Membership> {
        let name = input.name.trim().to_string();
        let team = toasty::create!(Team {
            name: name.clone(),
            created_at: Timestamp::now(),
            updated_at: Timestamp::now(),
        })
        .exec(ex)
        .await?;
        toasty::create!(TeamUser {
            team_id: team.id,
            user_id: user.user.id,
        })
        .exec(ex)
        .await?;
        Ok(Membership::new(team.id, team.name))
    }
}
