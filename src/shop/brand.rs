//! Brands, kept in a manual order the list's move actions change.

use tablo::prelude::*;
use tablo::write_create;
use topcoat::{Result, context::Cx, icon::iconify::iconify_icon};

use super::{Address, AddressResource, Brand, Product, ProductResource};
use crate::access::{self, Domain};

/// Brands.
pub struct BrandResource;

impl Resource for BrandResource {
    type Model = Brand;
    type Form = BrandForm;

    fn declare() -> ResourceDef<Self> {
        let c = BrandForm::controls();
        ResourceDef::new()
            .icon(iconify_icon!("lucide:bookmark"))
            .navigation_group(tablo::text!("navigation-shop"))
            .navigation_order(3)
            .policy(access::policy(Domain::Shop))
            .table(
                Table::new((
                    TextColumn::new(lens!(Brand.name))
                        .weight(FontWeight::Medium)
                        .searchable()
                        .sortable(),
                    TextColumn::new(lens!(Brand.website))
                        .searchable()
                        .sortable()
                        .toggleable(),
                    ToggleColumn::new(lens!(Brand.is_visible)),
                    DateColumn::new(lens!(Brand.updated_at))
                        .sortable()
                        .toggleable()
                        .hidden_by_default(),
                ))
                .default_sort(lens!(Brand.sort))
                .action_layout(ActionLayout::Menu)
                .row_link(
                    RowLink::new(tablo::text!("action-visit-website"), |brand: &Brand| {
                        brand.website.clone()
                    })
                    .new_tab()
                    .reading(Reads::of(lens!(Brand.website))),
                ),
            )
            .detail(Detail::new(
                Section::new(tablo::text!("section-brand")).columns((
                    TextColumn::new(lens!(Brand.name)),
                    TextColumn::new(lens!(Brand.slug)),
                    TextColumn::new(lens!(Brand.website)),
                    TextColumn::new(lens!(Brand.description)),
                    BooleanColumn::new(lens!(Brand.is_visible)),
                    DateColumn::new(lens!(Brand.updated_at)),
                )),
            ))
            .form(Schema::new(
                Section::new(tablo::text!("section-brand")).schema((
                    Grid::new(2).schema((c.name, c.slug.disabled_on_edit())),
                    c.website.url(),
                    c.is_visible.switch(),
                    c.description.multiline(4),
                )),
            ))
            .record_title(lens!(Brand.name))
            .relation(Relation::has_many::<ProductResource>(
                Product::fields().brand_id(),
            ))
            .relation(Relation::has_many::<AddressResource>(Address::fields().brand_id()).tab())
            .action::<MoveBrandUp>()
            .action::<MoveBrandDown>()
    }

    /// A new brand goes last in the manual order.
    async fn create_record(
        cx: &Cx,
        form: BrandForm,
        ex: &mut dyn toasty::Executor,
    ) -> Result<Brand, WriteError<BrandFormField>> {
        let mut brand = write_create::<Self>(cx, form, ex).await?;
        let last = Brand::all()
            .order_by(Brand::fields().sort().desc())
            .first()
            .exec(&mut *ex)
            .await?;
        brand.sort = last.map_or(0, |last| last.sort + 1);
        Brand::filter_by_id(brand.id)
            .update()
            .sort(brand.sort)
            .exec(&mut *ex)
            .await?;
        Ok(brand)
    }
}

/// The brand's record form.
#[derive(tablo::RecordForm)]
#[form(model = Brand)]
pub struct BrandForm {
    pub name: String,
    pub slug: String,
    pub website: Option<String>,
    pub is_visible: bool,
    pub description: Option<String>,
}

/// Moves a brand one place up the manual order, swapping with the brand above it.
pub struct MoveBrandUp;

impl Action<BrandResource> for MoveBrandUp {
    type Input = ();
    const NAME: &'static str = "move-up";
    const PLACES: Places = Places::ROW;

    async fn run(
        _cx: &Cx,
        brands: &[Brand],
        (): (),
        ex: &mut dyn toasty::Executor,
    ) -> Result<Outcome, WriteError<std::convert::Infallible>> {
        for brand in brands {
            swap_place(brand, Direction::Up, ex).await?;
        }
        Ok(Outcome::done())
    }

    fn label() -> Text {
        tablo::text!("action-move-up")
    }
}

/// Moves a brand one place down the manual order, swapping with the brand below it.
pub struct MoveBrandDown;

impl Action<BrandResource> for MoveBrandDown {
    type Input = ();
    const NAME: &'static str = "move-down";
    const PLACES: Places = Places::ROW;

    async fn run(
        _cx: &Cx,
        brands: &[Brand],
        (): (),
        ex: &mut dyn toasty::Executor,
    ) -> Result<Outcome, WriteError<std::convert::Infallible>> {
        for brand in brands {
            swap_place(brand, Direction::Down, ex).await?;
        }
        Ok(Outcome::done())
    }

    fn label() -> Text {
        tablo::text!("action-move-down")
    }
}

/// Which neighbour in the manual order a move swaps with.
#[derive(Clone, Copy)]
enum Direction {
    Up,
    Down,
}

/// Swaps `brand`'s place in the manual order with its neighbour's, when it has one.
async fn swap_place(
    brand: &Brand,
    direction: Direction,
    ex: &mut dyn toasty::Executor,
) -> toasty::Result<()> {
    let sort = Brand::fields().sort();
    let neighbour = match direction {
        Direction::Up => {
            Brand::filter(sort.lt(brand.sort))
                .order_by(Brand::fields().sort().desc())
                .first()
                .exec(&mut *ex)
                .await?
        }
        Direction::Down => {
            Brand::filter(sort.gt(brand.sort))
                .order_by(Brand::fields().sort().asc())
                .first()
                .exec(&mut *ex)
                .await?
        }
    };
    let Some(neighbour) = neighbour else {
        return Ok(());
    };
    Brand::filter_by_id(brand.id)
        .update()
        .sort(neighbour.sort)
        .exec(&mut *ex)
        .await?;
    Brand::filter_by_id(neighbour.id)
        .update()
        .sort(brand.sort)
        .exec(&mut *ex)
        .await?;
    Ok(())
}
