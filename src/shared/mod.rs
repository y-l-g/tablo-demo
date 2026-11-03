//! What the domains share: a form control, dashboard figures, the bell, upload
//! limits, and two small helpers.

mod controls;
mod notify;
pub mod stats;
pub mod uploads;

pub use controls::ColorInput;
pub use notify::notify;

use jiff::Timestamp;

/// Each of `words` as an option labelled by itself.
pub fn suggestions(words: &[&str]) -> Vec<(String, tablo::Text)> {
    words
        .iter()
        .map(|word| (word.to_string(), (*word).into()))
        .collect()
}

/// Five characters that tell a replica's slug or number apart from its original's.
pub fn replica_suffix() -> String {
    const DIGITS: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let mut n = Timestamp::now().as_nanosecond().unsigned_abs();
    (0..5)
        .map(|_| {
            let digit = DIGITS[(n % 36) as usize] as char;
            n /= 36;
            digit
        })
        .collect()
}
