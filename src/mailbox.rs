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
fn body(mail: &Mail) -> String {
    match mail.text() {
        TextBody::Text(text) => text.clone(),
        TextBody::None => String::new(),
        TextBody::FromHtml => "The mail carries an HTML body.".to_string(),
    }
}

/// The app panel's page: the mails the demo sent, newest first.
pub struct Sent;

impl Page for Sent {
    fn declare() -> PageDef {
        PageDef::new()
            .label("Mailbox")
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
                    tablo::ui::page_title("Mailbox")
                    tablo::ui::page_description(
                        "The mails the demo sent. The demo has no mail server, so they land here."
                    )
                )
                tablo::ui::page_content(
                    tablo::ui::table(
                        tablo::ui::table_header(
                            tablo::ui::table_row(
                                tablo::ui::table_head("To")
                                tablo::ui::table_head("Subject")
                                tablo::ui::table_head("Message")
                            )
                        )
                        tablo::ui::table_body(
                            for mail in mails {
                                tablo::ui::table_row(
                                    tablo::ui::table_cell(
                                        (mail.to().first().map(|to| to.address().to_string()).unwrap_or_default())
                                    )
                                    tablo::ui::table_cell((mail.subject()))
                                    tablo::ui::table_cell((body(&mail)))
                                )
                            }
                        )
                    )
                )
            )
        })
    }
}
