//! The demo's mail transport. The app panel's mail goes to the SMTP server the
//! [`Config`] names. Without one each send fails, and the failure is logged: the
//! demo keeps no copy of the mail it sends.

use crate::config::Smtp;
use topcoat::{
    context::Cx,
    mail::{Mail, SmtpTransport, Transport, TransportFuture},
};

/// The demo's transport, which sends each mail through the configured SMTP server.
pub struct Postbox {
    smtp: Option<SmtpTransport>,
}

impl Postbox {
    /// Sends through `smtp`, or fails each send without one.
    pub fn new(smtp: Option<&Smtp>) -> Self {
        Self {
            smtp: smtp.map(transport),
        }
    }
}

impl Transport for Postbox {
    fn send<'a>(&'a self, cx: &'a Cx, mail: Mail) -> TransportFuture<'a> {
        Box::pin(async move {
            let Some(smtp) = &self.smtp else {
                return Err(topcoat::Error::msg(
                    "no SMTP server: TABLO_DEMO_SMTP_HOST is unset",
                ));
            };
            // Tablo logs a failed send by its message alone, so the cause goes in it.
            smtp.send(cx, mail)
                .await
                .map_err(|error| topcoat::Error::msg(format!("{error:#}")))
        })
    }
}

/// A transport to `smtp`: implicit TLS on port 465, STARTTLS on any other.
fn transport(smtp: &Smtp) -> SmtpTransport {
    let builder = if smtp.port == 465 {
        SmtpTransport::relay(&smtp.host)
    } else {
        SmtpTransport::starttls(&smtp.host)
    }
    .expect("TABLO_DEMO_SMTP_HOST names an SMTP server")
    .port(smtp.port);
    match &smtp.credentials {
        Some((username, password)) => builder.credentials(username.clone(), password.clone()),
        None => builder,
    }
    .build()
}
