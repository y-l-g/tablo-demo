//! Tags, which posts share.

use tablo::prelude::*;
use topcoat::icon::iconify::iconify_icon;

use super::{PostResource, Tag};
use crate::access::{self, Domain};

/// Tags, which posts share.
pub struct TagResource;

impl Resource for TagResource {
    type Model = Tag;
    type Form = TagForm;

    fn declare() -> ResourceDef<Self> {
        let c = TagForm::controls();
        ResourceDef::new()
            .icon(iconify_icon!("lucide:tag"))
            .navigation_group(tablo::text!("navigation-blog"))
            .navigation_order(3)
            .policy(access::policy(Domain::Blog))
            .table(
                Table::new((
                    TextColumn::new(lens!(Tag.name)).searchable().sortable(),
                    CountColumn::of::<PostResource>(relation!(Tag.posts)),
                ))
                .default_sort(lens!(Tag.name))
                .action_layout(ActionLayout::Menu),
            )
            .detail(Detail::new(
                Section::new(tablo::text!("section-tag"))
                    .columns((TextColumn::new(lens!(Tag.name)),)),
            ))
            .form(Schema::new(
                Section::new(tablo::text!("section-tag")).schema(c.name.unique()),
            ))
            .record_title(lens!(Tag.name))
            .relation(Relation::belongs_to_many::<PostResource>(
                Tag::fields().posts(),
            ))
    }
}

/// The tag's record form.
#[derive(tablo::RecordForm)]
#[form(model = Tag)]
pub struct TagForm {
    pub name: String,
}
