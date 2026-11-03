//! The seed's randomness, drawn from a fixed seed so the demo data is the same on every run,
//! and what it draws: names, phone numbers, colours, amounts and dates.

use jiff::{Span, Timestamp, Zoned, civil::Date};
use rust_decimal::Decimal;

/// A small deterministic generator, so the demo data is the same on every run.
pub(super) struct Rng(u64);

impl Rng {
    pub(super) fn new(seed: u64) -> Self {
        Self(seed)
    }

    pub(super) fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    /// A number below `n`.
    pub(super) fn below(&mut self, n: u64) -> u64 {
        if n == 0 { 0 } else { self.next() % n }
    }

    /// A number in `lo..=hi`.
    pub(super) fn between(&mut self, lo: u64, hi: u64) -> u64 {
        lo + self.below(hi.saturating_sub(lo) + 1)
    }

    /// A number in `lo..hi`.
    pub(super) fn float(&mut self, lo: f64, hi: f64) -> f64 {
        let unit = (self.next() >> 11) as f64 / (1u64 << 53) as f64;
        lo + unit * (hi - lo)
    }

    /// Whether a `percent` share of answers are true.
    pub(super) fn boolean(&mut self, percent: u64) -> bool {
        self.below(100) < percent
    }

    /// One of `items`.
    pub(super) fn pick<'a, T>(&mut self, items: &'a [T]) -> &'a T {
        &items[self.below(items.len() as u64) as usize]
    }

    /// `count` of `items`, each drawn at most once.
    pub(super) fn pick_distinct<T: Clone>(&mut self, items: &[T], count: u64) -> Vec<T> {
        let count = count.min(items.len() as u64);
        let mut out: Vec<T> = Vec::with_capacity(count as usize);
        let mut taken: Vec<usize> = Vec::new();
        for _ in 0..count {
            let mut index = self.below(items.len() as u64) as usize;
            while taken.contains(&index) {
                index = (index + 1) % items.len();
            }
            taken.push(index);
            out.push(items[index].clone());
        }
        out
    }
}

/// How many days ago the `index`th of `count` records was made, the oldest two years ago, so
/// that more of them fall in each month than in the one before.
pub(super) fn growth_age(index: u64, count: u64) -> f64 {
    let left = 1.0 - index as f64 / count as f64;
    HISTORY_DAYS * left.powf(1.3)
}

/// A timestamp `hours` before `now`.
pub(super) fn hours_ago(now: &Zoned, hours: f64) -> Timestamp {
    now.timestamp()
        .checked_sub(Span::new().seconds((hours * 3600.0) as i64))
        .unwrap_or_else(|_| now.timestamp())
}

/// A timestamp `days` before `now`.
pub(super) fn days_ago(now: &Zoned, days: i64) -> Timestamp {
    now.checked_sub(Span::new().days(days))
        .unwrap_or_else(|_| now.clone())
        .timestamp()
}

/// The day `at` falls on, where the demo runs.
pub(super) fn date_of(at: Timestamp) -> Date {
    at.to_zoned(jiff::tz::TimeZone::system()).date()
}

/// The date `days` before today, or after it when `days` is negative.
pub(super) fn today_back(days: i64) -> Date {
    let today = Zoned::now().date();
    today.checked_sub(Span::new().days(days)).unwrap_or(today)
}

/// A URL-safe spelling of `text`.
pub(super) fn slug(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut last_dash = true;
    for c in text.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
            last_dash = false;
        } else if !last_dash {
            out.push('-');
            last_dash = true;
        }
    }
    out.trim_end_matches('-').to_string()
}

/// A person's name.
pub(super) fn person(rng: &mut Rng) -> String {
    format!("{} {}", rng.pick(FIRST_NAMES), rng.pick(LAST_NAMES))
}

/// A phone number.
pub(super) fn phone(rng: &mut Rng) -> String {
    format!(
        "+1 ({:03}) {:03}-{:04}",
        rng.below(1000),
        rng.below(1000),
        rng.below(10_000)
    )
}

/// A hex colour.
pub(super) fn hex_color(rng: &mut Rng) -> String {
    format!("#{:06x}", rng.below(0x100_0000))
}

/// A measure rounded to the two decimals a value of its kind holds.
pub fn round(amount: f64) -> f64 {
    (amount * 100.0).round() / 100.0
}

/// A money amount from a float, rounded to the two decimals a price holds.
pub fn money(amount: f64) -> Decimal {
    Decimal::from_f64_retain(amount)
        .unwrap_or_default()
        .round_dp(2)
}

/// How far back the shop's history reaches.
const HISTORY_DAYS: f64 = 720.0;

const FIRST_NAMES: &[&str] = &[
    "Ada", "Alan", "Grace", "Linus", "Margaret", "Ken", "Barbara", "Dennis", "Radia", "Vint",
    "Anita", "Tim", "Sophie", "Marcus", "Elena", "Priya", "Tomas", "Nadia", "Omar", "Lena", "Hugo",
    "Ines", "Jonas", "Maya", "Nils", "Olga", "Pablo", "Quinn", "Rosa", "Sven",
];

const LAST_NAMES: &[&str] = &[
    "Lovelace",
    "Turing",
    "Hopper",
    "Torvalds",
    "Hamilton",
    "Thompson",
    "Liskov",
    "Ritchie",
    "Perlman",
    "Cerf",
    "Borg",
    "Berners-Lee",
    "Wilson",
    "Okafor",
    "Novak",
    "Silva",
    "Haddad",
    "Bergman",
    "Costa",
    "Dubois",
];
