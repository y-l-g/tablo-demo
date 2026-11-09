//! Signing in and signing up: the demo's user table, read through Tablo's authenticator.

use tablo::{
    Authenticator, FieldErrors, Membership, PanelUser, Registrar,
    auth::{hash_password, new_password_errors, verify_password},
    db::db,
};
use topcoat::{context::Cx, l10n::Locale};

use super::{PARTNER_EMAIL, User};

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

    /// Who may open which panel. `/app` is the panel accounts sign up on, and
    /// `/admin`, holding every resource of the demo, admits staff alone, so a
    /// sign-up never reaches it. What each role may do there is
    /// [`access`](crate::access)'.
    ///
    /// Each panel is named and the rest refused, so a panel added or renamed
    /// later admits nobody until it has a rule here.
    fn can_access_panel(&self, panel: &str) -> bool {
        match panel {
            "/admin" => self.user.role.is_some(),
            "/app" => true,
            "/partners" => self.user.email == PARTNER_EMAIL,
            _ => false,
        }
    }

    fn tenants(&self) -> &[Membership] {
        &self.teams
    }

    fn locale(&self) -> Option<Locale> {
        self.user.locale.as_deref()?.parse().ok()
    }
}

/// Reads the demo's users and creates them from the sign-up page.
pub struct DemoAuth;

impl DemoAuth {
    /// Loads `user` with the teams they belong to, so a fresh account acts for no team until it
    /// registers one.
    ///
    /// Runs on `ex` rather than on a fresh handle: sign-up calls this while its transaction holds
    /// the connection, and a second one would wait forever.
    pub(crate) async fn signed_in(
        user: User,
        ex: &mut dyn toasty::Executor,
    ) -> topcoat::Result<SignedIn> {
        let teams = user
            .teams()
            .exec(ex)
            .await?
            .into_iter()
            .map(|team| Membership::new(team.id.into(), team.name))
            .collect();
        Ok(SignedIn { user, teams })
    }

    /// Finds the account whose email address is `email`.
    async fn by_email(cx: &Cx, email: &str) -> topcoat::Result<Option<User>> {
        let mut db = db(cx)?;
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
                let mut db = db(cx)?;
                Ok(Some(Self::signed_in(user, &mut db).await?))
            }
            None => Ok(None),
        }
    }

    async fn find_by_id(&self, cx: &Cx, id: &str) -> topcoat::Result<Option<SignedIn>> {
        let Ok(id) = id.parse::<u64>() else {
            return Ok(None);
        };
        let mut db = db(cx)?;
        let user = User::filter_by_id(id).first().exec(&mut db).await?;
        match user {
            Some(user) => Ok(Some(Self::signed_in(user, &mut db).await?)),
            None => Ok(None),
        }
    }

    /// Keeps the language picked from the menu on the account, so the next sign-in, in any
    /// browser, reads it too.
    async fn store_locale(&self, cx: &Cx, user: &SignedIn, locale: &Locale) -> topcoat::Result<()> {
        User::filter_by_id(user.user.id)
            .update()
            .locale(Some(locale.to_string()))
            .exec(&mut db(cx)?)
            .await?;
        Ok(())
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
