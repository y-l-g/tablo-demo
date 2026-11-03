use tablo_demo::{config::Config, router, seed};
use toasty::Db;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    // Tablo logs a mail that failed to send, and the log is where it shows.
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();
    let config = Config::from_env();
    // An in-memory database, created and seeded on every start, so a restart is a clean demo.
    let mut db = Db::builder()
        .models(tablo_demo::models())
        .connect("sqlite::memory:")
        .await
        .expect("connect");
    db.push_schema().await.expect("push schema");
    seed::seed(&mut db, &config.upload_dir)
        .await
        .expect("seed the demo data");
    topcoat::start(router::router(db, &config))
        .await
        .expect("serve");
}
