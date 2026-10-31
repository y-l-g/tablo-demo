//! The blog: authors, categories and posts.

use jiff::{Timestamp, civil::Date};
use toasty::Deferred;

use super::Comment;

/// Someone who writes posts.
#[derive(Debug, Clone, toasty::Model, tablo::ModelColumns)]
pub struct Author {
    #[key]
    #[auto]
    pub id: u64,
    pub name: String,
    #[unique]
    pub email: String,
    pub photo: Option<String>,
    pub bio: Option<String>,
    pub github_handle: Option<String>,
    pub twitter_handle: Option<String>,
    #[default(Timestamp::now())]
    pub created_at: Timestamp,
    #[update(Timestamp::now())]
    pub updated_at: Timestamp,
    #[has_many]
    pub posts: Deferred<Vec<Post>>,
}

/// A topic posts are filed under.
#[derive(Debug, Clone, toasty::Model, tablo::ModelColumns)]
pub struct PostCategory {
    #[key]
    #[auto]
    pub id: u64,
    pub name: String,
    #[unique]
    pub slug: String,
    pub description: Option<String>,
    pub is_visible: bool,
    pub seo_title: Option<String>,
    pub seo_description: Option<String>,
    #[default(Timestamp::now())]
    pub created_at: Timestamp,
    #[update(Timestamp::now())]
    pub updated_at: Timestamp,
    #[has_many]
    pub posts: Deferred<Vec<Post>>,
}

/// A label posts share.
#[derive(Debug, Clone, toasty::Model, tablo::ModelColumns)]
pub struct Tag {
    #[key]
    #[auto]
    pub id: u64,
    #[unique]
    pub name: String,
    #[default(Timestamp::now())]
    pub created_at: Timestamp,
    #[update(Timestamp::now())]
    pub updated_at: Timestamp,
    #[has_many]
    pub post_tags: Deferred<Vec<PostTag>>,
    #[has_many(via = post_tags.post)]
    pub posts: Deferred<Vec<Post>>,
}

/// Files a post under a tag: the join model of the two.
#[derive(Debug, Clone, toasty::Model, tablo::ModelColumns)]
#[key(post_id, tag_id)]
pub struct PostTag {
    #[index]
    pub post_id: u64,
    #[belongs_to(key = post_id, references = id)]
    pub post: Deferred<Post>,
    #[index]
    pub tag_id: u64,
    #[belongs_to(key = tag_id, references = id)]
    pub tag: Deferred<Tag>,
}

/// A published or drafted article.
#[derive(Debug, Clone, toasty::Model, tablo::ModelColumns)]
pub struct Post {
    #[key]
    #[auto]
    pub id: u64,
    #[index]
    pub author_id: Option<u64>,
    #[belongs_to(key = author_id, references = id)]
    pub author: Deferred<Option<Author>>,
    #[index]
    pub post_category_id: Option<u64>,
    #[belongs_to(key = post_category_id, references = id)]
    pub post_category: Deferred<Option<PostCategory>>,
    pub title: String,
    #[unique]
    pub slug: String,
    pub content: String,
    pub published_at: Option<Date>,
    /// The post's image, the path of the uploaded file.
    #[default(String::new())]
    pub image: String,
    pub seo_title: Option<String>,
    pub seo_description: Option<String>,
    #[default(Timestamp::now())]
    pub created_at: Timestamp,
    #[update(Timestamp::now())]
    pub updated_at: Timestamp,
    #[has_many]
    pub comments: Deferred<Vec<Comment>>,
    #[has_many]
    pub post_tags: Deferred<Vec<PostTag>>,
    #[has_many(via = post_tags.tag)]
    pub tags: Deferred<Vec<Tag>>,
}

impl Post {
    /// Whether the post is readable: a publishing date that has passed.
    pub fn is_published(&self) -> bool {
        self.published_at
            .is_some_and(|date| date <= jiff::Zoned::now().date())
    }
}
