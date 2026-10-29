//! The shop resources: products, brands, categories, customers and orders.

use tablo::{
    Action, ActionLayout, BooleanColumn, Change, Color, ColumnToggle, ColumnWidth, Currency,
    DateColumn, DateFilter, Detail, FieldErrors, FormInput, GlobalSearch, Grid, ImageColumn,
    Outcome, Places, Reads, Relation, RelationColumn, Resource, ResourceDef, RichTextColumn,
    Schema, Section, SelectFilter, Table, TernaryFilter, Text, TextColumn, WriteError,
    extend::{Column, Includes},
    lens, relation,
};
use topcoat::{Result, context::Cx};

use crate::blog::CommentResource;
use crate::dashboards::PipelineWidget;
use crate::enums::{CountryCode, OrderStatus};
use crate::models::replica_suffix;
use crate::models::{
    Address, Brand, Comment, Customer, Order, OrderAddress, OrderItem, Payment, Product,
    ProductCategory,
};
use crate::panel::{IMAGE_EXTENSIONS, IMAGE_MAX_BYTES};
use crate::roles::{self, Domain};
use crate::tabs::OptionTabs;

/// Products.
pub struct ProductResource;

impl Resource for ProductResource {
    type Model = Product;
    type Form = ProductForm;

    fn declare() -> ResourceDef<Self> {
        let c = ProductForm::controls();
        ResourceDef::new()
            .icon(crate::icons::PRODUCTS)
            .navigation_group(tablo::text!("navigation-shop"))
            .navigation_order(0)
            .navigation_badge(
                Product::fields()
                    .qty()
                    .lt(Product::fields().security_stock()),
            )
            .policy(roles::policy(Domain::Shop))
            .table(product_table())
            .form(Schema::new((
                Section::new(tablo::text!("section-product")).schema((
                    Grid::new(2).schema((c.name, c.slug.disabled_on_edit())),
                    c.description.rich_text(),
                )),
                Section::new(tablo::text!("section-images")).schema(
                    c.image
                        .extensions(IMAGE_EXTENSIONS)
                        .max_bytes(IMAGE_MAX_BYTES),
                ),
                Section::new(tablo::text!("section-pricing")).schema(
                    Grid::new(2).schema((
                        c.price.prefix("$"),
                        c.old_price
                            .prefix("$")
                            .label(tablo::text!("label-compare-at-price")),
                        c.cost
                            .prefix("$")
                            .label(tablo::text!("label-cost-per-item")),
                    )),
                ),
                Section::new(tablo::text!("section-inventory")).schema(
                    Grid::new(2).schema((
                        c.sku.label(tablo::text!("label-sku-stock-keeping-unit")),
                        c.barcode
                            .label(tablo::text!("label-barcode-isbn-upc-gtin-etc")),
                        c.qty.label(tablo::text!("label-quantity")),
                        c.security_stock,
                    )),
                ),
                Section::new(tablo::text!("section-shipping")).schema(
                    Grid::new(2).schema((c.backorder.switch(), c.requires_shipping.switch())),
                ),
                Section::new(tablo::text!("section-status")).schema(
                    Grid::new(2).schema((
                        c.is_visible
                            .switch()
                            .label(tablo::text!("label-visibility")),
                        c.published_at,
                    )),
                ),
                Section::new(tablo::text!("section-associations")).schema((
                    c.brand_id.searchable().label(tablo::text!("label-brand")),
                    c.categories.searchable(),
                )),
            )))
            .detail(Detail::new((Section::new(tablo::text!("section-product"))
                .columns((
                    TextColumn::new(lens!(Product.name)),
                    TextColumn::new(lens!(Product.slug)),
                    ImageColumn::new(lens!(Product.image)),
                    RelationColumn::of::<BrandResource>(relation!(Product.brand))
                        .label(tablo::text!("label-brand")),
                    RelationColumn::list::<ProductCategoryResource>(relation!(Product.categories)),
                    BooleanColumn::new(lens!(Product.is_visible))
                        .label(tablo::text!("label-visibility")),
                    TextColumn::new(lens!(Product.price)).money(Currency::USD),
                    TextColumn::new(lens!(Product.sku)).label(tablo::text!("label-sku")),
                    TextColumn::new(lens!(Product.qty))
                        .numeric(0)
                        .label(tablo::text!("label-quantity")),
                    DateColumn::new(lens!(Product.published_at))
                        .label(tablo::text!("label-publishing-date")),
                    RichTextColumn::new(lens!(Product.description)),
                )),)))
            .record_title(lens!(Product.name))
            .global_search(
                GlobalSearch::new()
                    .detail(tablo::text!("label-sku"), lens!(Product.sku))
                    .detail(tablo::text!("label-price"), lens!(Product.price)),
            )
            .relation(Relation::has_many::<CommentResource>(
                Comment::fields().product_id(),
            ))
            .action::<ToggleProductVisibility>()
            .action::<BulkSetProductVisibility>()
            .action::<AdjustProductPrice>()
            .action::<AdjustProductStock>()
    }

    crate::audit!();
}

/// The product list.
fn product_table() -> Table<Product> {
    Table::new((
        ImageColumn::new(lens!(Product.image)),
        TextColumn::new(lens!(Product.name)).searchable().sortable(),
        RelationColumn::of::<BrandResource>(relation!(Product.brand))
            .label(tablo::text!("label-brand"))
            .toggleable(),
        BooleanColumn::new(lens!(Product.is_visible))
            .label(tablo::text!("label-visibility"))
            .toggleable(),
        TextColumn::new(lens!(Product.price))
            .money(Currency::USD)
            .sortable(),
        TextColumn::new(lens!(Product.sku))
            .label(tablo::text!("label-sku"))
            .searchable()
            .sortable()
            .toggleable(),
        TextColumn::new(lens!(Product.qty))
            .numeric(0)
            .label(tablo::text!("label-quantity"))
            .sortable()
            .toggleable(),
        TextColumn::new(lens!(Product.security_stock))
            .numeric(0)
            .sortable()
            .toggleable()
            .hidden_by_default(),
        DateColumn::new(lens!(Product.published_at))
            .label(tablo::text!("label-publishing-date"))
            .sortable()
            .toggleable()
            .hidden_by_default(),
    ))
    .default_sort(lens!(Product.name))
    .action_layout(ActionLayout::Menu)
    .filters((
        SelectFilter::relationship::<BrandResource>(lens!(Product.brand_id))
            .label(tablo::text!("label-brand")),
        TernaryFilter::new(lens!(Product.is_visible)).label(tablo::text!("label-visibility")),
        TernaryFilter::new(lens!(Product.featured)),
        TernaryFilter::new(lens!(Product.backorder)),
        TernaryFilter::new(lens!(Product.requires_shipping)),
        DateFilter::new(lens!(Product.published_at)).label(tablo::text!("label-publishing-date")),
    ))
}

/// The product's record form.
#[derive(tablo::RecordForm)]
#[form(model = Product)]
pub struct ProductForm {
    pub name: String,
    pub slug: Option<String>,
    pub description: Option<String>,
    #[form(file, optional)]
    pub image: String,
    pub price: Option<f64>,
    pub old_price: Option<f64>,
    pub cost: Option<f64>,
    pub sku: Option<String>,
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

/// Shows or hides a product.
pub struct ToggleProductVisibility;

impl Action<ProductResource> for ToggleProductVisibility {
    type Input = ();
    const NAME: &'static str = "toggle-visibility";
    const PLACES: Places = Places::ROW;

    async fn run(
        _cx: &Cx,
        products: &[Product],
        (): (),
        ex: &mut dyn toasty::Executor,
    ) -> Result<Outcome, WriteError> {
        for product in products {
            Product::filter(Product::fields().id().eq(product.id))
                .update()
                .is_visible(!product.is_visible)
                .exec(&mut *ex)
                .await?;
        }
        Ok(Outcome::done())
    }
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
    ) -> Result<Outcome, WriteError> {
        for product in products {
            Product::filter(Product::fields().id().eq(product.id))
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
    pub price: f64,
    pub old_price: Option<f64>,
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
    ) -> Result<Outcome, WriteError> {
        for product in products {
            Product::filter(Product::fields().id().eq(product.id))
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
    ) -> Result<Outcome, WriteError> {
        for product in products {
            Product::filter(Product::fields().id().eq(product.id))
                .update()
                .qty(change.qty.max(0) as u64)
                .exec(&mut *ex)
                .await?;
        }
        Ok(Outcome::done())
    }
}

/// Brands.
pub struct BrandResource;

impl Resource for BrandResource {
    type Model = Brand;
    type Form = BrandForm;

    fn declare() -> ResourceDef<Self> {
        let c = BrandForm::controls();
        ResourceDef::new()
            .icon(crate::icons::BRANDS)
            .navigation_group(tablo::text!("navigation-shop"))
            .navigation_order(3)
            .policy(roles::policy(Domain::Shop))
            .table(
                Table::new((
                    TextColumn::new(lens!(Brand.name)).searchable().sortable(),
                    TextColumn::new(lens!(Brand.website))
                        .searchable()
                        .sortable()
                        .toggleable(),
                    BooleanColumn::new(lens!(Brand.is_visible))
                        .label(tablo::text!("label-visibility")),
                    DateColumn::new(lens!(Brand.updated_at))
                        .label(tablo::text!("label-last-modified-at"))
                        .sortable()
                        .toggleable()
                        .hidden_by_default(),
                ))
                .action_layout(ActionLayout::Menu),
            )
            .detail(Detail::new(
                Section::new(tablo::text!("section-brand")).columns((
                    TextColumn::new(lens!(Brand.name)),
                    TextColumn::new(lens!(Brand.slug)),
                    TextColumn::new(lens!(Brand.website)),
                    TextColumn::new(lens!(Brand.description)),
                    BooleanColumn::new(lens!(Brand.is_visible))
                        .label(tablo::text!("label-visibility")),
                    DateColumn::new(lens!(Brand.updated_at))
                        .label(tablo::text!("label-last-modified-at")),
                )),
            ))
            .form(Schema::new(
                Section::new(tablo::text!("section-brand")).schema((
                    Grid::new(2).schema((c.name, c.slug.disabled_on_edit())),
                    c.website.url(),
                    c.is_visible
                        .switch()
                        .label(tablo::text!("label-visibility")),
                    c.description.multiline(4),
                )),
            ))
            .record_title(lens!(Brand.name))
            .relation(Relation::has_many::<ProductResource>(
                Product::fields().brand_id(),
            ))
            .relation(Relation::has_many::<AddressResource>(
                Address::fields().brand_id(),
            ))
            .action::<ToggleBrandVisibility>()
    }

    crate::audit!();
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

/// Shows or hides a brand.
pub struct ToggleBrandVisibility;

impl Action<BrandResource> for ToggleBrandVisibility {
    type Input = ();
    const NAME: &'static str = "toggle-visibility";
    const PLACES: Places = Places::ROW;

    async fn run(
        _cx: &Cx,
        brands: &[Brand],
        (): (),
        ex: &mut dyn toasty::Executor,
    ) -> Result<Outcome, WriteError> {
        for brand in brands {
            Brand::filter(Brand::fields().id().eq(brand.id))
                .update()
                .is_visible(!brand.is_visible)
                .exec(&mut *ex)
                .await?;
        }
        Ok(Outcome::done())
    }
}

/// Product categories.
pub struct ProductCategoryResource;

impl Resource for ProductCategoryResource {
    type Model = ProductCategory;
    type Form = ProductCategoryForm;

    fn declare() -> ResourceDef<Self> {
        let c = ProductCategoryForm::controls();
        ResourceDef::new()
            .icon(crate::icons::PRODUCT_CATEGORIES)
            .navigation_group(tablo::text!("navigation-shop"))
            .navigation_order(4)
            .importable()
            .policy(roles::policy(Domain::Shop))
            .table(
                Table::new((
                    TextColumn::new(lens!(ProductCategory.name))
                        .searchable()
                        .sortable(),
                    RelationColumn::of::<ProductCategoryResource>(relation!(
                        ProductCategory.parent
                    ))
                    .label(tablo::text!("label-parent"))
                    .toggleable(),
                    BooleanColumn::new(lens!(ProductCategory.is_visible))
                        .label(tablo::text!("label-visibility")),
                    DateColumn::new(lens!(ProductCategory.updated_at))
                        .label(tablo::text!("label-last-modified-at"))
                        .sortable()
                        .toggleable()
                        .hidden_by_default(),
                ))
                .default_sort(lens!(ProductCategory.name))
                .action_layout(ActionLayout::Menu),
            )
            .detail(Detail::new(
                Section::new(tablo::text!("section-category")).columns((
                    TextColumn::new(lens!(ProductCategory.name)),
                    TextColumn::new(lens!(ProductCategory.slug)),
                    RelationColumn::of::<ProductCategoryResource>(relation!(
                        ProductCategory.parent
                    ))
                    .label(tablo::text!("label-parent")),
                    BooleanColumn::new(lens!(ProductCategory.is_visible))
                        .label(tablo::text!("label-visibility")),
                    TextColumn::new(lens!(ProductCategory.description)),
                    DateColumn::new(lens!(ProductCategory.updated_at))
                        .label(tablo::text!("label-last-modified-at")),
                )),
            ))
            .form(Schema::new(
                Section::new(tablo::text!("section-category")).schema((
                    Grid::new(2).schema((c.name, c.slug.disabled_on_edit())),
                    c.parent_id.label(tablo::text!("label-parent-category")),
                    c.is_visible
                        .switch()
                        .label(tablo::text!("label-visibility")),
                    c.description.multiline(4),
                )),
            ))
            .record_title(lens!(ProductCategory.name))
            .relation(Relation::belongs_to_many::<ProductResource>(
                ProductCategory::fields().products(),
            ))
            .action::<ToggleCategoryVisibility>()
    }

    crate::audit!();
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

/// Shows or hides a category.
pub struct ToggleCategoryVisibility;

impl Action<ProductCategoryResource> for ToggleCategoryVisibility {
    type Input = ();
    const NAME: &'static str = "toggle-visibility";
    const PLACES: Places = Places::ROW;

    async fn run(
        _cx: &Cx,
        categories: &[ProductCategory],
        (): (),
        ex: &mut dyn toasty::Executor,
    ) -> Result<Outcome, WriteError> {
        for category in categories {
            ProductCategory::filter(ProductCategory::fields().id().eq(category.id))
                .update()
                .is_visible(!category.is_visible)
                .exec(&mut *ex)
                .await?;
        }
        Ok(Outcome::done())
    }
}

/// Customers.
pub struct CustomerResource;

impl Resource for CustomerResource {
    type Model = Customer;
    type Form = CustomerForm;

    fn declare() -> ResourceDef<Self> {
        let c = CustomerForm::controls();
        ResourceDef::new()
            .icon(crate::icons::CUSTOMERS)
            .navigation_group(tablo::text!("navigation-shop"))
            .navigation_order(2)
            .policy(roles::policy(Domain::Shop))
            .table(
                Table::new((
                    TextColumn::new(lens!(Customer.name))
                        .searchable()
                        .sortable(),
                    TextColumn::new(lens!(Customer.email))
                        .label(tablo::text!("label-email-address"))
                        .searchable()
                        .sortable(),
                    CustomerCountryColumn::new(),
                    TextColumn::new(lens!(Customer.phone))
                        .searchable()
                        .sortable()
                        .toggleable(),
                ))
                .default_sort(lens!(Customer.name))
                .action_layout(ActionLayout::Menu),
            )
            .detail(Detail::new(
                Section::new(tablo::text!("section-customer")).columns((
                    TextColumn::new(lens!(Customer.name)),
                    TextColumn::new(lens!(Customer.email))
                        .label(tablo::text!("label-email-address")),
                    TextColumn::new(lens!(Customer.phone)),
                    TextColumn::new(lens!(Customer.birthday)),
                )),
            ))
            .form(Schema::new(
                Section::new(tablo::text!("section-customer")).schema((
                    Grid::new(2).schema((
                        c.name,
                        c.email
                            .email()
                            .unique()
                            .label(tablo::text!("label-email-address")),
                    )),
                    Grid::new(2).schema((c.phone.tel(), c.birthday)),
                )),
            ))
            .record_title(lens!(Customer.name))
            .global_search(
                GlobalSearch::new()
                    .detail(tablo::text!("label-email"), lens!(Customer.email))
                    .detail(tablo::text!("label-phone"), lens!(Customer.phone)),
            )
            .relation(Relation::has_many::<AddressResource>(
                Address::fields().customer_id(),
            ))
            .action::<SendCustomerEmail>()
            .soft_delete(lens!(Customer.deleted_at))
    }

    crate::audit!();
}

/// The customer's country, the first address's.
fn customer_country(customer: &Customer) -> Option<CountryCode> {
    if customer.addresses.is_unloaded() {
        return None;
    }
    customer
        .addresses
        .get()
        .first()
        .and_then(|address| address.country)
}

/// The customer list's country column: the first address's country, labelled in
/// the request's locale.
struct CustomerCountryColumn {
    includes: Includes<Customer>,
}

impl CustomerCountryColumn {
    fn new() -> Self {
        Self {
            includes: Includes::new().with(Customer::fields().addresses()),
        }
    }
}

impl Column<Customer> for CustomerCountryColumn {
    fn name(&self) -> &str {
        "country"
    }

    fn label(&self) -> Text {
        tablo::text!("field-country")
    }

    fn text(&self, cx: &Cx, customer: &Customer) -> String {
        customer_country(customer)
            .map(|country| country.label().localize(cx))
            .unwrap_or_default()
    }

    fn toggle(&self) -> ColumnToggle {
        ColumnToggle::ShownByDefault
    }

    fn includes(&self) -> Includes<Customer> {
        self.includes.clone()
    }
}

/// The customer's record form.
#[derive(tablo::RecordForm)]
#[form(model = Customer)]
pub struct CustomerForm {
    pub name: String,
    pub email: String,
    pub phone: Option<String>,
    pub birthday: Option<jiff::civil::Date>,
}

/// What `SendCustomerEmail` asks for.
#[derive(FormInput)]
pub struct EmailDraft {
    pub subject: String,
    #[form(multiline = 5)]
    pub body: String,
}

/// Reports an email to the customer, as the demo's action does without sending one.
pub struct SendCustomerEmail;

impl Action<CustomerResource> for SendCustomerEmail {
    type Input = EmailDraft;
    const NAME: &'static str = "send-email";
    const PLACES: Places = Places::ROW.with(Places::DETAIL).with(Places::EDIT);

    async fn run(
        _cx: &Cx,
        _customers: &[Customer],
        _draft: EmailDraft,
        _ex: &mut dyn toasty::Executor,
    ) -> Result<Outcome, WriteError> {
        Ok(Outcome::done())
    }

    fn success(count: usize) -> Text {
        tablo::text!("flash-email-sent", count = count)
    }
}

/// Orders.
pub struct OrderResource;

impl Resource for OrderResource {
    type Model = Order;
    type Form = OrderForm;

    fn declare() -> ResourceDef<Self> {
        let c = OrderForm::controls();
        ResourceDef::new()
            .icon(crate::icons::ORDERS)
            .navigation_group(tablo::text!("navigation-shop"))
            .navigation_order(1)
            .navigation_badge(Order::fields().status().eq(OrderStatus::New))
            .policy(roles::policy(Domain::Shop))
            .table(order_table())
            .detail(Detail::new(
                Section::new(tablo::text!("section-order")).columns((
                    Grid::new(2).columns((
                        TextColumn::new(lens!(Order.number)),
                        RelationColumn::of::<CustomerResource>(relation!(Order.customer))
                            .label(tablo::text!("label-customer")),
                        TextColumn::new(lens!(Order.status)).badge(),
                        TextColumn::new(lens!(Order.total_price)).money(Currency::USD),
                        TextColumn::new(lens!(Order.shipping_price))
                            .money(Currency::USD)
                            .label(tablo::text!("label-shipping-cost")),
                        DateColumn::new(lens!(Order.created_at))
                            .label(tablo::text!("label-order-date")),
                    )),
                    RichTextColumn::new(lens!(Order.notes)),
                )),
            ))
            .form(Schema::new(
                Section::new(tablo::text!("section-order")).schema(
                    Grid::new(2).schema((
                        c.number.label(tablo::text!("label-number")),
                        c.customer_id
                            .searchable()
                            .label(tablo::text!("label-customer")),
                        c.status.buttons(),
                        c.notes.rich_text(),
                    )),
                ),
            ))
            .record_title(lens!(Order.number))
            .global_search(
                GlobalSearch::new()
                    .detail(tablo::text!("label-status"), lens!(Order.status))
                    .detail(tablo::text!("label-total"), lens!(Order.total_price)),
            )
            .relation(Relation::has_many::<OrderItemResource>(
                OrderItem::fields().order_id(),
            ))
            .relation(Relation::has_many::<OrderAddressResource>(
                OrderAddress::fields().order_id(),
            ))
            .relation(Relation::has_many::<PaymentResource>(
                Payment::fields().order_id(),
            ))
            .action::<ProcessOrder>()
            .action::<ShipOrder>()
            .action::<DeliverOrder>()
            .action::<ReplicateOrder>()
            .action::<CancelOrder>()
            .soft_delete(lens!(Order.deleted_at))
            // The pipeline the shop dashboard counts, above the orders it counts.
            .list_widget::<PipelineWidget>()
    }

    /// Tells the signed-in user about a new order, once the write committed.
    async fn after_commit(cx: &Cx, change: Change<Order>) -> Result<()> {
        if let Change::Create(order) = &change {
            crate::notifications::new_order(cx, order).await?;
        }
        Ok(())
    }

    crate::audit!();
}

/// The order list.
fn order_table() -> Table<Order> {
    Table::new((
        TextColumn::new(lens!(Order.number)).searchable().sortable(),
        RelationColumn::of::<CustomerResource>(relation!(Order.customer))
            .label(tablo::text!("label-customer"))
            .toggleable(),
        TextColumn::new(lens!(Order.status)).badge(),
        TextColumn::new(lens!(Order.total_price)).money(Currency::USD),
        TextColumn::new(lens!(Order.shipping_price))
            .money(Currency::USD)
            .label(tablo::text!("label-shipping-cost"))
            .toggleable(),
        DateColumn::new(lens!(Order.created_at))
            .label(tablo::text!("label-order-date"))
            .sortable()
            .toggleable(),
    ))
    .default_sort_desc(lens!(Order.created_at))
    .action_layout(ActionLayout::Menu)
    .row_color(|order: &Order| (order.status == OrderStatus::Cancelled).then_some(Color::Danger))
    .row_color_reading(Reads::of(lens!(Order.status)))
    .option_tabs(|status: OrderStatus| Order::fields().status().eq(status))
    .filters((DateFilter::new(lens!(Order.created_at))
        .label(tablo::text!("label-order-date"))
        .range(),))
}

/// The order's record form.
#[derive(tablo::RecordForm)]
#[form(model = Order)]
pub struct OrderForm {
    pub number: String,
    #[form(relationship = CustomerResource)]
    pub customer_id: Option<u64>,
    #[form(options)]
    pub status: OrderStatus,
    pub notes: Option<String>,
}

/// Moves a new order to processing.
pub struct ProcessOrder;

impl Action<OrderResource> for ProcessOrder {
    type Input = ();
    const NAME: &'static str = "process";
    const PLACES: Places = Places::ROW;

    fn can_run(_cx: &Cx, order: &Order) -> bool {
        order.status == OrderStatus::New
    }

    async fn run(
        _cx: &Cx,
        orders: &[Order],
        (): (),
        ex: &mut dyn toasty::Executor,
    ) -> Result<Outcome, WriteError> {
        set_status(orders, OrderStatus::Processing, ex).await
    }

    fn success(_count: usize) -> Text {
        tablo::text!("flash-order-processing")
    }
}

/// What `ShipOrder` asks for.
#[derive(FormInput)]
pub struct ShippingNotes {
    #[form(multiline = 3)]
    pub shipping_notes: Option<String>,
}

/// Ships a processing order.
pub struct ShipOrder;

impl Action<OrderResource> for ShipOrder {
    type Input = ShippingNotes;
    const NAME: &'static str = "ship";
    const PLACES: Places = Places::ROW;

    fn can_run(_cx: &Cx, order: &Order) -> bool {
        order.status == OrderStatus::Processing
    }

    async fn run(
        _cx: &Cx,
        orders: &[Order],
        shipping: ShippingNotes,
        ex: &mut dyn toasty::Executor,
    ) -> Result<Outcome, WriteError> {
        for order in orders {
            Order::filter(Order::fields().id().eq(order.id))
                .update()
                .status(OrderStatus::Shipped)
                .notes(shipping.shipping_notes.clone())
                .exec(&mut *ex)
                .await?;
        }
        Ok(Outcome::done())
    }

    fn success(_count: usize) -> Text {
        tablo::text!("flash-order-shipped")
    }
}

/// Marks a shipped order delivered, confirming first.
pub struct DeliverOrder;

impl Action<OrderResource> for DeliverOrder {
    type Input = ();
    const NAME: &'static str = "deliver";
    const PLACES: Places = Places::ROW;
    const CONFIRM: bool = true;

    fn can_run(_cx: &Cx, order: &Order) -> bool {
        order.status == OrderStatus::Shipped
    }

    async fn run(
        _cx: &Cx,
        orders: &[Order],
        (): (),
        ex: &mut dyn toasty::Executor,
    ) -> Result<Outcome, WriteError> {
        set_status(orders, OrderStatus::Delivered, ex).await
    }

    fn success(_count: usize) -> Text {
        tablo::text!("flash-order-delivered")
    }
}

/// Copies an order as a new one under a fresh number, confirming first.
pub struct ReplicateOrder;

impl Action<OrderResource> for ReplicateOrder {
    type Input = ();
    const NAME: &'static str = "replicate";
    const PLACES: Places = Places::EDIT;
    const CONFIRM: bool = true;

    async fn run(
        _cx: &Cx,
        orders: &[Order],
        (): (),
        ex: &mut dyn toasty::Executor,
    ) -> Result<Outcome, WriteError> {
        let date = jiff::Zoned::now().strftime("%Y%m%d").to_string();
        for order in orders {
            toasty::create!(Order {
                customer_id: order.customer_id,
                number: format!("OR-{date}-{}", replica_suffix()),
                total_price: order.total_price,
                shipping_price: order.shipping_price,
                shipping_method: order.shipping_method.clone(),
                notes: order.notes.clone(),
            })
            .exec(&mut *ex)
            .await?;
        }
        Ok(Outcome::done())
    }

    fn success(_count: usize) -> Text {
        tablo::text!("flash-order-replicated")
    }
}

/// Cancels an order that is neither delivered nor already cancelled.
pub struct CancelOrder;

impl Action<OrderResource> for CancelOrder {
    type Input = ();
    const NAME: &'static str = "cancel";
    const PLACES: Places = Places::ROW;
    const CONFIRM: bool = true;

    fn can_run(_cx: &Cx, order: &Order) -> bool {
        !matches!(
            order.status,
            OrderStatus::Delivered | OrderStatus::Cancelled
        )
    }

    async fn run(
        _cx: &Cx,
        orders: &[Order],
        (): (),
        ex: &mut dyn toasty::Executor,
    ) -> Result<Outcome, WriteError> {
        set_status(orders, OrderStatus::Cancelled, ex).await
    }

    fn success(_count: usize) -> Text {
        tablo::text!("flash-order-cancelled")
    }
}

/// Moves every order of `orders` to `status`.
async fn set_status(
    orders: &[Order],
    status: OrderStatus,
    ex: &mut dyn toasty::Executor,
) -> Result<Outcome, WriteError> {
    for order in orders {
        Order::filter(Order::fields().id().eq(order.id))
            .update()
            .status(status)
            .exec(&mut *ex)
            .await?;
    }
    Ok(Outcome::done())
}

/// One line of an order, which the order page lists.
pub struct OrderItemResource;

impl Resource for OrderItemResource {
    type Model = OrderItem;
    type Form = OrderItemForm;

    fn declare() -> ResourceDef<Self> {
        let c = OrderItemForm::controls();
        ResourceDef::new()
            .icon(crate::icons::ORDER_ITEMS)
            .navigation_group(tablo::text!("navigation-shop"))
            .navigation_order(10)
            .policy(roles::policy(Domain::Shop))
            .without_global_search()
            .table(Table::new((
                RelationColumn::of::<ProductResource>(relation!(OrderItem.product))
                    .label(tablo::text!("label-product")),
                TextColumn::new(lens!(OrderItem.qty))
                    .numeric(0)
                    .label(tablo::text!("label-quantity")),
                TextColumn::new(lens!(OrderItem.unit_price))
                    .money(Currency::USD)
                    .label(tablo::text!("label-unit-price")),
            )))
            .detail(Detail::new(
                Section::new(tablo::text!("section-order-item")).columns((
                    RelationColumn::of::<OrderResource>(relation!(OrderItem.order))
                        .label(tablo::text!("label-order")),
                    RelationColumn::of::<ProductResource>(relation!(OrderItem.product))
                        .label(tablo::text!("label-product")),
                    TextColumn::new(lens!(OrderItem.qty))
                        .numeric(0)
                        .label(tablo::text!("label-quantity")),
                    TextColumn::new(lens!(OrderItem.unit_price))
                        .money(Currency::USD)
                        .label(tablo::text!("label-unit-price")),
                )),
            ))
            .form(Schema::new(
                Section::new(tablo::text!("section-order-item")).schema((
                    c.order_id.searchable().label(tablo::text!("label-order")),
                    c.product_id
                        .searchable()
                        .label(tablo::text!("label-product")),
                    Grid::new(2).schema((
                        c.qty.label(tablo::text!("label-quantity")),
                        c.unit_price.label(tablo::text!("label-unit-price")),
                    )),
                )),
            ))
    }

    crate::audit!();
}

/// The order item's record form.
#[derive(tablo::RecordForm)]
#[form(model = OrderItem)]
pub struct OrderItemForm {
    #[form(relationship = OrderResource)]
    pub order_id: Option<u64>,
    #[form(relationship = ProductResource)]
    pub product_id: Option<u64>,
    pub qty: i64,
    pub unit_price: f64,
}

/// Payments.
pub struct PaymentResource;

impl Resource for PaymentResource {
    type Model = Payment;
    type Form = PaymentForm;

    fn declare() -> ResourceDef<Self> {
        let c = PaymentForm::controls();
        ResourceDef::new()
            .icon(crate::icons::PAYMENTS)
            .navigation_group(tablo::text!("navigation-shop"))
            .navigation_order(11)
            .policy(roles::policy(Domain::Shop))
            .without_global_search()
            .table(Table::new((
                RelationColumn::of::<OrderResource>(relation!(Payment.order))
                    .label(tablo::text!("label-order")),
                TextColumn::new(lens!(Payment.reference)).searchable(),
                TextColumn::new(lens!(Payment.amount))
                    .money(Currency::USD)
                    .sortable(),
                TextColumn::new(lens!(Payment.provider)).sortable(),
                TextColumn::new(lens!(Payment.method)).sortable(),
            )))
            .detail(Detail::new(
                Section::new(tablo::text!("section-payment")).columns((
                    RelationColumn::of::<OrderResource>(relation!(Payment.order))
                        .label(tablo::text!("label-order")),
                    TextColumn::new(lens!(Payment.reference)),
                    TextColumn::new(lens!(Payment.amount)).money(Currency::USD),
                    TextColumn::new(lens!(Payment.currency)),
                    TextColumn::new(lens!(Payment.provider)),
                    TextColumn::new(lens!(Payment.method)),
                )),
            ))
            .form(Schema::new(
                Section::new(tablo::text!("section-payment")).schema((
                    c.order_id.searchable().label(tablo::text!("label-order")),
                    c.reference,
                    Grid::new(2).schema((c.amount, c.currency.searchable())),
                    c.provider,
                    c.method,
                )),
            ))
            .record_title(lens!(Payment.reference))
    }

    crate::audit!();
}

/// The payment's record form.
#[derive(tablo::RecordForm)]
#[form(model = Payment)]
pub struct PaymentForm {
    #[form(relationship = OrderResource)]
    pub order_id: u64,
    pub reference: String,
    pub amount: f64,
    #[form(options)]
    pub currency: crate::enums::CurrencyCode,
    #[form(options = Providers)]
    pub provider: String,
    #[form(options)]
    pub method: crate::enums::PaymentMethod,
}

/// The two payment providers the demo offers.
pub struct Providers;

impl tablo::Options for Providers {
    fn options() -> Vec<(String, tablo::Text)> {
        vec![
            ("stripe".to_string(), "Stripe".into()),
            ("paypal".to_string(), "PayPal".into()),
        ]
    }
}

/// Addresses, which the customer and brand pages list.
pub struct AddressResource;

impl Resource for AddressResource {
    type Model = Address;
    type Form = AddressForm;

    fn declare() -> ResourceDef<Self> {
        let c = AddressForm::controls();
        ResourceDef::new()
            .icon(crate::icons::ADDRESSES)
            .navigation_group(tablo::text!("navigation-shop"))
            .navigation_order(12)
            .policy(roles::policy(Domain::Shop))
            .without_global_search()
            .table(
                Table::new((
                    TextColumn::new(lens!(Address.street)),
                    TextColumn::new(lens!(Address.zip)),
                    TextColumn::new(lens!(Address.city)).sortable(),
                    TextColumn::new(lens!(Address.country)).width(ColumnWidth::Auto),
                ))
                .default_sort(lens!(Address.city)),
            )
            .detail(Detail::new(
                Section::new(tablo::text!("section-address")).columns((
                    TextColumn::new(lens!(Address.street)),
                    TextColumn::new(lens!(Address.city)),
                    TextColumn::new(lens!(Address.state)),
                    TextColumn::new(lens!(Address.zip)),
                    TextColumn::new(lens!(Address.country)),
                )),
            ))
            .form(Schema::new(
                Section::new(tablo::text!("section-address")).schema((
                    c.customer_id
                        .searchable()
                        .label(tablo::text!("label-customer")),
                    c.brand_id.searchable().label(tablo::text!("label-brand")),
                    c.country.searchable(),
                    c.street.label(tablo::text!("label-street-address")),
                    Grid::new(3).schema((
                        c.city.suggestions(crate::seed::CITIES.iter().copied()),
                        c.state.label(tablo::text!("label-state-province")),
                        c.zip.label(tablo::text!("label-zip-postal-code")),
                    )),
                )),
            ))
            .record_title(lens!(Address.street))
    }

    crate::audit!();
}

/// The address's record form.
#[derive(tablo::RecordForm)]
#[form(model = Address)]
pub struct AddressForm {
    #[form(relationship = CustomerResource)]
    pub customer_id: Option<u64>,
    #[form(relationship = BrandResource)]
    pub brand_id: Option<u64>,
    #[form(options)]
    pub country: Option<crate::enums::CountryCode>,
    pub street: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub zip: Option<String>,
}

/// Where an order ships, which the order page lists.
///
/// Tablo has no nested relationship form, so these fields are a resource of
/// their own rather than part of the order form.
pub struct OrderAddressResource;

impl Resource for OrderAddressResource {
    type Model = OrderAddress;
    type Form = OrderAddressForm;

    fn declare() -> ResourceDef<Self> {
        let c = OrderAddressForm::controls();
        ResourceDef::new()
            .icon(crate::icons::ORDER_ADDRESSES)
            .navigation_group(tablo::text!("navigation-shop"))
            .navigation_order(13)
            .policy(roles::policy(Domain::Shop))
            .without_global_search()
            .table(Table::new((
                RelationColumn::of::<OrderResource>(relation!(OrderAddress.order))
                    .label(tablo::text!("label-order")),
                TextColumn::new(lens!(OrderAddress.street)),
                TextColumn::new(lens!(OrderAddress.city)),
                TextColumn::new(lens!(OrderAddress.zip)),
                TextColumn::new(lens!(OrderAddress.country)).width(ColumnWidth::Auto),
            )))
            .detail(Detail::new(
                Section::new(tablo::text!("section-order-address")).columns((
                    RelationColumn::of::<OrderResource>(relation!(OrderAddress.order))
                        .label(tablo::text!("label-order")),
                    TextColumn::new(lens!(OrderAddress.street)),
                    TextColumn::new(lens!(OrderAddress.city)),
                    TextColumn::new(lens!(OrderAddress.state)),
                    TextColumn::new(lens!(OrderAddress.zip)),
                    TextColumn::new(lens!(OrderAddress.country)),
                )),
            ))
            .form(Schema::new(
                Section::new(tablo::text!("section-order-address")).schema((
                    c.order_id.searchable().label(tablo::text!("label-order")),
                    c.country.searchable(),
                    c.street.label(tablo::text!("label-street-address")),
                    Grid::new(3).schema((
                        c.city.suggestions(crate::seed::CITIES.iter().copied()),
                        c.state.label(tablo::text!("label-state-province")),
                        c.zip.label(tablo::text!("label-zip-postal-code")),
                    )),
                )),
            ))
            .record_title(lens!(OrderAddress.street))
    }

    crate::audit!();
}

/// The order address's record form.
#[derive(tablo::RecordForm)]
#[form(model = OrderAddress)]
pub struct OrderAddressForm {
    #[form(relationship = OrderResource)]
    pub order_id: Option<u64>,
    #[form(options)]
    pub country: Option<crate::enums::CountryCode>,
    pub street: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub zip: Option<String>,
}
