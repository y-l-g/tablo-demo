//! Where uploaded files are served, and what an upload may be.

/// The URL prefix uploaded files are served under.
pub const URL_PREFIX: &str = "/uploads";

/// The image types a product or a post takes, and the most bytes an image may hold.
pub const IMAGE_EXTENSIONS: [&str; 4] = ["jpg", "jpeg", "png", "webp"];
pub const IMAGE_MAX_BYTES: usize = 2 * 1024 * 1024;

/// The file types an expense's receipt takes, and the most bytes a receipt may hold.
pub const RECEIPT_EXTENSIONS: [&str; 5] = ["jpg", "jpeg", "png", "webp", "pdf"];
pub const RECEIPT_MAX_BYTES: usize = 5 * 1024 * 1024;
