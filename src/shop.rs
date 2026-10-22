//! The shop resources: products, brands, categories, customers and orders.

use tablo::{
    Action, ActionLayout, Allow, BooleanColumn, Change, ColumnWidth, ComputedColumn, Currency,
    DateColumn, DateFilter, Detail, FieldErrors, FormInput, GlobalSearch, Grid, ImageColumn,
    Outcome, Places, Relation, RelationColumn, Resource, ResourceDef, RichTextColumn, Schema,
    Section, SelectFilter, Table, TernaryFilter, TextColumn, WriteError, lens, relation,
};
use topcoat::{Result, context::Cx};

use crate::blog::CommentResource;
use crate::enums::OrderStatus;
use crate::models::{
    Address, Brand, Comment, Customer, Order, OrderAddress, OrderItem, Payment, Product,
    ProductCategory,
};

/// Products.
pub struct ProductResource;

impl Resource for ProductResource {
    type Model = Product;
    type Form = ProductForm;

    fn declare() -> ResourceDef<Self> {
        let c = ProductForm::controls();
        ResourceDef::new()
            .icon(crate::icons::PRODUCTS)
            .navigation_group("Shop")
            .navigation_order(0)
            .policy(Allow)
            .table(product_table())
            .form(Schema::new((
                Section::new("Product").schema((
                    Grid::new(2).schema((c.name, c.slug)),
                    c.description.rich_text(),
                )),
                Section::new("Images").schema(c.image.label("Image")),
                Section::new("Pricing").schema(Grid::new(2).schema((
                    c.price.prefix("$"),
                    c.old_price.prefix("$").label("Compare at price"),
                    c.cost.prefix("$").label("Cost per item"),
                ))),
                Section::new("Inventory").schema(Grid::new(2).schema((
                    c.sku.label("SKU (Stock Keeping Unit)"),
                    c.barcode.label("Barcode (ISBN, UPC, GTIN, etc.)"),
                    c.qty.label("Quantity"),
                    c.security_stock,
                ))),
                Section::new("Shipping").schema(
                    Grid::new(2).schema((c.backorder.switch(), c.requires_shipping.switch())),
                ),
                Section::new("Status").schema(
                    Grid::new(2)
                        .schema((c.is_visible.switch().label("Visibility"), c.published_at)),
                ),
                Section::new("Associations").schema((
                    c.brand_id.searchable().label("Brand"),
                    c.categories.searchable(),
                )),
            )))
            .detail(Detail::new((Section::new("Product").columns((
                TextColumn::new(lens!(Product.name)),
                TextColumn::new(lens!(Product.slug)),
                ImageColumn::new(lens!(Product.image)).label("Image"),
                RelationColumn::of::<BrandResource>(relation!(Product.brand)).label("Brand"),
                RelationColumn::list::<ProductCategoryResource>(relation!(Product.categories))
                    .label("Categories"),
                BooleanColumn::new(lens!(Product.is_visible)).label("Visibility"),
                TextColumn::new(lens!(Product.price))
                    .money(Currency::USD)
                    .label("Price"),
                TextColumn::new(lens!(Product.sku)).label("SKU"),
                TextColumn::new(lens!(Product.qty))
                    .numeric(0)
                    .label("Quantity"),
                DateColumn::new(lens!(Product.published_at)).label("Publishing date"),
                RichTextColumn::new(lens!(Product.description)),
            )),)))
            .record_title(lens!(Product.name))
            .global_search(
                GlobalSearch::new()
                    .detail("SKU", lens!(Product.sku))
                    .detail("Price", lens!(Product.price)),
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
        ImageColumn::new(lens!(Product.image)).label("Image"),
        TextColumn::new(lens!(Product.name)).searchable().sortable(),
        RelationColumn::of::<BrandResource>(relation!(Product.brand))
            .label("Brand")
            .toggleable(),
        BooleanColumn::new(lens!(Product.is_visible))
            .label("Visibility")
            .toggleable(),
        TextColumn::new(lens!(Product.price))
            .money(Currency::USD)
            .sortable()
            .label("Price"),
        TextColumn::new(lens!(Product.sku))
            .label("SKU")
            .searchable()
            .sortable()
            .toggleable(),
        TextColumn::new(lens!(Product.qty))
            .numeric(0)
            .label("Quantity")
            .sortable()
            .toggleable(),
        TextColumn::new(lens!(Product.security_stock))
            .numeric(0)
            .sortable()
            .toggleable()
            .hidden_by_default(),
        DateColumn::new(lens!(Product.published_at))
            .label("Publishing date")
            .sortable()
            .toggleable()
            .hidden_by_default(),
    ))
    .default_sort(lens!(Product.name))
    .action_layout(ActionLayout::Menu)
    .filters((
        SelectFilter::relationship::<BrandResource>(lens!(Product.brand_id)).label("Brand"),
        TernaryFilter::new(lens!(Product.is_visible)).label("Visibility"),
        TernaryFilter::new(lens!(Product.featured)),
        TernaryFilter::new(lens!(Product.backorder)),
        TernaryFilter::new(lens!(Product.requires_shipping)),
        DateFilter::new(lens!(Product.published_at)).label("Publishing date"),
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
    #[form(options = VisibilityOptions, label = "Visibility")]
    pub is_visible: bool,
}

/// The two answers a visibility choice offers.
pub struct VisibilityOptions;

impl tablo::Options for VisibilityOptions {
    fn options() -> Vec<(String, String)> {
        vec![
            ("true".to_string(), "Visible".to_string()),
            ("false".to_string(), "Hidden".to_string()),
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

    fn label(_cx: &Cx) -> String {
        "Set visibility".to_string()
    }
}

/// What `AdjustProductPrice` asks for.
#[derive(FormInput)]
pub struct PriceChange {
    #[form(label = "Price")]
    pub price: f64,
    #[form(label = "Compare at price")]
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
    #[form(label = "Quantity")]
    pub qty: i64,
}

/// Sets how many of a product are in stock.
pub struct AdjustProductStock;

impl Action<ProductResource> for AdjustProductStock {
    type Input = StockChange;
    const NAME: &'static str = "adjust-stock";
    const PLACES: Places = Places::ROW;

    fn validate_input(_cx: &Cx, input: &StockChange) -> FieldErrors {
        let mut errors = FieldErrors::new();
        if input.qty < 0 {
            errors.add("qty", "Quantity must be zero or more");
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
            .navigation_group("Shop")
            .navigation_order(3)
            .policy(Allow)
            .table(
                Table::new((
                    TextColumn::new(lens!(Brand.name)).searchable().sortable(),
                    TextColumn::new(lens!(Brand.website))
                        .searchable()
                        .sortable(),
                    BooleanColumn::new(lens!(Brand.is_visible)).label("Visibility"),
                    DateColumn::new(lens!(Brand.updated_at))
                        .label("Last modified at")
                        .sortable(),
                ))
                .action_layout(ActionLayout::Menu),
            )
            .detail(Detail::new(Section::new("Brand").columns((
                TextColumn::new(lens!(Brand.name)),
                TextColumn::new(lens!(Brand.slug)),
                TextColumn::new(lens!(Brand.website)),
                TextColumn::new(lens!(Brand.description)),
                BooleanColumn::new(lens!(Brand.is_visible)).label("Visibility"),
                DateColumn::new(lens!(Brand.updated_at)).label("Last modified at"),
            ))))
            .form(Schema::new(Section::new("Brand").schema((
                Grid::new(2).schema((c.name, c.slug)),
                c.website.url(),
                c.is_visible.switch().label("Visibility"),
                c.description.multiline(4),
            ))))
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
            .label("Product category")
            .plural_label("Product categories")
            .icon(crate::icons::PRODUCT_CATEGORIES)
            .navigation_group("Shop")
            .navigation_order(4)
            .importable()
            .policy(Allow)
            .table(
                Table::new((
                    TextColumn::new(lens!(ProductCategory.name))
                        .searchable()
                        .sortable(),
                    RelationColumn::of::<ProductCategoryResource>(relation!(
                        ProductCategory.parent
                    ))
                    .label("Parent"),
                    BooleanColumn::new(lens!(ProductCategory.is_visible)).label("Visibility"),
                    DateColumn::new(lens!(ProductCategory.updated_at))
                        .label("Last modified at")
                        .sortable(),
                ))
                .default_sort(lens!(ProductCategory.name))
                .action_layout(ActionLayout::Menu),
            )
            .detail(Detail::new(
                Section::new("Category").columns((
                    TextColumn::new(lens!(ProductCategory.name)),
                    TextColumn::new(lens!(ProductCategory.slug)),
                    RelationColumn::of::<ProductCategoryResource>(relation!(
                        ProductCategory.parent
                    ))
                    .label("Parent"),
                    BooleanColumn::new(lens!(ProductCategory.is_visible)).label("Visibility"),
                    TextColumn::new(lens!(ProductCategory.description)),
                    DateColumn::new(lens!(ProductCategory.updated_at)).label("Last modified at"),
                )),
            ))
            .form(Schema::new(Section::new("Category").schema((
                Grid::new(2).schema((c.name, c.slug)),
                c.parent_id.label("Parent category"),
                c.is_visible.switch().label("Visibility"),
                c.description.multiline(4),
            ))))
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
            .navigation_group("Shop")
            .navigation_order(2)
            .policy(Allow)
            .table(
                Table::new((
                    TextColumn::new(lens!(Customer.name))
                        .searchable()
                        .sortable(),
                    TextColumn::new(lens!(Customer.email))
                        .label("Email address")
                        .searchable()
                        .sortable(),
                    ComputedColumn::new("Country", |customer: &Customer| {
                        customer_country(customer).unwrap_or_default()
                    })
                    .include_unscoped(Customer::fields().addresses()),
                    TextColumn::new(lens!(Customer.phone))
                        .searchable()
                        .sortable(),
                ))
                .default_sort(lens!(Customer.name))
                .action_layout(ActionLayout::Menu),
            )
            .detail(Detail::new(Section::new("Customer").columns((
                TextColumn::new(lens!(Customer.name)),
                TextColumn::new(lens!(Customer.email)).label("Email address"),
                TextColumn::new(lens!(Customer.phone)),
                TextColumn::new(lens!(Customer.birthday)),
            ))))
            .form(Schema::new(Section::new("Customer").schema((
                Grid::new(2).schema((c.name, c.email.email().unique().label("Email address"))),
                Grid::new(2).schema((c.phone.tel(), c.birthday)),
            ))))
            .record_title(lens!(Customer.name))
            .global_search(
                GlobalSearch::new()
                    .detail("Email", lens!(Customer.email))
                    .detail("Phone", lens!(Customer.phone)),
            )
            .relation(Relation::has_many::<AddressResource>(
                Address::fields().customer_id(),
            ))
            .action::<SendCustomerEmail>()
            .soft_delete(lens!(Customer.deleted_at))
    }

    crate::audit!();
}

/// The first address's country, as the list's country column reads it.
fn customer_country(customer: &Customer) -> Option<String> {
    if customer.addresses.is_unloaded() {
        return None;
    }
    customer
        .addresses
        .get()
        .first()
        .and_then(|address| address.country)
        .map(|country| country.label().to_string())
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
    #[form(label = "Subject")]
    pub subject: String,
    #[form(label = "Message", multiline = 5)]
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

    fn success(_cx: &Cx, count: usize) -> String {
        format!("Email sent to {count} customer(s)")
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
            .navigation_group("Shop")
            .navigation_order(1)
            .navigation_badge(Order::fields().status().eq(OrderStatus::New))
            .policy(Allow)
            .table(order_table())
            .detail(Detail::new(
                Section::new("Order").columns((
                    Grid::new(2).columns((
                        TextColumn::new(lens!(Order.number)),
                        RelationColumn::of::<CustomerResource>(relation!(Order.customer))
                            .label("Customer"),
                        TextColumn::new(lens!(Order.status)).badge(),
                        TextColumn::new(lens!(Order.total_price)).money(Currency::USD),
                        TextColumn::new(lens!(Order.shipping_price))
                            .money(Currency::USD)
                            .label("Shipping cost"),
                        DateColumn::new(lens!(Order.created_at)).label("Order date"),
                    )),
                    RichTextColumn::new(lens!(Order.notes)),
                )),
            ))
            .form(Schema::new(Section::new("Order").schema(
                Grid::new(2).schema((
                    c.number.label("Number"),
                    c.customer_id.searchable().label("Customer"),
                    c.status.buttons(),
                    c.notes.rich_text(),
                )),
            )))
            .record_title(lens!(Order.number))
            .global_search(
                GlobalSearch::new()
                    .detail("Status", lens!(Order.status))
                    .detail("Total", lens!(Order.total_price)),
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
            .action::<CancelOrder>()
            .soft_delete(lens!(Order.deleted_at))
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
            .label("Customer")
            .toggleable(),
        TextColumn::new(lens!(Order.status)).badge(),
        TextColumn::new(lens!(Order.total_price)).money(Currency::USD),
        TextColumn::new(lens!(Order.shipping_price))
            .money(Currency::USD)
            .label("Shipping cost")
            .toggleable(),
        DateColumn::new(lens!(Order.created_at))
            .label("Order date")
            .sortable()
            .toggleable(),
    ))
    .default_sort_desc(lens!(Order.created_at))
    .action_layout(ActionLayout::Menu)
    .filters((
        DateFilter::new(lens!(Order.created_at))
            .label("Order date")
            .range(),
        SelectFilter::of(lens!(Order.status)),
    ))
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

    fn success(_cx: &Cx, _count: usize) -> String {
        "Order is now processing".to_string()
    }
}

/// What `ShipOrder` asks for.
#[derive(FormInput)]
pub struct ShippingNotes {
    #[form(label = "Shipping notes", multiline = 3)]
    pub notes: Option<String>,
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
                .notes(shipping.notes.clone())
                .exec(&mut *ex)
                .await?;
        }
        Ok(Outcome::done())
    }

    fn success(_cx: &Cx, _count: usize) -> String {
        "Order shipped".to_string()
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

    fn success(_cx: &Cx, _count: usize) -> String {
        "Order marked as delivered".to_string()
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

    fn success(_cx: &Cx, _count: usize) -> String {
        "Order cancelled".to_string()
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
            .label("Order item")
            .plural_label("Order items")
            .icon(crate::icons::ORDER_ITEMS)
            .navigation_group("Shop")
            .navigation_order(10)
            .policy(Allow)
            .without_global_search()
            .table(Table::new((
                RelationColumn::of::<ProductResource>(relation!(OrderItem.product))
                    .label("Product"),
                TextColumn::new(lens!(OrderItem.qty))
                    .numeric(0)
                    .label("Quantity"),
                TextColumn::new(lens!(OrderItem.unit_price))
                    .money(Currency::USD)
                    .label("Unit price"),
            )))
            .detail(Detail::new(
                Section::new("Order item").columns((
                    RelationColumn::of::<OrderResource>(relation!(OrderItem.order)).label("Order"),
                    RelationColumn::of::<ProductResource>(relation!(OrderItem.product))
                        .label("Product"),
                    TextColumn::new(lens!(OrderItem.qty))
                        .numeric(0)
                        .label("Quantity"),
                    TextColumn::new(lens!(OrderItem.unit_price))
                        .money(Currency::USD)
                        .label("Unit price"),
                )),
            ))
            .form(Schema::new(Section::new("Order item").schema((
                c.order_id.searchable().label("Order"),
                c.product_id.searchable().label("Product"),
                Grid::new(2).schema((c.qty.label("Quantity"), c.unit_price.label("Unit price"))),
            ))))
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
            .navigation_group("Shop")
            .navigation_order(11)
            .policy(Allow)
            .without_global_search()
            .table(Table::new((
                RelationColumn::of::<OrderResource>(relation!(Payment.order)).label("Order"),
                TextColumn::new(lens!(Payment.reference)).searchable(),
                TextColumn::new(lens!(Payment.amount))
                    .money(Currency::USD)
                    .sortable(),
                TextColumn::new(lens!(Payment.provider)).sortable(),
                TextColumn::new(lens!(Payment.method)).sortable(),
            )))
            .detail(Detail::new(Section::new("Payment").columns((
                RelationColumn::of::<OrderResource>(relation!(Payment.order)).label("Order"),
                TextColumn::new(lens!(Payment.reference)),
                TextColumn::new(lens!(Payment.amount)).money(Currency::USD),
                TextColumn::new(lens!(Payment.currency)),
                TextColumn::new(lens!(Payment.provider)),
                TextColumn::new(lens!(Payment.method)),
            ))))
            .form(Schema::new(Section::new("Payment").schema((
                c.order_id.searchable().label("Order"),
                c.reference,
                Grid::new(2).schema((c.amount, c.currency.searchable())),
                c.provider,
                c.method,
            ))))
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
    fn options() -> Vec<(String, String)> {
        vec![
            ("stripe".to_string(), "Stripe".to_string()),
            ("paypal".to_string(), "PayPal".to_string()),
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
            .plural_label("Addresses")
            .icon(crate::icons::ADDRESSES)
            .navigation_group("Shop")
            .navigation_order(12)
            .policy(Allow)
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
            .detail(Detail::new(Section::new("Address").columns((
                TextColumn::new(lens!(Address.street)),
                TextColumn::new(lens!(Address.city)),
                TextColumn::new(lens!(Address.state)),
                TextColumn::new(lens!(Address.zip)),
                TextColumn::new(lens!(Address.country)),
            ))))
            .form(Schema::new(Section::new("Address").schema((
                c.customer_id.searchable().label("Customer"),
                c.brand_id.searchable().label("Brand"),
                c.country.searchable(),
                c.street.label("Street address"),
                Grid::new(3).schema((
                    c.city,
                    c.state.label("State / Province"),
                    c.zip.label("Zip / Postal code"),
                )),
            ))))
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
/// The Filament demo edits these fields inside the order form; Tablo has no
/// nested relationship form, so they are a resource of their own.
pub struct OrderAddressResource;

impl Resource for OrderAddressResource {
    type Model = OrderAddress;
    type Form = OrderAddressForm;

    fn declare() -> ResourceDef<Self> {
        let c = OrderAddressForm::controls();
        ResourceDef::new()
            .label("Order address")
            .plural_label("Order addresses")
            .icon(crate::icons::ORDER_ADDRESSES)
            .navigation_group("Shop")
            .navigation_order(13)
            .policy(Allow)
            .without_global_search()
            .table(Table::new((
                RelationColumn::of::<OrderResource>(relation!(OrderAddress.order)).label("Order"),
                TextColumn::new(lens!(OrderAddress.street)),
                TextColumn::new(lens!(OrderAddress.city)),
                TextColumn::new(lens!(OrderAddress.zip)),
                TextColumn::new(lens!(OrderAddress.country)).width(ColumnWidth::Auto),
            )))
            .detail(Detail::new(Section::new("Order address").columns((
                RelationColumn::of::<OrderResource>(relation!(OrderAddress.order)).label("Order"),
                TextColumn::new(lens!(OrderAddress.street)),
                TextColumn::new(lens!(OrderAddress.city)),
                TextColumn::new(lens!(OrderAddress.state)),
                TextColumn::new(lens!(OrderAddress.zip)),
                TextColumn::new(lens!(OrderAddress.country)),
            ))))
            .form(Schema::new(Section::new("Order address").schema((
                c.order_id.searchable().label("Order"),
                c.country.searchable(),
                c.street.label("Street address"),
                Grid::new(3).schema((
                    c.city,
                    c.state.label("State / Province"),
                    c.zip.label("Zip / Postal code"),
                )),
            ))))
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
