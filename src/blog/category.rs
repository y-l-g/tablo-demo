//! Post categories.

use tablo::prelude::*;
use topcoat::icon::iconify::iconify_icon;

use super::PostCategory;
use crate::access::{self, Domain};

/// Post categories.
pub struct PostCategoryResource;

impl Resource for PostCategoryResource {
    type Model = PostCategory;
    type Form = PostCategoryForm;

    fn declare() -> ResourceDef<Self> {
        let col = PostCategory::columns();
        let shown = PostCategory::columns();
        let c = PostCategoryForm::controls();
        ResourceDef::new()
            .icon(iconify_icon!("lucide:tags"))
            .navigation_group(tablo::text!("navigation-blog"))
            .navigation_order(1)
            .importable()
            .policy(access::policy(Domain::Blog))
            .table(
                Table::new((
                    col.name.text().searchable().sortable(),
                    col.slug.text().searchable().sortable().toggleable(),
                    col.is_visible.toggle(),
                    col.updated_at
                        .date()
                        .sortable()
                        .toggleable()
                        .hidden_by_default(),
                ))
                .default_sort(lens!(PostCategory.name))
                .action_layout(ActionLayout::Menu),
            )
            .detail(Detail::new(
                Section::new(tablo::text!("section-category")).columns((
                    shown.name.text(),
                    shown.slug.text(),
                    shown.description.text(),
                    shown.is_visible.boolean(),
                    shown.updated_at.date(),
                )),
            ))
            .form(Schema::new(
                Section::new(tablo::text!("section-category")).schema((
                    c.name,
                    c.slug.disabled_on_edit(),
                    c.description.multiline(3),
                    c.is_visible.switch(),
                )),
            ))
            .record_title(lens!(PostCategory.name))
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
