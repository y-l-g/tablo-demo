//! Signing in and signing up: the demo's user table, which Tablo's [`PasswordAuth`] signs in,
//! and whose profile, password reset and email verification it serves.

use jiff::Timestamp;
use tablo::{
    FieldErrors, Lens, Membership, PanelId, PanelUser, PasswordAccount, PasswordAuth, Profile,
    Registrar,
    auth::{hash_password, new_password_errors, normalize_email},
    db::db,
    lens,
};
use topcoat::{context::Cx, l10n::Locale};

use super::{PARTNER_EMAIL, User};

/// Signs the demo's users in, and edits, resets and verifies their accounts.
pub fn password_auth() -> PasswordAuth<User> {
    PasswordAuth::default()
}

/// The signed-in user and the teams they may act for.
#[derive(Debug)]
pub struct SignedIn {
    pub user: User,
    pub teams: Vec<Membership>,
}

impl SignedIn {
    /// Loads `user` with the teams they belong to, so a fresh account acts for no team until it
    /// registers one.
    ///
    /// Runs on `ex` rather than on a fresh handle: sign-up calls this while its transaction holds
    /// the connection, and a second one would wait forever.
    pub(crate) async fn load(user: User, ex: &mut dyn toasty::Executor) -> topcoat::Result<Self> {
        let teams = user
            .teams()
            .exec(ex)
            .await?
            .into_iter()
            .map(|team| Membership::new(team.id.into(), team.name))
            .collect();
        Ok(Self { user, teams })
    }
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
    fn can_access_panel(&self, panel: &PanelId) -> bool {
        if panel == "admin" {
            self.user.role.is_some()
        } else if panel == "app" {
            true
        } else if panel == "partners" {
            self.user.email == PARTNER_EMAIL
        } else {
            false
        }
    }

    fn tenants(&self) -> &[Membership] {
        &self.teams
    }

    fn locale(&self) -> Option<Locale> {
        self.user.locale.as_deref()?.parse().ok()
    }
}

/// The columns [`password_auth`] reads, and what it signs in: the user with their teams.
impl PasswordAccount for User {
    type User = SignedIn;

    fn name() -> Lens<Self, String> {
        lens!(User.name)
    }

    fn email() -> Lens<Self, String> {
        lens!(User.email)
    }

    fn password_hash() -> Lens<Self, String> {
        lens!(User.password)
    }

    fn email_verified_at() -> Lens<Self, Option<Timestamp>> {
        lens!(User.email_verified_at)
    }

    fn account(user: &SignedIn) -> &Self {
        &user.user
    }

    async fn sign_in(
        self,
        _cx: &Cx,
        ex: &mut dyn toasty::Executor,
    ) -> topcoat::Result<Option<SignedIn>> {
        Ok(Some(SignedIn::load(self, ex).await?))
    }

    /// Staff keep the sign-in details the seed gave them, which the admin panel signs in with.
    fn profile_errors(user: &SignedIn, profile: &Profile) -> FieldErrors<Profile> {
        let mut errors = FieldErrors::<Profile>::new();
        if user.user.role.is_some() {
            if normalize_email(&profile.email) != user.user.email {
                errors.add(|f| f.email, tablo::text!("error-credentials-locked"));
            }
            if profile.password.is_some() {
                errors.add(|f| f.password, tablo::text!("error-credentials-locked"));
            }
        }
        errors
    }

    /// Keeps the language picked from the menu on the account, so the next sign-in, in any
    /// browser, reads it too.
    async fn store_locale(cx: &Cx, user: &SignedIn, locale: &Locale) -> topcoat::Result<()> {
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

/// Creates an account from the `app` panel's sign-up page, its email spelled as
/// [`password_auth`] looks it up.
pub struct DemoSignUp;

impl Registrar for DemoSignUp {
    type Input = SignUp;
    type User = SignedIn;

    async fn validate(
        &self,
        _cx: &Cx,
        sign_up: &SignUp,
        ex: &mut dyn toasty::Executor,
    ) -> topcoat::Result<FieldErrors> {
        let mut errors = new_password_errors(&sign_up.password, &sign_up.password_confirmation);
        let taken = User::filter(User::fields().email().eq(normalize_email(&sign_up.email)))
            .first()
            .exec(ex)
            .await?;
        if taken.is_some() {
            errors.add_field("email", tablo::text!("error-email-taken"));
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
            email: normalize_email(&sign_up.email),
            password: hash_password(&sign_up.password).await?,
        })
        .exec(&mut *ex)
        .await?;
        SignedIn::load(user, ex).await
    }
}
