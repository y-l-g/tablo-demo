//! The notifications the demo stores for the bell of the admin panel.

use tablo::{Notification, Notifications, PanelUser};
use topcoat::{Result, context::Cx};

use crate::auth::SignedIn;
use crate::models::{Comment, Customer, Order};

/// Tells the signed-in user that an order was placed.
pub async fn new_order(cx: &Cx, order: &Order) -> Result<()> {
    let customer = customer_name(cx, order.customer_id).await;
    let description = match customer {
        Some(name) => format!("{name} placed an order."),
        None => "An order was placed.".to_string(),
    };
    send(
        cx,
        Notification::success("New order").description(description),
        tablo::url::edit::<crate::shop::OrderResource>(cx, order),
    )
    .await
}

/// Tells the signed-in user that a comment was left.
pub async fn new_comment(cx: &Cx, comment: &Comment) -> Result<()> {
    let customer = customer_name(cx, comment.customer_id).await;
    let description = match customer {
        Some(name) => format!("{name} left a comment."),
        None => "A comment was left.".to_string(),
    };
    send(
        cx,
        Notification::success("New comment").description(description),
        tablo::url::edit::<crate::blog::CommentResource>(cx, comment),
    )
    .await
}

/// Stores `notification` for the signed-in user, with a link to `url` when there is one.
async fn send(cx: &Cx, notification: Notification, url: Option<String>) -> Result<()> {
    let Some(user) = tablo::auth::user::<SignedIn>(cx) else {
        return Ok(());
    };
    let user_id = user.user_id();
    let notification = match url {
        Some(url) => notification.action("View", url),
        None => notification,
    };
    Notifications::of(cx)?.send(&user_id, notification).await?;
    Ok(())
}

/// The name of the customer `id` names, read after the write committed.
async fn customer_name(cx: &Cx, id: Option<u64>) -> Option<String> {
    let id = id?;
    let mut db = tablo::db::db(cx);
    Customer::filter(Customer::fields().id().eq(id))
        .first()
        .exec(&mut db)
        .await
        .ok()
        .flatten()
        .map(|customer| customer.name)
}
