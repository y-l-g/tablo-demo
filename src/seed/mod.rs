//! The demo data: a year and more of a shop that grows, a blog with a post a week, and a company
//! with its staff, their leave, their projects and their expenses.
//!
//! The seed imports the domains and none imports it. Each domain's data is drawn in a file of its
//! own, all from one generator with a fixed seed, so the demo data is the same on every run.

mod accounts;
mod blog;
mod hr;
mod projects;
mod random;
mod shop;

use std::path::Path;

use jiff::Zoned;
use toasty::Db;
use uuid::Uuid;

use crate::shared::uploads;
use random::Rng;

/// Fills an empty database with the demo's rows, and `upload_dir` with the images they show.
pub async fn seed(db: &mut Db, upload_dir: &Path) -> toasty::Result<()> {
    let mut rng = Rng::new(0x5eed);
    let now = Zoned::now();
    let today = now.date();

    let staff = accounts::seed_staff(db).await?;
    let teams = accounts::seed_teams(db).await?;
    let shots = upload_all(upload_dir, 0, PRODUCT_SHOTS).await?;
    let covers = upload_all(upload_dir, PRODUCT_SHOTS.len(), POST_COVERS).await?;
    let (brands, customers) = shop::seed_shop(db, &mut rng, &now).await?;
    let products = shop::seed_products(db, &mut rng, &brands, &customers, &shots).await?;
    shop::seed_orders(db, &mut rng, &now, &customers, &products).await?;
    blog::seed_blog(db, &mut rng, &now, &customers, &covers).await?;
    let (departments, employees) = hr::seed_people(db, &mut rng, &now).await?;
    let projects =
        projects::seed_projects(db, &mut rng, &now, today, &departments, &employees).await?;
    hr::seed_expenses(db, &mut rng, &now, &employees, &projects).await?;
    accounts::seed_memberships(db, &teams, &staff).await
}

/// Writes each of `images` to `dir`, the upload directory, and returns the paths the panel serves them
/// at. A file's name is fixed by its place, from `first` on, so each start writes over the files
/// the last one wrote.
async fn upload_all(
    dir: &Path,
    first: usize,
    images: &[(&str, &[u8])],
) -> toasty::Result<Vec<String>> {
    tokio::fs::create_dir_all(dir).await?;
    let mut paths = Vec::new();
    for (offset, (name, bytes)) in images.iter().enumerate() {
        let file = format!("{}-{name}", Uuid::from_u128((first + offset) as u128 + 1));
        tokio::fs::write(dir.join(&file), bytes).await?;
        paths.push(format!("{}/{file}", uploads::URL_PREFIX));
    }
    Ok(paths)
}

/// Embeds each named image of `assets/seed/{dir}`.
macro_rules! images {
    ($dir:literal: $($name:literal),* $(,)?) => {
        &[$(($name, include_bytes!(concat!("../../assets/seed/", $dir, "/", $name)))),*]
    };
}

/// The product shots the seeded products show.
const PRODUCT_SHOTS: &[(&str, &[u8])] = images!("products":
    "product-01.webp", "product-02.webp", "product-03.webp", "product-04.webp",
    "product-05.webp", "product-06.webp", "product-07.webp", "product-08.webp",
    "product-09.webp", "product-10.webp", "product-11.webp", "product-12.webp",
    "product-13.webp", "product-14.webp", "product-15.webp", "product-16.webp",
);

/// The covers the seeded posts show.
const POST_COVERS: &[(&str, &[u8])] = images!("posts":
    "cover-01.webp", "cover-02.webp", "cover-03.webp", "cover-04.webp",
    "cover-05.webp", "cover-06.webp", "cover-07.webp", "cover-08.webp",
);
