# Tablo demo

The [Filament demo](https://github.com/filamentphp/demo) rebuilt on
[Tablo](https://github.com/y-l-g/tablo): the same models, relations, resources,
tables, forms, actions and dashboards, served by Topcoat and persisted with
Toasty.

Two panels are mounted, as in the Filament demo:

| Panel   | URL      | What it is                                                                           |
| ------- | -------- | ------------------------------------------------------------------------------------ |
| `admin` | `/admin` | every resource: the blog, the shop, HR and projects, plus the shop and HR dashboards |
| `app`   | `/app`   | the tenant panel: every auth page, a team switcher and the settings page             |

Sign in with `admin@example.com` / `password`.

## Running it

The demo builds Tablo from the checkout beside it, so keep the two repositories
side by side:

```
Github/
  filament-topcoat/tablo
  tablo-demo
```

Tablo's own dependencies, Topcoat and Toasty, come from crates.io at the versions
Tablo pins, so no other checkout is needed.

```sh
cargo run
# open http://127.0.0.1:3000/admin
```

The database is in-memory SQLite, created and seeded on every start, so a restart
is always a clean demo. `topcoat dev` builds the asset bundle and serves the app
the way a release build does; a plain `cargo run` needs `topcoat asset bundle`
first, because the panel serves its stylesheet, font and scripts from a bundle
beside the executable.

The router's cookie key signs the links to uploaded files and seals each
account's authenticator secret. Set
`TABLO_DEMO_COOKIE_KEY` to 128 hex characters (`openssl rand -hex 64`) so those
links and authenticator apps stay valid across restarts and replicas; without
it, each start draws a key.

### Mail

The app panel mails sign-up verification links, password reset links,
email-change confirmations and two-factor codes. Every mail lands in the admin
panel's **Mailbox** page. To deliver them too, point the demo at an SMTP
server:

| Variable                   | What it is                                                    |
| -------------------------- | ------------------------------------------------------------- |
| `TABLO_DEMO_SMTP_HOST`     | the server, e.g. `smtp-relay.brevo.com`; unset keeps mail in memory only |
| `TABLO_DEMO_SMTP_PORT`     | `587` (STARTTLS, the default) or `465` (implicit TLS)          |
| `TABLO_DEMO_SMTP_USERNAME` | the SMTP login                                                 |
| `TABLO_DEMO_SMTP_PASSWORD` | the SMTP key or password                                       |
| `TABLO_DEMO_MAIL_FROM`     | the sender address, one your provider has verified             |
| `TABLO_DEMO_BASE_URL`      | the URL the mailed links start with; `http://127.0.0.1:3000` by default |

## What it covers

- **Resources** for twenty-one tables across four domains, each with a list, a
  record form, a detail page and its relations.
- **Tables** with search, sort, filters, grouping, pagination and CSV export.
- **Forms** with choice enums, relationship choices, dependent choices,
  conditional fields, tabs, embedded values, `has_many` and `belongs_to` writes,
  and file uploads, checked by extension and size before they are stored and
  served only at signed links that expire.
- **Actions**, record and bulk, with input dialogs, confirmation and success
  notifications.
- **Dashboards** that read the seeded data: figures as Tablo's stat tiles,
  linked to their lists, and ranked lists the demo draws itself.
- **Navigation** that swaps the page in place, and an icon of its own for every
  sidebar entry, from a Lucide subset the build stages.
- **Auth** on the `app` panel: sign-up, password reset, email verification, a
  profile page that changes the name, password and email (the new address
  confirmed by a mailed link), two-factor sign-in with an authenticator app or
  mailed codes and recovery codes, and creating and renaming a team.
- **Global search** in the topbar, over every resource, with results that respect
  the list's tenancy, trash and field access and open the record's page.
- **Translations** in English and French, the browser's `Accept-Language`
  choosing. The demo's own words are in [`locales/`](locales/), beside Tablo's
  catalog.

[GAPS.md](GAPS.md) lists what the Filament demo does that this demo cannot,
and what blocks it.
