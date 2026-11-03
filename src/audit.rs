//! The audit trail: a row per write, written in the write's own transaction.
//!
//! Tablo's commit hook runs inside the transaction the write opened,
//! so the row it writes commits with the write or not at all: there is no
//! window where the record changed and the trail did not.

use jiff::Timestamp;
use tablo::{
    Change, ChangedRecord, CommitHook, DateColumn, NoForm, PanelChange, ReadOnly, Resource,
    ResourceDef, ResourcePage, Table, TextColumn, lens, when,
};
use topcoat::{Result, context::Cx, icon::iconify::iconify_icon};

use crate::access;
use crate::accounts::SignedIn;

/// One write the panel made, recorded in the write's own transaction.
#[derive(Debug, Clone, toasty::Model, tablo::ModelColumns)]
pub struct AuditEntry {
    #[key]
    #[auto]
    pub id: u64,
    /// The resource that was written, by its label, e.g. `Leave request`.
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

/// Writes an audit row for every write the admin panel makes, in the write's transaction.
///
/// The hook matches every kind of write, so a kind Tablo adds later stops the build instead of
/// going unrecorded.
pub struct Audit;

impl CommitHook for Audit {
    async fn before_commit(
        cx: &Cx,
        change: &PanelChange,
        ex: &mut dyn toasty::Executor,
    ) -> Result<()> {
        let kind = match change.change() {
            Change::Create(_) => tablo::t!(cx, "audit-create"),
            Change::Update { .. } => tablo::t!(cx, "audit-update"),
            Change::Delete(_) => tablo::t!(cx, "audit-delete"),
            Change::Restore(_) => tablo::t!(cx, "audit-restore"),
            Change::ForceDelete(_) => tablo::t!(cx, "audit-force-delete"),
            Change::Action { name, .. } => tablo::t!(cx, "audit-action", name = *name),
            Change::HeaderAction { name } => tablo::t!(cx, "audit-header-action", name = *name),
            Change::Attach { .. } => tablo::t!(cx, "audit-attach"),
            Change::Detach { .. } => tablo::t!(cx, "audit-detach"),
        };
        let user = match tablo::auth::user::<SignedIn>(cx) {
            Some(user) => user.user.email.clone(),
            None => "anonymous".to_string(),
        };
        toasty::create!(AuditEntry {
            resource: change.label().localize(cx),
            change: kind,
            // The first record the write names; a bulk write links the first of its rows.
            record: change
                .change()
                .records()
                .first()
                .and_then(ChangedRecord::url)
                .unwrap_or_default()
                .to_string(),
            user,
            at: jiff::Timestamp::now(),
        })
        .exec(&mut *ex)
        .await?;
        Ok(())
    }
}

/// The audit trail: every write the panel made, newest first, for administrators to read.
///
/// The trail records every domain's writes, so only administrators open it, and no one writes
/// to it through the panel: its rows come from [`Audit`] alone.
pub struct AuditTrail;

impl Resource for AuditTrail {
    type Model = AuditEntry;
    type Form = NoForm<AuditEntry>;

    fn declare() -> ResourceDef<Self> {
        ResourceDef::new()
            .slug("audit-trail")
            .icon(iconify_icon!("lucide:rotate-ccw-clock"))
            .navigation_order(-1)
            .policy(ReadOnly.and(when(access::is_administrator)))
            .without_global_search()
            .without_page(ResourcePage::View)
            .table(
                Table::new((
                    DateColumn::new(lens!(AuditEntry.at)).sortable(),
                    TextColumn::new(lens!(AuditEntry.resource)).searchable(),
                    TextColumn::new(lens!(AuditEntry.change)).searchable(),
                    TextColumn::new(lens!(AuditEntry.record))
                        .url(|entry: &AuditEntry| {
                            (!entry.record.is_empty()).then(|| entry.record.clone())
                        })
                        .placeholder("-"),
                    TextColumn::new(lens!(AuditEntry.user)).searchable(),
                ))
                .default_sort_desc(lens!(AuditEntry.at)),
            )
    }
}
