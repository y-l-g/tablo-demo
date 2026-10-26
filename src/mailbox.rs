//! The mailbox: the mails the demo sent, so a demo can show its own links.
//!
//! Every mail the app panel sends lands in a transport that keeps it in
//! memory, and the admin panel's Mailbox page lists them. When
//! `TABLO_DEMO_SMTP_HOST` names a server, the mails go out over SMTP too.

use std::sync::Arc;

use tablo::{Page, PageDef};
use topcoat::mail::{Mail, SmtpTransport, TextBody, Transport, TransportFuture};
use topcoat::{
    Result,
    context::Cx,
    mail::MemoryTransport,
    view::{View, view},
};

/// The transport that keeps the mails the demo sent.
pub type Mailbox = Arc<MemoryTransport>;

/// The demo's transport: each mail goes to the SMTP server when one is
/// configured, and is kept in the mailbox once it was accepted.
pub struct Postbox {
    mailbox: Mailbox,
    smtp: Option<SmtpTransport>,
}

impl Postbox {
    /// Keeps the mails in `mailbox`, and sends them through the server the
    /// environment names, if any.
    pub fn new(mailbox: Mailbox) -> Self {
        Self {
            mailbox,
            smtp: smtp_from_env(),
        }
    }
}

impl Transport for Postbox {
    fn send<'a>(&'a self, cx: &'a Cx, mail: Mail) -> TransportFuture<'a> {
        Box::pin(async move {
            let Some(smtp) = &self.smtp else {
                return self.mailbox.send(cx, mail).await;
            };
            let receipt = smtp.send(cx, mail.clone()).await?;
            self.mailbox.send(cx, mail).await?;
            Ok(receipt)
        })
    }
}

/// The SMTP server `TABLO_DEMO_SMTP_HOST` names, or `None` to keep mails in
/// memory only. Port 465 connects with implicit TLS, any other port with
/// STARTTLS (587 by default); `TABLO_DEMO_SMTP_USERNAME` and
/// `TABLO_DEMO_SMTP_PASSWORD` sign in.
fn smtp_from_env() -> Option<SmtpTransport> {
    let host = std::env::var("TABLO_DEMO_SMTP_HOST").ok()?;
    let port = std::env::var("TABLO_DEMO_SMTP_PORT")
        .ok()
        .map(|port| port.parse::<u16>().expect("TABLO_DEMO_SMTP_PORT is a port number"))
        .unwrap_or(587);
    let builder = if port == 465 {
        SmtpTransport::relay(&host)
    } else {
        SmtpTransport::starttls(&host)
    }
    .expect("TABLO_DEMO_SMTP_HOST names an SMTP server")
    .port(port);
    let builder = match (
        std::env::var("TABLO_DEMO_SMTP_USERNAME"),
        std::env::var("TABLO_DEMO_SMTP_PASSWORD"),
    ) {
        (Ok(username), Ok(password)) => builder.credentials(username, password),
        _ => builder,
    };
    Some(builder.build())
}

/// The plain text of a mail, which is what the demo's mails carry.
fn body(cx: &Cx, mail: &Mail) -> String {
    match mail.text() {
        TextBody::Text(text) => text.clone(),
        TextBody::None => String::new(),
        TextBody::FromHtml => tablo::t!(cx, "copy-mail-html-body"),
    }
}

/// The admin panel's page: the mails the demo sent, in the order it sent them.
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
