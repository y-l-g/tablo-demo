//! Product categories, which nest one level deep.

use tablo::prelude::*;
use topcoat::icon::iconify::iconify_icon;

use super::{ProductCategory, ProductResource};
use crate::access::{self, Domain};

/// Product categories.
pub struct ProductCategoryResource;

impl Resource for ProductCategoryResource {
    type Model = ProductCategory;
    type Form = ProductCategoryForm;

    fn declare() -> ResourceDef<Self> {
        let col = ProductCategory::columns();
        let shown = ProductCategory::columns();
        let c = ProductCategoryForm::controls();
        ResourceDef::new()
            .icon(iconify_icon!("lucide:tag"))
            .navigation_group(tablo::text!("navigation-shop"))
            .navigation_order(4)
            .importable()
            .policy(access::policy(Domain::Shop))
            .table(
                Table::new((
                    col.name.text().searchable().sortable(),
                    RelationColumn::of::<ProductCategoryResource>(relation!(
                        ProductCategory.parent
                    ))
                    .toggleable(),
                    col.is_visible.toggle(),
                    col.updated_at
                        .date()
                        .sortable()
                        .toggleable()
                        .hidden_by_default(),
                ))
                .default_sort(lens!(ProductCategory.name))
                .action_layout(ActionLayout::Menu),
            )
            .detail(Detail::new(
                Section::new(tablo::text!("section-category")).columns((
                    shown.name.text(),
                    shown.slug.text(),
                    RelationColumn::of::<ProductCategoryResource>(relation!(
                        ProductCategory.parent
                    )),
                    shown.is_visible.boolean(),
                    shown.description.text(),
                    shown.updated_at.date(),
                )),
            ))
            .form(Schema::new(
                Section::new(tablo::text!("section-category")).schema((
                    Grid::new(2).schema((c.name, c.slug.disabled_on_edit())),
                    // Categories nest one level deep: only a top-level category may be a parent.
                    c.parent_id
                        .label(tablo::text!("label-parent-category"))
                        .options_where(|_cx| ProductCategory::fields().parent_id().is_none()),
                    c.is_visible.switch(),
                    c.description.multiline(4),
                )),
            ))
            .record_title(lens!(ProductCategory.name))
            .relation(Relation::belongs_to_many::<ProductResource>(
                ProductCategory::fields().products(),
            ))
    }
}

/// The category's record form.
#[derive(tablo::RecordForm)]
#[form(model = ProductCategory)]
pub struct ProductCategoryForm {
    pub name: String,
    pub slug: String,
    #[form(relationship = ProductCategoryResource)]
    pub parent_id: Option<u64>,
    pub is_visible: bool,
    pub description: Option<String>,
}
