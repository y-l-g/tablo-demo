//! The `app` panel's tenants: creating a team, and renaming the one the session acts for.

use jiff::Timestamp;
use tablo::{FieldErrors, Membership, TenantEditor};
use topcoat::context::Cx;

use super::{SignedIn, Team, TeamUser};

/// The name of the team a signed-in user creates.
#[derive(tablo::FormInput)]
pub struct NewTeam {
    pub name: String,
}

impl NewTeam {
    /// Refuses a blank name.
    fn errors(&self) -> FieldErrors {
        let mut errors = FieldErrors::new();
        if self.name.trim().is_empty() {
            errors.add_field("name", tablo::text!("error-team-name-required"));
        }
        errors
    }
}

/// Creates a team for a signed-in user.
pub struct DemoTenantRegistrar;

impl tablo::TenantRegistrar for DemoTenantRegistrar {
    type User = SignedIn;
    type Input = NewTeam;

    async fn validate(
        &self,
        _cx: &Cx,
        _user: &SignedIn,
        input: &Self::Input,
        _ex: &mut dyn toasty::Executor,
    ) -> topcoat::Result<FieldErrors> {
        Ok(input.errors())
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
        Ok(Membership::new(team.id.into(), team.name))
    }
}

/// Renames the team the session acts for.
impl TenantEditor for DemoTenantRegistrar {
    type Input = NewTeam;
    type User = SignedIn;

    async fn fill(
        &self,
        _cx: &Cx,
        _user: &SignedIn,
        team: &Membership,
    ) -> topcoat::Result<NewTeam> {
        Ok(NewTeam {
            name: team.name.clone(),
        })
    }

    async fn validate(
        &self,
        _cx: &Cx,
        _user: &SignedIn,
        _team: &Membership,
        input: &NewTeam,
        _ex: &mut dyn toasty::Executor,
    ) -> topcoat::Result<FieldErrors> {
        Ok(input.errors())
    }

    async fn save(
        &self,
        _cx: &Cx,
        _user: &SignedIn,
        team: &Membership,
        input: NewTeam,
        ex: &mut dyn toasty::Executor,
    ) -> topcoat::Result<()> {
        Team::filter_by_id(team.tenant.get())
            .update()
            .name(input.name.trim().to_string())
            .exec(ex)
            .await?;
        Ok(())
    }
}
