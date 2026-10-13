use toasty::Db;

#[tokio::main]
async fn main() {
    let db = Db::builder()
        .models(toasty::models!(
            tablo_demo::models::Address,
            tablo_demo::models::Comment,
            tablo_demo::models::Author,
            tablo_demo::models::Post,
            tablo_demo::models::PostCategory,
            tablo_demo::models::Brand,
            tablo_demo::models::Product,
            tablo_demo::models::ProductCategory,
            tablo_demo::models::ProductCategoryLink,
            tablo_demo::models::Customer,
            tablo_demo::models::Order,
            tablo_demo::models::OrderItem,
            tablo_demo::models::OrderAddress,
            tablo_demo::models::Payment,
            tablo_demo::models::User,
            tablo_demo::models::Team,
            tablo_demo::models::TeamUser,
            tablo::auth::AuthSession
        ))
        .connect("sqlite::memory:")
        .await
        .expect("connect");
    db.push_schema().await.expect("push schema");
    let router = tablo_demo::panel::router(db);
    topcoat::start(router).await.expect("serve");
}
