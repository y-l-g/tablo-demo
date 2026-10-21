use std::sync::Arc;

use toasty::Db;
use topcoat::mail::MemoryTransport;

#[tokio::main]
async fn main() {
    let mut db = Db::builder()
        .models(tablo::models!(
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
            tablo_demo::models::Department,
            tablo_demo::models::Employee,
            tablo_demo::models::LeaveRequest,
            tablo_demo::models::Project,
            tablo_demo::models::Task,
            tablo_demo::models::Timesheet,
            tablo_demo::models::Expense,
            tablo_demo::models::ExpenseLine,
            tablo_demo::models::User,
            tablo_demo::models::Team,
            tablo_demo::models::TeamUser,
            tablo_demo::models::AuditEntry
        ))
        .connect("sqlite::memory:")
        .await
        .expect("connect");
    db.push_schema().await.expect("push schema");
    tablo_demo::seed::seed(&mut db)
        .await
        .expect("seed the demo data");
    // The demo has no mail server: the mails the app panel sends land here.
    let mailbox = Arc::new(MemoryTransport::new());
    let router = tablo_demo::panel::router(db, mailbox);
    topcoat::start(router).await.expect("serve");
}
