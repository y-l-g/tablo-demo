//! The blog resources: posts, categories and authors.

use tablo::{
    Action, ActionLayout, Allow, BooleanColumn, ComputedColumn, DateColumn, DateFilter, Detail,
    Grid, ImageColumn, Outcome, Places, Relation, RelationColumn, Resource, ResourceDef, Schema,
    Section, Table, TextColumn, lens, relation,
};
use topcoat::{Result, context::Cx};

use crate::models::{Author, Comment, Post, PostCategory};

/// Posts, their comments, and the categories and authors they are filed under.
pub struct PostResource;

impl Resource for PostResource {
    type Model = Post;
    type Form = PostForm;

    fn declare() -> ResourceDef<Self> {
        let c = PostForm::controls();
        ResourceDef::new()
            .icon(crate::icons::POSTS)
            .navigation_group("Blog")
            .navigation_order(0)
            .policy(Allow)
            .table(post_table())
            .detail(post_detail())
            .form(Schema::new((
                Section::new("Post").schema(
                    Grid::new(2).schema((
                        c.title.placeholder("A title readers click"),
                        c.slug,
                        c.content.multiline(8),
                        c.author_id.searchable().label("Author"),
                        c.post_category_id.searchable().label("Category"),
                        c.published_at.label("Publishing date"),
                        c.tags
                            .label("Tags")
                            .placeholder("Comma separated, as Spatie's tag input stores them"),
                    )),
                ),
                Section::new("Image").schema(c.image.label("Image")),
            )))
            .record_title(lens!(Post.title))
            // The comments of a post, as `ManagePostComments` manages them.
            .relation(Relation::has_many::<CommentResource>(
                Comment::fields().post_id(),
            ))
            .action::<TogglePostPublish>()
    }
}

/// The post list.
fn post_table() -> Table<Post> {
    Table::new((
        ImageColumn::new(lens!(Post.image)).label("Image"),
        TextColumn::new(lens!(Post.title)).searchable().sortable(),
        TextColumn::new(lens!(Post.slug)).searchable().sortable(),
        RelationColumn::of::<AuthorResource>(relation!(Post.author)).label("Author"),
        ComputedColumn::new("Status", |post: &Post| post.status().to_string()),
        RelationColumn::of::<PostCategoryResource>(relation!(Post.post_category))
            .label("Post category"),
        DateColumn::new(lens!(Post.published_at))
            .label("Publishing date")
            .sortable(),
    ))
    .default_sort_desc(lens!(Post.published_at))
    .action_layout(ActionLayout::Menu)
    .filters((DateFilter::new(lens!(Post.published_at))
        .label("Publishing date")
        .range(),))
}

/// The post's detail page.
fn post_detail() -> Detail<Post> {
    Detail::new((
        Section::new("Post").columns((
            Grid::new(2).columns((
                TextColumn::new(lens!(Post.title)),
                TextColumn::new(lens!(Post.slug)),
                DateColumn::new(lens!(Post.published_at)).label("Publishing date"),
                RelationColumn::of::<AuthorResource>(relation!(Post.author)).label("Author"),
                RelationColumn::of::<PostCategoryResource>(relation!(Post.post_category))
                    .label("Category"),
                TextColumn::new(lens!(Post.tags)),
            )),
            ImageColumn::new(lens!(Post.image)).label("Image"),
        )),
        Section::new("Content").columns(TextColumn::new(lens!(Post.content))),
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
    ) -> Result<Outcome> {
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
            .label("Post category")
            .plural_label("Post categories")
            .icon(crate::icons::POST_CATEGORIES)
            .navigation_group("Blog")
            .navigation_order(1)
            .policy(Allow)
            .table(
                Table::new((
                    TextColumn::new(lens!(PostCategory.name))
                        .searchable()
                        .sortable(),
                    TextColumn::new(lens!(PostCategory.slug))
                        .searchable()
                        .sortable(),
                    BooleanColumn::new(lens!(PostCategory.is_visible)).label("Visibility"),
                    DateColumn::new(lens!(PostCategory.updated_at))
                        .label("Last modified at")
                        .sortable(),
                ))
                .default_sort(lens!(PostCategory.name))
                .action_layout(ActionLayout::Menu),
            )
            .detail(Detail::new(Section::new("Category").columns((
                TextColumn::new(lens!(PostCategory.name)),
                TextColumn::new(lens!(PostCategory.slug)),
                TextColumn::new(lens!(PostCategory.description)),
                BooleanColumn::new(lens!(PostCategory.is_visible)).label("Visibility"),
                DateColumn::new(lens!(PostCategory.updated_at)).label("Last modified at"),
            ))))
            .form(Schema::new(Section::new("Category").schema((
                c.name,
                c.slug,
                c.description.multiline(3),
                c.is_visible.label("Visibility"),
            ))))
            .record_title(lens!(PostCategory.name))
            .action::<ToggleCategoryVisibility>()
    }
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
    ) -> Result<Outcome> {
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
            .label("Author")
            .icon(crate::icons::AUTHORS)
            .navigation_group("Blog")
            .navigation_order(2)
            .policy(Allow)
            .table(
                Table::new((
                    TextColumn::new(lens!(Author.name)).searchable().sortable(),
                    TextColumn::new(lens!(Author.email))
                        .label("Email address")
                        .searchable()
                        .sortable(),
                    TextColumn::new(lens!(Author.github_handle)).label("GitHub handle"),
                    TextColumn::new(lens!(Author.twitter_handle)).label("Twitter handle"),
                ))
                .default_sort(lens!(Author.name)),
            )
            .detail(Detail::new(Section::new("Author").columns((
                TextColumn::new(lens!(Author.name)),
                TextColumn::new(lens!(Author.email)).label("Email address"),
                TextColumn::new(lens!(Author.bio)),
                TextColumn::new(lens!(Author.github_handle)).label("GitHub handle"),
                TextColumn::new(lens!(Author.twitter_handle)).label("Twitter handle"),
            ))))
            .form(Schema::new(Section::new("Author").schema((
                c.name,
                c.email.email().unique().label("Email address"),
                c.bio.multiline(5),
                c.github_handle.label("GitHub handle"),
                c.twitter_handle.label("Twitter handle"),
            ))))
            .record_title(lens!(Author.name))
    }
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
/// Filament manages comments from the post and product pages; Tablo renders a
/// relation as the related resource's table, which the panel must register.
pub struct CommentResource;

impl Resource for CommentResource {
    type Model = Comment;
    type Form = CommentForm;

    fn declare() -> ResourceDef<Self> {
        let c = CommentForm::controls();
        ResourceDef::new()
            .plural_label("Comments")
            .icon(crate::icons::COMMENTS)
            .navigation_group("Shop")
            .navigation_order(9)
            .policy(Allow)
            .table(Table::new((
                TextColumn::new(lens!(Comment.title))
                    .searchable()
                    .sortable(),
                RelationColumn::of::<crate::shop::CustomerResource>(relation!(Comment.customer))
                    .label("Customer"),
                BooleanColumn::new(lens!(Comment.is_visible)).label("Public visibility"),
            )))
            .detail(Detail::new(
                Section::new("Comment").columns((
                    TextColumn::new(lens!(Comment.title)),
                    RelationColumn::of::<crate::shop::CustomerResource>(relation!(
                        Comment.customer
                    ))
                    .label("Customer"),
                    BooleanColumn::new(lens!(Comment.is_visible)).label("Public visibility"),
                    TextColumn::new(lens!(Comment.content)),
                )),
            ))
            .form(Schema::new(Section::new("Comment").schema((
                c.title,
                c.customer_id.searchable().label("Customer"),
                c.post_id.searchable().label("Post"),
                c.product_id.searchable().label("Product"),
                c.is_visible.label("Public visibility"),
                c.content.multiline(5),
            ))))
            .record_title(lens!(Comment.title))
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
