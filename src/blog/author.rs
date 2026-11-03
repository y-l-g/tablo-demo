//! The people who write posts.

use tablo::prelude::*;
use topcoat::icon::iconify::iconify_icon;

use super::Author;
use crate::access::{self, Domain};

/// The people who write posts.
pub struct AuthorResource;

impl Resource for AuthorResource {
    type Model = Author;
    type Form = AuthorForm;

    fn declare() -> ResourceDef<Self> {
        let c = AuthorForm::controls();
        ResourceDef::new()
            .icon(iconify_icon!("lucide:pen-line"))
            .navigation_group(tablo::text!("navigation-blog"))
            .navigation_order(2)
            .policy(access::policy(Domain::Blog))
            .table(
                Table::new((
                    TextColumn::new(lens!(Author.name)).searchable().sortable(),
                    TextColumn::new(lens!(Author.email)).searchable().sortable(),
                    TextColumn::new(lens!(Author.github_handle)),
                    TextColumn::new(lens!(Author.twitter_handle)),
                ))
                .default_sort(lens!(Author.name)),
            )
            .detail(Detail::new(
                Section::new(tablo::text!("section-author")).columns((
                    TextColumn::new(lens!(Author.name)),
                    TextColumn::new(lens!(Author.email)),
                    TextColumn::new(lens!(Author.bio)),
                    TextColumn::new(lens!(Author.github_handle)),
                    TextColumn::new(lens!(Author.twitter_handle)),
                )),
            ))
            .form(Schema::new(
                Section::new(tablo::text!("section-author")).schema((
                    c.name,
                    c.email.email().unique(),
                    c.bio.multiline(5),
                    c.github_handle,
                    c.twitter_handle,
                )),
            ))
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
