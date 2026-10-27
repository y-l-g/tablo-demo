//! The blog resources: posts, categories and authors.

use tablo::{
    Action, ActionLayout, Allow, BooleanColumn, Change, DateColumn, DateFilter, Detail, Grid,
    ImageColumn, Outcome, Places, Relation, RelationColumn, Resource, ResourceDef, RichTextColumn,
    Schema, Section, Table, Text, TextColumn, WriteError,
    extend::{Column, Includes},
    lens, relation,
};
use topcoat::{Result, context::Cx};

use crate::models::{Author, Comment, Post, PostCategory};
use crate::panel::{IMAGE_EXTENSIONS, IMAGE_MAX_BYTES};

/// Posts, their comments, and the categories and authors they are filed under.
pub struct PostResource;

impl Resource for PostResource {
    type Model = Post;
    type Form = PostForm;

    fn declare() -> ResourceDef<Self> {
        let c = PostForm::controls();
        ResourceDef::new()
            .icon(crate::icons::POSTS)
            .navigation_group(tablo::text!("navigation-blog"))
            .navigation_order(0)
            .policy(Allow)
            .table(post_table())
            .detail(post_detail())
            .form(Schema::new((
                Section::new(tablo::text!("section-post")).schema(
                    Grid::new(2).schema((
                        c.title.placeholder(tablo::text!("hint-post-title")),
                        c.slug.disabled_on_edit(),
                        c.content.rich_text(),
                        c.author_id.searchable().label(tablo::text!("label-author")),
                        c.post_category_id
                            .searchable()
                            .label(tablo::text!("label-category")),
                        c.published_at.label(tablo::text!("label-publishing-date")),
                        c.tags
                            .label(tablo::text!("label-tags"))
                            .placeholder(tablo::text!("hint-tags")),
                    )),
                ),
                Section::new(tablo::text!("section-image")).schema(
                    c.image
                        .extensions(IMAGE_EXTENSIONS)
                        .max_bytes(IMAGE_MAX_BYTES),
                ),
            )))
            .record_title(lens!(Post.title))
            // The comments of a post, as `ManagePostComments` manages them.
            .relation(Relation::has_many::<CommentResource>(
                Comment::fields().post_id(),
            ))
            .action::<TogglePostPublish>()
    }

    crate::audit!();
}

/// The post list's status: published or draft, spelled in the request's locale.
struct PostStatusColumn;

impl Column<Post> for PostStatusColumn {
    fn name(&self) -> &str {
        "status"
    }

    fn label(&self) -> Text {
        tablo::text!("label-status")
    }

    fn text(&self, cx: &Cx, post: &Post) -> String {
        if post.is_published() {
            tablo::t!(cx, "status-published")
        } else {
            tablo::t!(cx, "status-draft")
        }
    }

    fn includes(&self) -> Includes<Post> {
        Includes::new()
    }
}

/// The post list.
fn post_table() -> Table<Post> {
    Table::new((
        ImageColumn::new(lens!(Post.image)),
        TextColumn::new(lens!(Post.title)).searchable().sortable(),
        TextColumn::new(lens!(Post.slug))
            .searchable()
            .sortable()
            .toggleable()
            .hidden_by_default(),
        RelationColumn::of::<AuthorResource>(relation!(Post.author))
            .label(tablo::text!("label-author"))
            .toggleable(),
        PostStatusColumn,
        RelationColumn::of::<PostCategoryResource>(relation!(Post.post_category))
            .label(tablo::text!("label-post-category"))
            .toggleable()
            .hidden_by_default(),
        DateColumn::new(lens!(Post.published_at))
            .label(tablo::text!("label-publishing-date"))
            .sortable(),
    ))
    .default_sort_desc(lens!(Post.published_at))
    .action_layout(ActionLayout::Menu)
    .filters((DateFilter::new(lens!(Post.published_at))
        .label(tablo::text!("label-publishing-date"))
        .range(),))
}

/// The post's detail page.
fn post_detail() -> Detail<Post> {
    Detail::new((
        Section::new(tablo::text!("section-post")).columns((
            Grid::new(2).columns((
                TextColumn::new(lens!(Post.title)),
                TextColumn::new(lens!(Post.slug)),
                DateColumn::new(lens!(Post.published_at))
                    .label(tablo::text!("label-publishing-date")),
                RelationColumn::of::<AuthorResource>(relation!(Post.author))
                    .label(tablo::text!("label-author")),
                RelationColumn::of::<PostCategoryResource>(relation!(Post.post_category))
                    .label(tablo::text!("label-category")),
                TextColumn::new(lens!(Post.tags)),
            )),
            ImageColumn::new(lens!(Post.image)),
        )),
        Section::new(tablo::text!("section-content"))
            .columns(RichTextColumn::new(lens!(Post.content))),
    ))
}

/// The post's record form.
#[derive(tablo::RecordForm)]
#[form(model = Post)]
pub struct PostForm {
    pub title: String,
    pub slug: String,
    pub content: String,
    #[form(relationship = AuthorResource)]
    pub author_id: Option<u64>,
    #[form(relationship = PostCategoryResource)]
    pub post_category_id: Option<u64>,
    pub published_at: Option<jiff::civil::Date>,
    #[form(optional)]
    pub tags: String,
    #[form(file, optional)]
    pub image: String,
}

/// Publishes a draft post, or returns a published one to draft.
pub struct TogglePostPublish;

impl Action<PostResource> for TogglePostPublish {
    type Input = ();
    const NAME: &'static str = "toggle-publish";
    const PLACES: Places = Places::ROW.with(Places::DETAIL);

    async fn run(
        _cx: &Cx,
        posts: &[Post],
        (): (),
        ex: &mut dyn toasty::Executor,
    ) -> Result<Outcome, WriteError> {
        for post in posts {
            let published_at = if post.is_published() {
                None
            } else {
                Some(jiff::Zoned::now().date())
            };
            Post::filter(Post::fields().id().eq(post.id))
                .update()
                .published_at(published_at)
                .exec(&mut *ex)
                .await?;
        }
        Ok(Outcome::done())
    }
}

/// Post categories.
pub struct PostCategoryResource;

impl Resource for PostCategoryResource {
    type Model = PostCategory;
    type Form = PostCategoryForm;

    fn declare() -> ResourceDef<Self> {
        let c = PostCategoryForm::controls();
        ResourceDef::new()
            .icon(crate::icons::POST_CATEGORIES)
            .navigation_group(tablo::text!("navigation-blog"))
            .navigation_order(1)
            .importable()
            .policy(Allow)
            .table(
                Table::new((
                    TextColumn::new(lens!(PostCategory.name))
                        .searchable()
                        .sortable(),
                    TextColumn::new(lens!(PostCategory.slug))
                        .searchable()
                        .sortable()
                        .toggleable(),
                    BooleanColumn::new(lens!(PostCategory.is_visible))
                        .label(tablo::text!("label-visibility")),
                    DateColumn::new(lens!(PostCategory.updated_at))
                        .label(tablo::text!("label-last-modified-at"))
                        .sortable()
                        .toggleable()
                        .hidden_by_default(),
                ))
                .default_sort(lens!(PostCategory.name))
                .action_layout(ActionLayout::Menu),
            )
            .detail(Detail::new(
                Section::new(tablo::text!("section-category")).columns((
                    TextColumn::new(lens!(PostCategory.name)),
                    TextColumn::new(lens!(PostCategory.slug)),
                    TextColumn::new(lens!(PostCategory.description)),
                    BooleanColumn::new(lens!(PostCategory.is_visible))
                        .label(tablo::text!("label-visibility")),
                    DateColumn::new(lens!(PostCategory.updated_at))
                        .label(tablo::text!("label-last-modified-at")),
                )),
            ))
            .form(Schema::new(
                Section::new(tablo::text!("section-category")).schema((
                    c.name,
                    c.slug.disabled_on_edit(),
                    c.description.multiline(3),
                    c.is_visible
                        .switch()
                        .label(tablo::text!("label-visibility")),
                )),
            ))
            .record_title(lens!(PostCategory.name))
            .action::<ToggleCategoryVisibility>()
    }

    crate::audit!();
}

/// The category's record form.
#[derive(tablo::RecordForm)]
#[form(model = PostCategory)]
pub struct PostCategoryForm {
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub is_visible: bool,
}

/// Shows or hides a category.
pub struct ToggleCategoryVisibility;

impl Action<PostCategoryResource> for ToggleCategoryVisibility {
    type Input = ();
    const NAME: &'static str = "toggle-visibility";
    const PLACES: Places = Places::ROW;

    async fn run(
        _cx: &Cx,
        categories: &[PostCategory],
        (): (),
        ex: &mut dyn toasty::Executor,
    ) -> Result<Outcome, WriteError> {
        for category in categories {
            PostCategory::filter(PostCategory::fields().id().eq(category.id))
                .update()
                .is_visible(!category.is_visible)
                .exec(&mut *ex)
                .await?;
        }
        Ok(Outcome::done())
    }
}

/// The people who write posts.
pub struct AuthorResource;

impl Resource for AuthorResource {
    type Model = Author;
    type Form = AuthorForm;

    fn declare() -> ResourceDef<Self> {
        let c = AuthorForm::controls();
        ResourceDef::new()
            .icon(crate::icons::AUTHORS)
            .navigation_group(tablo::text!("navigation-blog"))
            .navigation_order(2)
            .policy(Allow)
            .table(
                Table::new((
                    TextColumn::new(lens!(Author.name)).searchable().sortable(),
                    TextColumn::new(lens!(Author.email))
                        .label(tablo::text!("label-email-address"))
                        .searchable()
                        .sortable(),
                    TextColumn::new(lens!(Author.github_handle))
                        .label(tablo::text!("label-github-handle")),
                    TextColumn::new(lens!(Author.twitter_handle))
                        .label(tablo::text!("label-twitter-handle")),
                ))
                .default_sort(lens!(Author.name)),
            )
            .detail(Detail::new(
                Section::new(tablo::text!("section-author")).columns((
                    TextColumn::new(lens!(Author.name)),
                    TextColumn::new(lens!(Author.email)).label(tablo::text!("label-email-address")),
                    TextColumn::new(lens!(Author.bio)),
                    TextColumn::new(lens!(Author.github_handle))
                        .label(tablo::text!("label-github-handle")),
                    TextColumn::new(lens!(Author.twitter_handle))
                        .label(tablo::text!("label-twitter-handle")),
                )),
            ))
            .form(Schema::new(
                Section::new(tablo::text!("section-author")).schema((
                    c.name,
                    c.email
                        .email()
                        .unique()
                        .label(tablo::text!("label-email-address")),
                    c.bio.multiline(5),
                    c.github_handle.label(tablo::text!("label-github-handle")),
                    c.twitter_handle.label(tablo::text!("label-twitter-handle")),
                )),
            ))
            .record_title(lens!(Author.name))
    }

    crate::audit!();
}

/// The author's record form.
#[derive(tablo::RecordForm)]
#[form(model = Author)]
pub struct AuthorForm {
    pub name: String,
    pub email: String,
    pub bio: Option<String>,
    pub github_handle: Option<String>,
    pub twitter_handle: Option<String>,
}

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
            .icon(crate::icons::COMMENTS)
            .navigation_group(tablo::text!("navigation-shop"))
            .navigation_order(9)
            .policy(Allow)
            .table(Table::new((
                TextColumn::new(lens!(Comment.title))
                    .searchable()
                    .sortable(),
                RelationColumn::of::<crate::shop::CustomerResource>(relation!(Comment.customer))
                    .label(tablo::text!("label-customer")),
                BooleanColumn::new(lens!(Comment.is_visible))
                    .label(tablo::text!("label-public-visibility")),
            )))
            .detail(Detail::new(
                Section::new(tablo::text!("section-comment")).columns((
                    TextColumn::new(lens!(Comment.title)),
                    RelationColumn::of::<crate::shop::CustomerResource>(relation!(
                        Comment.customer
                    ))
                    .label(tablo::text!("label-customer")),
                    BooleanColumn::new(lens!(Comment.is_visible))
                        .label(tablo::text!("label-public-visibility")),
                    RichTextColumn::new(lens!(Comment.content)),
                )),
            ))
            .form(Schema::new(
                Section::new(tablo::text!("section-comment")).schema((
                    c.title,
                    c.customer_id
                        .searchable()
                        .label(tablo::text!("label-customer")),
                    c.post_id.searchable().label(tablo::text!("label-post")),
                    c.product_id
                        .searchable()
                        .label(tablo::text!("label-product")),
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
            crate::notifications::new_comment(cx, comment).await?;
        }
        Ok(())
    }

    crate::audit!();
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
