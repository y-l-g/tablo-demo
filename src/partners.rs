//! The partner portal's sign-in. The demo owns it, with its own page and its own sessions, and the
//! `partners` panel trusts it through [`ExternalAuth`] instead of showing a login form. It is the
//! shape of a panel added to an app that already signs its users in, by SSO for one.

use jiff::Timestamp;
use serde::Deserialize;
use tablo::{ExternalAuth, PanelHandle, auth::safe_next, db::db};
use topcoat::{
    Result,
    context::{Cx, context},
    router::{
        content::Form,
        error::{SeeOther, see_other},
        page, query_params, route,
    },
    session::{self, TokenHash},
    view::{View, view},
};

use crate::accounts::{PARTNER_EMAIL, SignedIn, User};
use crate::welcome::AdminPanel;

/// Where a request with no session of the demo's goes.
const SIGN_IN: &str = "/partner-sign-in";

/// Where the panel's Sign out goes, once it has ended the panel's session.
const SIGN_OUT: &str = "/partner-sign-out";

/// The mounted partner panel, which the welcome page links to.
pub struct PartnerPanel(pub PanelHandle);

/// A session of the demo's own, apart from the panel's: this one proves who the visitor is, and
/// the panel's holds what the panel keeps per browser.
#[derive(Debug, Clone, toasty::Model)]
pub struct PartnerSession {
    /// The hash of the session's token, which never reaches the database itself.
    #[key]
    pub token_hash: String,
    pub user_id: u64,
    pub expires_at: Timestamp,
}

/// A session hash as the table's key.
fn key(hash: &TokenHash) -> String {
    hash.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// The demo's sessions, as the panel reads them.
pub struct PartnerSignIn;

impl ExternalAuth for PartnerSignIn {
    type User = SignedIn;

    /// Reads a row the server holds, so that signing out ends a live connection too.
    async fn current_user(&self, cx: &Cx) -> Result<Option<SignedIn>> {
        let Some(hash) = session::token_hash(cx).await? else {
            return Ok(None);
        };
        let mut db = db(cx)?;
        let session = PartnerSession::filter_by_token_hash(key(&hash))
            .first()
            .exec(&mut db)
            .await?;
        let Some(session) = session.filter(|session| session.expires_at > Timestamp::now()) else {
            return Ok(None);
        };
        let user = User::filter_by_id(session.user_id)
            .first()
            .exec(&mut db)
            .await?;
        match user {
            Some(user) => Ok(Some(SignedIn::load(user, &mut db).await?)),
            None => Ok(None),
        }
    }

    fn login_url(&self, _cx: &Cx, next: &str) -> String {
        format!("{SIGN_IN}?return_to={next}")
    }

    fn logout_url(&self, _cx: &Cx) -> String {
        SIGN_OUT.to_string()
    }
}

/// Where the sign-in page goes next, as the panel sent it.
#[query_params]
struct ReturnTo {
    return_to: Option<String>,
}

/// `GET /partner-sign-in` — the demo's own page, in the admin panel's document and catalog.
#[page("/partner-sign-in")]
async fn sign_in_page(cx: &Cx) -> Result<impl View> {
    let AdminPanel(admin) = context::<AdminPanel>(cx);
    let entered = admin.enter(cx);
    let return_to = query_params::<ReturnTo>(cx)
        .ok()
        .and_then(|query| query.return_to.clone());
    let csrf = tablo::csrf::ensure_token(cx);
    let title = tablo::t!(&entered, "partner-sign-in-title");
    let heading = title.clone();
    let body = tablo::t!(&entered, "partner-sign-in-body");
    let submit = tablo::t!(&entered, "partner-sign-in-submit", name = "Robin Ellis");
    let page = view! {
        <main class="mx-auto flex min-h-screen max-w-md flex-col justify-center gap-4 px-6">
            <h1 class="text-2xl font-semibold tracking-tight">(heading)</h1>
            <p class="text-sm text-muted-foreground text-pretty">(body)</p>
            <form method="post" action=(SIGN_IN)>
                <input type="hidden" name=(tablo::csrf::FIELD_NAME) value=(csrf)>
                <input type="hidden" name="return_to" value=(return_to.unwrap_or_default())>
                <button type="submit" class="w-full rounded-lg bg-primary px-4 py-2 text-sm font-medium text-primary-foreground">
                    (submit)
                </button>
            </form>
        </main>
    };
    admin.document(cx, title, page).await
}

/// What the sign-in page posts.
#[derive(Deserialize)]
struct SignIn {
    csrf_token: String,
    return_to: String,
}

/// `POST /partner-sign-in` — starts the demo's session for the partner account. One click, as on
/// the welcome page: every password of the demo is public anyway.
#[route(POST "/partner-sign-in")]
async fn sign_in(cx: &Cx, Form(form): Form<SignIn>) -> Result<SeeOther> {
    tablo::csrf::verify(cx, Some(&form.csrf_token))?;
    let mut db = db(cx)?;
    let user = User::filter(User::fields().email().eq(PARTNER_EMAIL.to_string()))
        .get(&mut db)
        .await?;
    let started = session::start(cx).await?;
    toasty::create!(PartnerSession {
        token_hash: key(&started.token_hash),
        user_id: user.id,
        expires_at: Timestamp::try_from(started.expires_at).map_err(topcoat::Error::from)?,
    })
    .exec(&mut db)
    .await?;
    // The address came through the browser, so only a path of this site is followed.
    Ok(see_other(safe_next(&form.return_to).unwrap_or("/partners")))
}

/// `GET /partner-sign-out` — ends the demo's session. The panel's Sign out lands here by GET, which
/// costs a forged link no more than a sign-in it can click through again.
#[route(GET "/partner-sign-out")]
async fn sign_out(cx: &Cx) -> Result<SeeOther> {
    if let Some(hash) = session::stop(cx).await? {
        let mut db = db(cx)?;
        PartnerSession::filter_by_token_hash(key(&hash))
            .delete()
            .exec(&mut db)
            .await?;
    }
    Ok(see_other("/"))
}
