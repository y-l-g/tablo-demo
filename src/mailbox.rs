//! The mailbox: the mails the demo sent, so a demo can show its own links.
//!
//! The demo has no mail server, so the mails the app panel sends land in a
//! transport that keeps them in memory, and the app panel's Mailbox page
//! lists them. A real app would send them over SMTP.

use std::sync::Arc;

use tablo::{Page, PageDef};
use topcoat::mail::{Mail, TextBody};
use topcoat::{
    Result,
    context::Cx,
    mail::MemoryTransport,
    view::{View, view},
};

/// The transport that keeps the mails the demo sent.
pub type Mailbox = Arc<MemoryTransport>;

/// The plain text of a mail, which is what the demo's mails carry.
fn body(cx: &Cx, mail: &Mail) -> String {
    match mail.text() {
        TextBody::Text(text) => text.clone(),
        TextBody::None => String::new(),
        TextBody::FromHtml => tablo::t!(cx, "copy-mail-html-body"),
    }
}

/// The app panel's page: the mails the demo sent, newest first.
pub struct Sent;

impl Page for Sent {
    fn declare() -> PageDef {
        PageDef::new()
            .label(tablo::text!("page-mailbox"))
            .icon(crate::icons::MAILBOX)
            .navigation_order(1)
    }

    async fn render(cx: &Cx) -> Result<impl View> {
        let mails = topcoat::context::try_app_context::<Mailbox>(cx)
            .map(|mailbox| mailbox.sent())
            .unwrap_or_default();

        Ok(view! {
            cx =>
            tablo::ui::page(
                tablo::ui::page_header(
                    tablo::ui::page_title((tablo::t!(cx, "page-mailbox")))
                    tablo::ui::page_description(
                        (tablo::t!(cx, "copy-mailbox-description"))
                    )
                )
                tablo::ui::page_content(
                    tablo::ui::table(
                        tablo::ui::table_header(
                            tablo::ui::table_row(
                                tablo::ui::table_head((tablo::t!(cx, "label-to")))
                                tablo::ui::table_head((tablo::t!(cx, "label-subject")))
                                tablo::ui::table_head((tablo::t!(cx, "label-message")))
                            )
                        )
                        tablo::ui::table_body(
                            for mail in mails {
                                tablo::ui::table_row(
                                    tablo::ui::table_cell(
                                        (mail.to().first().map(|to| to.address().to_string()).unwrap_or_default())
                                    )
                                    tablo::ui::table_cell((mail.subject()))
                                    tablo::ui::table_cell((body(cx, &mail)))
                                )
                            }
                        )
                    )
                )
            )
        })
    }
}
