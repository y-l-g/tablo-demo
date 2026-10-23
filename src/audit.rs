//! The audit trail: a row per write, written in the write's own transaction.
//!
//! Tablo's `before_commit` hook runs inside the transaction the write opened,
//! so the row it writes commits with the write or not at all: there is no
//! window where the record changed and the trail did not.

use tablo::{Change, Page, PageDef, Resource, db::db};
use topcoat::{
    Result,
    context::Cx,
    view::{View, view},
};

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

/// The panel's page: every write the panel made, newest first.
pub struct AuditTrail;

impl Page for AuditTrail {
    fn declare() -> PageDef {
        PageDef::new()
            .icon(crate::icons::AUDIT_TRAIL)
            .navigation_order(-1)
    }

    async fn render(cx: &Cx) -> Result<impl View> {
        let mut db = db(cx);
        let mut entries = AuditEntry::all().exec(&mut db).await?;
        entries.reverse();

        Ok(view! {
            cx =>
            tablo::ui::page(
                tablo::ui::page_header(
                    tablo::ui::page_title((tablo::t!(cx, "page-audit-trail")))
                    tablo::ui::page_description(
                        (tablo::t!(cx, "copy-audit-description"))
                    )
                )
                tablo::ui::page_content(
                    tablo::ui::table(
                        tablo::ui::table_header(
                            tablo::ui::table_row(
                                tablo::ui::table_head((tablo::t!(cx, "label-when")))
                                tablo::ui::table_head((tablo::t!(cx, "label-resource")))
                                tablo::ui::table_head((tablo::t!(cx, "label-change")))
                                tablo::ui::table_head((tablo::t!(cx, "label-record")))
                                tablo::ui::table_head((tablo::t!(cx, "label-by")))
                            )
                        )
                        tablo::ui::table_body(
                            for entry in entries {
                                tablo::ui::table_row(
                                    tablo::ui::table_cell((moment(entry.at)))
                                    tablo::ui::table_cell((entry.resource))
                                    tablo::ui::table_cell((entry.change))
                                    tablo::ui::table_cell(
                                        if entry.record.is_empty() {
                                            "-"
                                        } else {
                                            <a
                                                href=(entry.record.clone())
                                                class="text-primary underline underline-offset-4"
                                            >
                                                (entry.record)
                                            </a>
                                        }
                                    )
                                    tablo::ui::table_cell((entry.user))
                                )
                            }
                        )
                    )
                )
            )
        })
    }
}

/// The moment the write committed, as a readable time.
fn moment(at: jiff::Timestamp) -> String {
    at.to_zoned(jiff::tz::TimeZone::UTC)
        .strftime("%Y-%m-%d %H:%M:%S")
        .to_string()
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
