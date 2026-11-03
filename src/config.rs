//! What the environment sets, read once at start: every `TABLO_DEMO_*` variable the demo reads.

use std::path::PathBuf;

use topcoat::{cookie::Key, mail::Mailbox as Sender};

/// The demo's settings, from the environment.
pub struct Config {
    /// The router's cookie key, which signs the links to uploaded files and seals each account's
    /// authenticator secret.
    pub cookie_key: Key,
    /// The directory uploaded images are written to.
    pub upload_dir: PathBuf,
    /// The address the app panel's mail comes from.
    pub sender: Sender,
    /// The URL the app answers on, which every mailed link starts with.
    pub base_url: String,
    /// The SMTP server the app panel's mail goes through, `None` when no mail goes out.
    pub smtp: Option<Smtp>,
}

/// An SMTP server and its sign-in.
pub struct Smtp {
    pub host: String,
    /// 465 connects with implicit TLS, any other port with STARTTLS.
    pub port: u16,
    pub credentials: Option<(String, String)>,
}

impl Config {
    /// Reads the environment, panicking on a variable set to something the demo cannot use.
    pub fn from_env() -> Self {
        Self {
            cookie_key: cookie_key(),
            upload_dir: var("TABLO_DEMO_UPLOAD_DIR")
                .map_or_else(|| PathBuf::from("target/demo-uploads"), PathBuf::from),
            sender: sender(),
            base_url: var("TABLO_DEMO_BASE_URL")
                .unwrap_or_else(|| "http://127.0.0.1:3000".to_string()),
            smtp: smtp(),
        }
    }
}

/// The variable `name`, `None` when it is unset.
fn var(name: &str) -> Option<String> {
    std::env::var(name).ok()
}

/// `TABLO_DEMO_COOKIE_KEY` holds the key's 64 bytes as 128 hex characters
/// (`openssl rand -hex 64`); without it each start draws a key, so the upload links a page showed
/// before a restart stop working after it.
fn cookie_key() -> Key {
    let Some(secret) = var("TABLO_DEMO_COOKIE_KEY") else {
        return Key::generate();
    };
    let bytes: Option<Vec<u8>> = (0..secret.len())
        .step_by(2)
        .map(|at| {
            secret
                .get(at..at + 2)
                .and_then(|pair| u8::from_str_radix(pair, 16).ok())
        })
        .collect();
    bytes
        .and_then(|bytes| Key::try_from(bytes.as_slice()).ok())
        .expect("TABLO_DEMO_COOKIE_KEY holds 128 hex characters")
}

/// `TABLO_DEMO_MAIL_FROM`, which a mail provider has verified, or a placeholder.
fn sender() -> Sender {
    let address =
        var("TABLO_DEMO_MAIL_FROM").unwrap_or_else(|| "no-reply@demo.example.com".to_string());
    Sender::named("Tablo Demo", &address).expect("TABLO_DEMO_MAIL_FROM is an email address")
}

/// The server `TABLO_DEMO_SMTP_HOST` names, on `TABLO_DEMO_SMTP_PORT` (587 by default), signing in
/// with `TABLO_DEMO_SMTP_USERNAME` and `TABLO_DEMO_SMTP_PASSWORD` when both are set.
fn smtp() -> Option<Smtp> {
    let host = var("TABLO_DEMO_SMTP_HOST")?;
    let port = var("TABLO_DEMO_SMTP_PORT").map_or(587, |port| {
        port.parse().expect("TABLO_DEMO_SMTP_PORT is a port number")
    });
    let credentials = var("TABLO_DEMO_SMTP_USERNAME").zip(var("TABLO_DEMO_SMTP_PASSWORD"));
    Some(Smtp {
        host,
        port,
        credentials,
    })
}
