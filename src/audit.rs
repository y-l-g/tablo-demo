//! The audit trail: a row per write, written in the write's own transaction.
//!
//! Tablo's `before_commit` hook runs inside the transaction the write opened,
//! so the row it writes commits with the write or not at all: there is no
//! window where the record changed and the trail did not.

use tablo::{
    Change, DateColumn, NoForm, ReadOnly, Resource, ResourceDef, ResourcePage, Table, TextColumn,
    lens, when,
};
use topcoat::{Result, context::Cx};

use crate::auth::SignedIn;
use crate::models::AuditEntry;

/// Writes the audit row for `change`, in the write's transaction.
pub async fn write<R: Resource>(
    cx: &Cx,
    change: &Change<R::Model>,
    ex: &mut dyn toasty::Executor,
) -> Result<()> {
    let (kind, record) = match change {
        Change::Create(record) => (tablo::t!(cx, "audit-create"), url::<R>(cx, record)),
        Change::Update { after, .. } => (tablo::t!(cx, "audit-update"), url::<R>(cx, after)),
        Change::Delete(records) => (tablo::t!(cx, "audit-delete"), first_url::<R>(cx, records)),
        Change::Restore(records) => (tablo::t!(cx, "audit-restore"), first_url::<R>(cx, records)),
        Change::ForceDelete(records) => (
            tablo::t!(cx, "audit-force-delete"),
            first_url::<R>(cx, records),
        ),
        Change::Action { name, records } => (
            tablo::t!(cx, "audit-action", name = *name),
            first_url::<R>(cx, records),
        ),
        Change::HeaderAction { name } => (
            tablo::t!(cx, "audit-header-action", name = *name),
            String::new(),
        ),
        Change::Attach { owner, .. } => (tablo::t!(cx, "audit-attach"), url::<R>(cx, owner)),
        Change::Detach { owner, .. } => (tablo::t!(cx, "audit-detach"), url::<R>(cx, owner)),
    };
    let user = match tablo::auth::user::<SignedIn>(cx) {
        Some(user) => user.user.email.clone(),
        None => "anonymous".to_string(),
    };
    toasty::create!(AuditEntry {
        resource: resource_name::<R>(),
        change: kind,
        record,
        user,
        at: jiff::Timestamp::now(),
    })
    .exec(&mut *ex)
    .await?;
    Ok(())
}

/// Records every write to the resource in the write's own transaction.
///
/// The audit matches every kind of write, so a kind Tablo adds later stops the
/// build instead of going unrecorded.
#[macro_export]
macro_rules! audit {
    () => {
        async fn before_commit(
            cx: &topcoat::context::Cx,
            change: &tablo::Change<Self::Model>,
            ex: &mut dyn toasty::Executor,
        ) -> topcoat::Result<()> {
            $crate::audit::write::<Self>(cx, change, ex).await
        }
    };
}

/// The audit trail: every write the panel made, newest first, for administrators to read.
///
/// The trail records every domain's writes, so only administrators open it, and no one writes
/// to it through the panel: its rows come from [`write`] alone.
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

/// The record's path in the panel, empty when the resource serves no such page.
fn url<R: Resource>(cx: &Cx, record: &R::Model) -> String {
    tablo::url::record::<R>(cx, record).unwrap_or_default()
}

/// The path of the first of `records`, empty when there is none.
fn first_url<R: Resource>(cx: &Cx, records: &[R::Model]) -> String {
    records
        .first()
        .map(|record| url::<R>(cx, record))
        .unwrap_or_default()
}

/// The resource's name, e.g. `Order` for `tablo_demo::shop::OrderResource`.
fn resource_name<R: Resource>() -> String {
    std::any::type_name::<R>()
        .rsplit("::")
        .next()
        .unwrap_or("Resource")
        .trim_end_matches("Resource")
        .to_string()
}
