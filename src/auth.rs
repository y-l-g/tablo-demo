//! Who signs in: the demo's user table, read through Tablo's authenticator.

use tablo::{
    FieldErrors, Membership, PanelUser,
    auth::{Authenticator, Registrar, hash_password, new_password_errors, verify_password},
};
use toasty::Db;
use topcoat::context::Cx;

use crate::models::{Team, User};

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
    async fn signed_in(cx: &Cx, user: User) -> topcoat::Result<SignedIn> {
        let mut db = tablo::db::db(cx);
        let teams = Team::all()
            .exec(&mut db)
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
        if !verify_password(password, user.as_ref().map(|user| user.password.as_str())) {
            return Ok(None);
        }
        match user {
            Some(user) => Ok(Some(Self::signed_in(cx, user).await?)),
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
            Some(user) => Ok(Some(Self::signed_in(cx, user).await?)),
            None => Ok(None),
        }
    }
}

/// What the `app` panel's sign-up page asks for.
#[derive(tablo::ActionInput)]
pub struct SignUp {
    #[form(label = "Name")]
    pub name: String,
    #[form(label = "Email address", email)]
    pub email: String,
    #[form(password)]
    pub password: String,
    #[form(password, label = "Confirm password")]
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
            errors.add("email", "Email has already been taken");
        }
        Ok(errors)
    }

    async fn register(
        &self,
        cx: &Cx,
        sign_up: SignUp,
        ex: &mut dyn toasty::Executor,
    ) -> topcoat::Result<SignedIn> {
        let user = toasty::create!(User {
            name: sign_up.name,
            email: sign_up.email,
            password: hash_password(&sign_up.password)?,
        })
        .exec(&mut *ex)
        .await?;
        Self::signed_in(cx, user).await
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
        password: hash_password(password).expect("hash the demo password"),
    })
    .exec(db)
    .await
}
