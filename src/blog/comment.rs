//! The comments customers leave on posts and products.

use tablo::{Notification, prelude::*};
use topcoat::{Result, context::Cx, icon::iconify::iconify_icon};

use super::{Comment, PostResource};
use crate::access::{self, Domain};
use crate::shared::notify;
use crate::shop::customer_name;

/// The comments of a post, or of a product, as a resource of their own.
///
/// Comments are managed from the post and product pages; Tablo renders a
/// relation as the related resource's table, which the panel must register.
pub struct CommentResource;

impl Resource for CommentResource {
    type Model = Comment;
    type Form = CommentForm;

    fn declare() -> ResourceDef<Self> {
        let c = CommentForm::controls();
        ResourceDef::new()
            .icon(iconify_icon!("lucide:message-square"))
            .without_navigation()
            // A comment is read under a post or a product, so it follows the blog's access, which
            // every role that reaches the shop holds as well.
            .policy(access::policy(Domain::Blog))
            .table(Table::new((
                TextColumn::new(lens!(Comment.title))
                    .searchable()
                    .sortable(),
                RelationColumn::of::<crate::shop::CustomerResource>(relation!(Comment.customer)),
                BooleanColumn::new(lens!(Comment.is_visible))
                    .label(tablo::text!("label-public-visibility")),
            )))
            .detail(Detail::new(
                Section::new(tablo::text!("section-comment")).columns((
                    TextColumn::new(lens!(Comment.title)),
                    RelationColumn::of::<crate::shop::CustomerResource>(relation!(
                        Comment.customer
                    )),
                    BooleanColumn::new(lens!(Comment.is_visible))
                        .label(tablo::text!("label-public-visibility")),
                    RichTextColumn::new(lens!(Comment.content)),
                )),
            ))
            .form(Schema::new(
                Section::new(tablo::text!("section-comment")).schema((
                    c.title,
                    c.customer_id.searchable(),
                    c.post_id.searchable(),
                    c.product_id.searchable(),
                    c.is_visible
                        .switch()
                        .label(tablo::text!("label-public-visibility")),
                    c.content.rich_text(),
                )),
            ))
            .record_title(lens!(Comment.title))
    }

    /// Tells the signed-in user about a new comment, once the write committed.
    async fn after_commit(cx: &Cx, change: Change<Comment>) -> Result<()> {
        if let Change::Create(comment) = &change {
            notify_new_comment(cx, comment).await?;
        }
        Ok(())
    }
}

/// The comment's record form.
#[derive(tablo::RecordForm)]
#[form(model = Comment)]
pub struct CommentForm {
    pub title: Option<String>,
    #[form(relationship = crate::shop::CustomerResource)]
    pub customer_id: Option<u64>,
    #[form(relationship = PostResource)]
    pub post_id: Option<u64>,
    #[form(relationship = crate::shop::ProductResource)]
    pub product_id: Option<u64>,
    pub is_visible: bool,
    pub content: Option<String>,
}

/// Tells the signed-in user that a comment was left.
async fn notify_new_comment(cx: &Cx, comment: &Comment) -> Result<()> {
    let description = match customer_name(cx, comment.customer_id).await {
        Some(name) => tablo::t!(cx, "notice-comment-left", customer = name),
        None => tablo::t!(cx, "notice-comment-left-unknown"),
    };
    notify(
        cx,
        Notification::success(tablo::t!(cx, "notice-new-comment")).description(description),
        tablo::url::edit::<CommentResource>(cx, comment),
    )
    .await
}
