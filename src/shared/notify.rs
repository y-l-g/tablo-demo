//! The bell of the admin panel: notifications stored for the signed-in user.

use tablo::{Notification, Notifications, PanelUser};
use topcoat::{Result, context::Cx};

use crate::accounts::SignedIn;

/// Stores `notification` for the signed-in user, with a link to `url` when there is one.
pub async fn notify(cx: &Cx, notification: Notification, url: Option<String>) -> Result<()> {
    let Some(user) = tablo::auth::user::<SignedIn>(cx) else {
        return Ok(());
    };
    let user_id = user.user_id();
    let notification = match url {
        Some(url) => notification.action(tablo::t!(cx, "notice-view"), url),
        None => notification,
    };
    Notifications::of(cx)?.send(&user_id, notification).await?;
    Ok(())
}
