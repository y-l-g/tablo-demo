//! The demo's mail transport. The app panel's mail goes to the SMTP server
//! `TABLO_DEMO_SMTP_HOST` names. Without one each send fails, and the failure
//! is logged: the demo keeps no copy of the mail it sends.

use topcoat::{
    context::Cx,
    mail::{Mail, SmtpTransport, Transport, TransportFuture},
};

/// The demo's transport, which sends each mail through the SMTP server the
/// environment names.
pub struct Postbox {
    smtp: Option<SmtpTransport>,
}

impl Postbox {
    /// Sends through the server the environment names, if any.
    pub fn from_env() -> Self {
        Self {
            smtp: smtp_from_env(),
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

/// The SMTP server `TABLO_DEMO_SMTP_HOST` names, or `None` when it is unset.
/// Port 465 connects with implicit TLS, any other port with STARTTLS (587 by
/// default); `TABLO_DEMO_SMTP_USERNAME` and `TABLO_DEMO_SMTP_PASSWORD` sign in.
fn smtp_from_env() -> Option<SmtpTransport> {
    let host = std::env::var("TABLO_DEMO_SMTP_HOST").ok()?;
    let port = std::env::var("TABLO_DEMO_SMTP_PORT")
        .ok()
        .map(|port| {
            port.parse::<u16>()
                .expect("TABLO_DEMO_SMTP_PORT is a port number")
        })
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
