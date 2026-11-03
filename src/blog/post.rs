//! Posts: the list with its publishing figures, the detail page, the form, and publishing.

use jiff::Zoned;
use tablo::db;
use tablo::extend::{Column, Includes};
use tablo::prelude::*;
use topcoat::{Result, context::Cx, icon::iconify::iconify_icon, view::View};

use super::{AuthorResource, Comment, CommentResource, Post, PostCategoryResource, TagResource};
use crate::access::{self, Domain};
use crate::shared::stats::{percent, share};
use crate::shared::uploads::{IMAGE_EXTENSIONS, IMAGE_MAX_BYTES};

/// Posts, their comments, and the categories and authors they are filed under.
pub struct PostResource;

impl Resource for PostResource {
    type Model = Post;
    type Form = PostForm;

    fn declare() -> ResourceDef<Self> {
        let c = PostForm::controls();
        ResourceDef::new()
            .icon(iconify_icon!("lucide:file-text"))
            .navigation_group(tablo::text!("navigation-blog"))
            .navigation_order(0)
            .policy(access::policy(Domain::Blog))
            .table(post_table())
            .detail(post_detail())
            .form(Schema::new((
                Section::new(tablo::text!("section-post")).schema(Grid::new(2).schema((
                    c.title.placeholder(tablo::text!("hint-post-title")),
                    c.slug.disabled_on_edit(),
                    c.content.rich_text(),
                    c.author_id.searchable(),
                    c.post_category_id.searchable(),
                    c.published_at,
                    c.tags.searchable(),
                ))),
                Section::new(tablo::text!("section-image")).schema(
                    c.image
                        .image()
                        .extensions(IMAGE_EXTENSIONS)
                        .max_bytes(IMAGE_MAX_BYTES),
                ),
            )))
            .record_title(lens!(Post.title))
            .relation(Relation::has_many::<CommentResource>(Comment::fields().post_id()).tab())
            .action::<TogglePostPublish>()
            .list_widget::<PostStatusWidget>()
    }
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
        RelationColumn::of::<AuthorResource>(relation!(Post.author)).toggleable(),
        PostStatusColumn,
        RelationColumn::list::<TagResource>(relation!(Post.tags)).toggleable(),
        RelationColumn::of::<PostCategoryResource>(relation!(Post.post_category))
            .toggleable()
            .hidden_by_default(),
        DateColumn::new(lens!(Post.published_at)).sortable(),
    ))
    .default_sort_desc(lens!(Post.published_at))
    .action_layout(ActionLayout::Menu)
    .filters((DateFilter::new(lens!(Post.published_at)).range(),))
}

/// The post's detail page.
fn post_detail() -> Detail<Post> {
    Detail::new((
        Section::new(tablo::text!("section-post")).columns((
            Grid::new(2).columns((
                TextColumn::new(lens!(Post.title)),
                TextColumn::new(lens!(Post.slug)),
                DateColumn::new(lens!(Post.published_at)),
                RelationColumn::of::<AuthorResource>(relation!(Post.author)),
                RelationColumn::of::<PostCategoryResource>(relation!(Post.post_category)),
                RelationColumn::list::<TagResource>(relation!(Post.tags)),
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
    #[form(relationship = TagResource)]
    pub tags: Vec<u64>,
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
    ) -> Result<Outcome, WriteError<std::convert::Infallible>> {
        for post in posts {
            let published_at = if post.is_published() {
                None
            } else {
                Some(jiff::Zoned::now().date())
            };
            Post::filter_by_id(post.id)
                .update()
                .published_at(published_at)
                .exec(&mut *ex)
                .await?;
        }
        Ok(Outcome::done())
    }
}

/// The articles published and the drafts, above the post list, following its search and filters.
/// A post dated after today counts as a draft, as the list's status column shows it. The list
/// filters by date alone, so the tiles do not link. Only a user who may open the post list sees
/// it.
pub struct PostStatusWidget;

impl Widget for PostStatusWidget {
    fn declare() -> WidgetDef {
        WidgetDef::new()
            .full_width()
            .access(can_list::<PostResource>)
    }

    async fn render(cx: &Cx) -> Result<impl View> {
        let posts = list_query::<PostResource>(cx).await?;
        let total = db::count(cx, posts.clone()).await?;
        let today = Zoned::now().date();
        let published = posts.filter(Post::fields().published_at().le(today));
        let published = db::count(cx, published).await?;
        let drafts = total - published;
        let detail = |count: u64| {
            tablo::text!(
                "stat-detail-share-articles",
                share = percent(share(count as f64, total as f64))
            )
        };
        Ok(tablo::stats(
            cx,
            [
                Stat::new(tablo::text!("status-published"), published.to_string())
                    .detail(detail(published))
                    .color(Color::Success),
                Stat::new(tablo::text!("status-draft"), drafts.to_string())
                    .detail(detail(drafts))
                    .color(Color::Gray),
            ],
        ))
    }
}
