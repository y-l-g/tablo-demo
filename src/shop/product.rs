//! Products: the list, the form, and the actions that adjust a product's price, stock and
//! visibility.

use rust_decimal::Decimal;
use tablo::prelude::*;
use topcoat::{Result, context::Cx, icon::iconify::iconify_icon};

use super::{BrandResource, Product, ProductCategoryResource};
use crate::access::{self, Domain};
use crate::blog::{Comment, CommentResource};
use crate::shared::uploads::{IMAGE_EXTENSIONS, IMAGE_MAX_BYTES};

/// Products.
pub struct ProductResource;

impl Resource for ProductResource {
    type Model = Product;
    type Form = ProductForm;

    fn declare() -> ResourceDef<Self> {
        let c = ProductForm::controls();
        ResourceDef::new()
            .icon(iconify_icon!("lucide:package"))
            .navigation_group(tablo::text!("navigation-shop"))
            .navigation_order(0)
            .navigation_badge(
                Product::fields()
                    .qty()
                    .lt(Product::fields().security_stock()),
            )
            .policy(access::policy(Domain::Shop))
            .table(product_table())
            .form(Schema::new((
                Section::new(tablo::text!("section-product")).schema((
                    Grid::new(2).schema((c.name, c.slug.disabled_on_edit())),
                    c.description.rich_text(),
                )),
                Section::new(tablo::text!("section-images")).schema((
                    c.image
                        .extensions(IMAGE_EXTENSIONS)
                        .max_bytes(IMAGE_MAX_BYTES),
                    c.gallery
                        .extensions(IMAGE_EXTENSIONS)
                        .max_bytes(IMAGE_MAX_BYTES)
                        .max_files(4),
                )),
                Section::new(tablo::text!("section-pricing")).schema(Grid::new(2).schema((
                    c.price.prefix("$"),
                    c.old_price.prefix("$"),
                    c.cost.prefix("$"),
                ))),
                Section::new(tablo::text!("section-inventory")).schema(Grid::new(2).schema((
                    c.sku.label(tablo::text!("label-sku-stock-keeping-unit")),
                    c.barcode,
                    c.qty,
                    c.security_stock,
                ))),
                Section::new(tablo::text!("section-shipping")).schema(
                    Grid::new(2).schema((c.backorder.switch(), c.requires_shipping.switch())),
                ),
                Section::new(tablo::text!("section-status"))
                    .schema(Grid::new(2).schema((c.is_visible.switch(), c.published_at))),
                Section::new(tablo::text!("section-associations"))
                    .schema((c.brand_id.searchable(), c.categories.searchable())),
            )))
            .detail(Detail::new((Section::new(tablo::text!("section-product"))
                .columns((
                    TextColumn::new(lens!(Product.name)),
                    TextColumn::new(lens!(Product.slug)),
                    ImageColumn::new(lens!(Product.image)),
                    ComputedColumn::new(tablo::text!("label-gallery"), |product: &Product| {
                        product.gallery.len().to_string()
                    }),
                    RelationColumn::of::<BrandResource>(relation!(Product.brand)),
                    RelationColumn::list::<ProductCategoryResource>(relation!(Product.categories)),
                    BooleanColumn::new(lens!(Product.is_visible)),
                    TextColumn::new(lens!(Product.price)).money(Currency::USD),
                    TextColumn::new(lens!(Product.sku)),
                    TextColumn::new(lens!(Product.qty)).numeric(0),
                    DateColumn::new(lens!(Product.published_at)),
                    RichTextColumn::new(lens!(Product.description)),
                )),)))
            .record_title(lens!(Product.name))
            .global_search(
                GlobalSearch::new()
                    .detail(tablo::text!("label-sku"), lens!(Product.sku))
                    .detail(tablo::text!("label-price"), lens!(Product.price)),
            )
            .relation(Relation::has_many::<CommentResource>(Comment::fields().product_id()).tab())
            .action::<BulkSetProductVisibility>()
            .action::<AdjustProductPrice>()
            .action::<AdjustProductStock>()
    }
}

/// The product list.
fn product_table() -> Table<Product> {
    Table::new((
        ImageColumn::new(lens!(Product.image)),
        TextColumn::new(lens!(Product.name))
            .weight(FontWeight::Medium)
            .searchable()
            .sortable(),
        RelationColumn::of::<BrandResource>(relation!(Product.brand)).toggleable(),
        ToggleColumn::new(lens!(Product.is_visible)).toggleable(),
        TextColumn::new(lens!(Product.price)).money(Currency::USD),
        TextColumn::new(lens!(Product.sku))
            .searchable()
            .sortable()
            .toggleable(),
        TextColumn::new(lens!(Product.qty))
            .numeric(0)
            .sortable()
            .toggleable(),
        TextColumn::new(lens!(Product.security_stock))
            .numeric(0)
            .sortable()
            .toggleable()
            .hidden_by_default(),
        TextColumn::new(lens!(Product.units_sold))
            .numeric(0)
            .sortable(),
        DateColumn::new(lens!(Product.published_at))
            .sortable()
            .toggleable()
            .hidden_by_default(),
    ))
    .default_sort(lens!(Product.name))
    .action_layout(ActionLayout::Menu)
    // Six filters: a reader sets several, then reruns the list once.
    .deferred_filters()
    .filters((
        SelectFilter::relationship::<BrandResource>(lens!(Product.brand_id)),
        TernaryFilter::new(lens!(Product.is_visible)),
        TernaryFilter::new(lens!(Product.featured)),
        TernaryFilter::new(lens!(Product.backorder)),
        TernaryFilter::new(lens!(Product.requires_shipping)),
        DateFilter::new(lens!(Product.published_at)),
    ))
    .summary(Summary::count(tablo::text!("summary-products")))
    .summary(
        Summary::count(tablo::text!("summary-low-stock")).query(Product::fields().qty().lt(10)),
    )
}

/// The product's record form.
#[derive(tablo::RecordForm)]
#[form(model = Product)]
pub struct ProductForm {
    pub name: String,
    #[form(required)]
    pub slug: Option<String>,
    pub description: Option<String>,
    #[form(file, optional)]
    pub image: String,
    #[form(file)]
    pub gallery: Vec<String>,
    pub price: Option<Decimal>,
    pub old_price: Option<Decimal>,
    pub cost: Option<Decimal>,
    #[form(required)]
    pub sku: Option<String>,
    #[form(required)]
    pub barcode: Option<String>,
    pub qty: u64,
    pub security_stock: u64,
    pub backorder: bool,
    pub requires_shipping: bool,
    pub is_visible: bool,
    pub published_at: Option<jiff::civil::Date>,
    #[form(relationship = BrandResource)]
    pub brand_id: Option<u64>,
    #[form(relationship = ProductCategoryResource)]
    pub categories: Vec<u64>,
}

/// What showing or hiding the selection asks for.
#[derive(FormInput)]
pub struct VisibilityChoice {
    #[form(options = VisibilityOptions)]
    pub is_visible: bool,
}

/// The two answers a visibility choice offers.
pub struct VisibilityOptions;

impl tablo::Options for VisibilityOptions {
    fn options() -> Vec<(String, tablo::Text)> {
        vec![
            ("true".to_string(), tablo::text!("label-visible")),
            ("false".to_string(), tablo::text!("label-hidden")),
        ]
    }
}

/// Shows or hides every product in the selection.
pub struct BulkSetProductVisibility;

impl Action<ProductResource> for BulkSetProductVisibility {
    type Input = VisibilityChoice;
    const NAME: &'static str = "set-visibility";
    const PLACES: Places = Places::BULK;

    async fn run(
        _cx: &Cx,
        products: &[Product],
        choice: VisibilityChoice,
        ex: &mut dyn toasty::Executor,
    ) -> Result<Outcome, WriteError<VisibilityChoiceField>> {
        for product in products {
            Product::filter_by_id(product.id)
                .update()
                .is_visible(choice.is_visible)
                .exec(&mut *ex)
                .await?;
        }
        Ok(Outcome::done())
    }

    fn label() -> Text {
        tablo::text!("action-set-visibility")
    }
}

/// What `AdjustProductPrice` asks for.
#[derive(FormInput)]
pub struct PriceChange {
    pub price: Decimal,
    pub old_price: Option<Decimal>,
}

/// Sets a product's price and its compare-at price.
pub struct AdjustProductPrice;

impl Action<ProductResource> for AdjustProductPrice {
    type Input = PriceChange;
    const NAME: &'static str = "adjust-price";
    const PLACES: Places = Places::ROW;

    async fn run(
        _cx: &Cx,
        products: &[Product],
        change: PriceChange,
        ex: &mut dyn toasty::Executor,
    ) -> Result<Outcome, WriteError<PriceChangeField>> {
        for product in products {
            Product::filter_by_id(product.id)
                .update()
                .price(Some(change.price))
                .old_price(change.old_price)
                .exec(&mut *ex)
                .await?;
        }
        Ok(Outcome::done())
    }
}

/// What `AdjustProductStock` asks for.
#[derive(FormInput)]
pub struct StockChange {
    pub qty: i64,
}

/// Sets how many of a product are in stock.
pub struct AdjustProductStock;

impl Action<ProductResource> for AdjustProductStock {
    type Input = StockChange;
    const NAME: &'static str = "adjust-stock";
    const PLACES: Places = Places::ROW;

    fn validate_input(_cx: &Cx, input: &StockChange) -> FieldErrors<StockChangeField> {
        let mut errors = FieldErrors::new();
        if input.qty < 0 {
            errors.add(
                StockChangeField::Qty,
                tablo::text!("error-quantity-negative"),
            );
        }
        errors
    }

    async fn run(
        _cx: &Cx,
        products: &[Product],
        change: StockChange,
        ex: &mut dyn toasty::Executor,
    ) -> Result<Outcome, WriteError<StockChangeField>> {
        for product in products {
            Product::filter_by_id(product.id)
                .update()
                .qty(change.qty.max(0) as u64)
                .exec(&mut *ex)
                .await?;
        }
        Ok(Outcome::done())
    }
}
