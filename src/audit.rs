//! The audit trail: a row per write, written in the write's own transaction.
//!
//! Tablo's commit hook runs inside the transaction the write opened,
//! so the row it writes commits with the write or not at all: there is no
//! window where the record changed and the trail did not.

use tablo::{
    Change, ChangedRecord, CommitHook, DateColumn, NoForm, PanelChange, ReadOnly, Resource,
    ResourceDef, ResourcePage, Table, TextColumn, lens, when,
};
use topcoat::{Result, context::Cx};

use crate::auth::SignedIn;
use crate::models::AuditEntry;

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
            .icon(crate::icons::AUDIT_TRAIL)
            .navigation_order(-1)
            .policy(ReadOnly.and(when(crate::roles::is_administrator)))
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
