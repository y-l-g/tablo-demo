//! What an account changes about itself: its details, its password and its address.

use jiff::Timestamp;
use tablo::{
    FieldErrors, Profile, ProfileEditor, ProfileUpdate,
    auth::{EmailChange, hash_password, request_email_change, verify_password},
    db::db,
};
use topcoat::context::Cx;

use super::{DemoAuth, SignedIn, User};

/// The id of the account `user_id` names, as [`PanelUser::user_id`] spells it.
fn account_id(user_id: &str) -> topcoat::Result<u64> {
    user_id
        .parse()
        .map_err(|_| topcoat::Error::msg("the account's id is not a number"))
}

/// Resets a password, for the app panel's "Forgot your password?" page.
pub struct DemoResetter;

impl tablo::auth::PasswordResetter for DemoResetter {
    type User = SignedIn;

    async fn find_by_email(&self, cx: &Cx, email: &str) -> topcoat::Result<Option<SignedIn>> {
        let mut handle = db(cx)?;
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
        let id = account_id(user_id)?;
        let password = hash_password(password).await?;
        User::filter_by_id(id)
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
        let id = account_id(user_id)?;
        User::filter_by_id(id)
            .update()
            .email(email.to_string())
            .email_verified_at(Some(Timestamp::now()))
            .exec(ex)
            .await?;
        Ok(())
    }
}

/// Edits the signed-in account from the app panel's profile page: the name, the
/// email address and the password, asking for the current password before the
/// address or the password changes.
impl ProfileEditor for DemoAuth {
    type Input = Profile;
    type User = SignedIn;

    fn fill(&self, _cx: &Cx, signed: &SignedIn) -> Profile {
        Profile {
            name: signed.user.name.clone(),
            email: signed.user.email.clone(),
            password: None,
            password_confirmation: String::new(),
            current_password: None,
        }
    }

    async fn validate(
        &self,
        _cx: &Cx,
        signed: &SignedIn,
        profile: &Profile,
        ex: &mut dyn toasty::Executor,
    ) -> topcoat::Result<FieldErrors> {
        let user = &signed.user;
        let mut errors = profile.password_errors();
        let new_email = profile.email.trim() != user.email;
        // Staff keep the sign-in details the seed gave them, which the admin panel signs in with.
        if user.role.is_some() {
            if new_email {
                errors.add("email", tablo::text!("error-credentials-locked"));
            }
            if profile.password.is_some() {
                errors.add("password", tablo::text!("error-credentials-locked"));
            }
        }
        if new_email || profile.password.is_some() {
            match &profile.current_password {
                None => errors.add(
                    "current_password",
                    tablo::text!("tablo-profile-current-password-needed"),
                ),
                Some(current) if !verify_password(current, Some(&user.password)).await => {
                    errors.add(
                        "current_password",
                        tablo::text!("tablo-profile-current-password-wrong"),
                    );
                }
                Some(_) => {}
            }
        }
        // Only the account's owner learns whether another account holds an
        // address: a stolen session without the password must not probe them.
        if new_email && !errors.contains_key("current_password") && !errors.contains_key("email") {
            let taken = User::filter(User::fields().email().eq(profile.email.trim()))
                .first()
                .exec(ex)
                .await?;
            if taken.is_some_and(|other| other.id != user.id) {
                errors.add("email", tablo::text!("error-email-taken"));
            }
        }
        Ok(errors)
    }

    async fn save(
        &self,
        cx: &Cx,
        signed: &SignedIn,
        profile: Profile,
        ex: &mut dyn toasty::Executor,
    ) -> topcoat::Result<ProfileUpdate> {
        let email = profile.email.trim().to_string();
        // The app panel verifies addresses, so a new one waits for the link
        // mailed to it, and the old one gets a link that keeps it.
        let stored = email != signed.user.email
            && request_email_change(cx, &mut *ex, signed, &email).await? == EmailChange::Immediate;
        let mut update = User::filter_by_id(signed.user.id)
            .update()
            .name(profile.name);
        if stored {
            update.set_email(email);
            update.set_email_verified_at(None);
        }
        if let Some(password) = &profile.password {
            update.set_password(hash_password(password).await?);
        }
        update.exec(ex).await?;
        Ok(if stored || profile.password.is_some() {
            ProfileUpdate::Credentials
        } else {
            ProfileUpdate::Details
        })
    }
}
