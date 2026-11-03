//! The blog: posts, the categories, tags and authors they are filed under, and the comments
//! customers leave on posts and products.
//!
//! Its resources form the sidebar's Blog group. Comments are a tab on a post's or a product's
//! page, so their resource has no sidebar entry of its own.

mod author;
mod category;
mod comment;
mod models;
mod post;
mod tag;

pub use author::AuthorResource;
pub use category::PostCategoryResource;
pub use comment::CommentResource;
pub use models::{Author, Comment, Post, PostCategory, PostTag, Tag};
pub use post::PostResource;
pub use tag::TagResource;
