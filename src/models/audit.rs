//! The audit trail: one row per write the panel made.

use jiff::Timestamp;

/// One write the panel made, recorded in the write's own transaction.
#[derive(Debug, Clone, toasty::Model, tablo::ModelColumns)]
pub struct AuditEntry {
    #[key]
    #[auto]
    pub id: u64,
    /// The resource that was written, e.g. `Order`.
    pub resource: String,
    /// What the write did, e.g. `create` or `action: publish`.
    pub change: String,
    /// The record the write touched, as its path in the panel.
    pub record: String,
    /// Who made the write.
    pub user: String,
    /// When the write committed.
    pub at: Timestamp,
}
