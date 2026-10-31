# Tablo demo

A demo of [Tablo](https://github.com/y-l-g/tablo): models, relations, resources,
tables, forms, actions and dashboards, served by Topcoat and persisted with
Toasty.

Two panels are mounted:

| Panel   | URL      | What it is                                                                           |
| ------- | -------- | ------------------------------------------------------------------------------------ |
| `admin` | `/admin` | every resource: the blog, the shop, HR and projects, plus the shop and HR dashboards. Staff only |
| `app`   | `/app`   | the tenant panel: every auth page, a team switcher and the settings page             |

The page at `/` introduces both, with a button per demo account that signs in to
it, a tour of what Tablo covers, and switches for the language and the theme that
the panels share. It is
public, rendered in the admin panel's document and catalog through its
`PanelHandle`, in [`src/welcome.rs`](src/welcome.rs).

The seed creates one account per staff role, each signing in with `password`;
the welcome page signs in as any of them in one click, through the panel's own
login form. What each role may do is [`src/roles.rs`](src/roles.rs): a policy per
domain, field access on what employees are paid, and the dashboards and the
audit trail gated to the roles that read them.

| Account              | Role          | In `/admin`                                                  |
| -------------------- | ------------- | ------------------------------------------------------------ |
| `admin@example.com`  | Administrator | everything, the audit trail included                         |
| `shop@example.com`   | Shop manager  | the shop and the blog; the projects read-only; no HR         |
| `hr@example.com`     | HR manager    | HR and the projects, pay included; the blog read-only; no shop |
| `viewer@example.com` | Viewer        | every domain read-only, without pay                          |

An account that signs up on `/app` holds no role: it verifies its address,
registers a team of its own, and `/admin` refuses it.

## Running it

Tablo comes from its [Git repository](https://github.com/y-l-g/tablo) at the
last commit of its `master` branch. Topcoat comes from its
[Git repository](https://github.com/tokio-rs/topcoat) at `d84362b7`, the commit
Tablo's `master` builds against, through the `[patch.crates-io]` section of
`Cargo.toml`. Toasty comes from crates.io, so no other checkout is needed.
`Cargo.lock` records the commit a build uses; `cargo update -p tablo` moves
Tablo to the newest one. The demo needs `can_access_panel` to receive the
asking panel's prefix, so the commit cannot be older than Tablo's `424077a9`.

The asset bundle has to come from that same Topcoat commit, so install its CLI
from there:

```sh
cargo install --git https://github.com/tokio-rs/topcoat --rev d84362b72bc1034a8342e00c1cdf1a36d68dd9cf topcoat-cli --locked
```

```sh
cargo run
# open http://127.0.0.1:3000/
```

The database is in-memory SQLite, created and seeded on every start, so a restart
is always a clean demo. The seed covers a year of a shop that grows, with its
orders paid, shipped and now and then cancelled; its product shots and blog
covers are generated art in [`assets/seed/`](assets/seed/), copied to the upload
directory at start. `topcoat dev` builds the asset bundle and serves the app
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
email-change confirmations and two-factor codes through an SMTP server. Its
settings are below. Without `TABLO_DEMO_SMTP_HOST` no mail goes out, and each
send that fails is logged. The demo keeps no copy of what it sends.

| Variable                   | What it is                                                    |
| -------------------------- | ------------------------------------------------------------- |
| `TABLO_DEMO_SMTP_HOST`     | the server, e.g. `smtp-relay.brevo.com`; unset, no mail is sent |
| `TABLO_DEMO_SMTP_PORT`     | `465` (implicit TLS), or any other port with STARTTLS; `587` by default          |
| `TABLO_DEMO_SMTP_USERNAME` | the SMTP login                                                 |
| `TABLO_DEMO_SMTP_PASSWORD` | the SMTP key or password                                       |
| `TABLO_DEMO_MAIL_FROM`     | the sender address, one your provider has verified             |
| `TABLO_DEMO_BASE_URL`      | the URL the mailed links start with; `http://127.0.0.1:3000` by default |

Failed sends are logged to stdout. `RUST_LOG` sets the level, `info` by default.

## What it covers

- **Resources** for twenty-one tables across four domains, each with a list, a
  record form, a detail page and its relations. A record's child tables, its
  comments, addresses, order items, payments and expense lines, are tabs on its
  page, so their resources have no sidebar entry.
- **Tables** with search, sort, filters, grouping, pagination and CSV export,
  status tabs with their counts, and switches and selects that save in their
  cells: a product's or a brand's visibility, a leave request's status. Cancelled
  orders are tinted.
- **Forms** with choice enums, relationship choices, dependent choices,
  conditional fields, tabs, embedded values, `has_many` and `belongs_to` writes, a
  shipping address as a has-one form inside the order, a customer created in place
  from the order, many-to-many tags a post shares with other posts, key-value rows
  (an employee's metadata), colour inputs, skills and a task's labels as lists with
  suggestions, and file uploads, a product's gallery of up to four images among them,
  checked by extension and size before they are stored and
  served only at signed links that expire.
- **Actions**, record and bulk, with input dialogs, confirmation and success
  notifications.
- **Dashboards** built from Tablo's widgets, each page a grid of them: figures
  as stat tiles with trends and sparklines, linked to the lists they count and
  to their filtered lists; line and bar charts over the last twelve weeks, of
  revenue, orders placed and hours booked, billable and not, and of each
  department's headcount against its seats; and tables of the newest posts
  and orders, and of the leave and expenses waiting for a decision, each row
  linking to its record.
- **List widgets** above three lists, the orders, leave requests and posts: the
  order pipeline, the leave requests by status, and the articles by publishing
  state. The order and leave figures link to their list tabs.
- **Theming** through Tablo's tokens: [`styles.css`](styles.css) gives both
  panels an indigo brand, in light and dark.
- **Navigation** that swaps the page in place, and an icon of its own for every
  sidebar entry, from a Lucide subset the build stages.
- **Auth** on the `app` panel: sign-up, password reset, email verification, a
  profile page that changes the name, password and email (the new address
  confirmed by a mailed link), two-factor sign-in with an authenticator app or
  mailed codes and recovery codes, and creating and renaming a team.
- **Global search** in the topbar, over every resource, with results that respect
  the list's tenancy, trash and field access and open the record's page.
- **Translations** in English and French, the browser's `Accept-Language`
  choosing until a visitor picks the other from the language menu in the
  sidebar footer. The demo's own words are in [`locales/`](locales/), beside Tablo's
  catalog.

[GAPS.md](GAPS.md) lists the features this demo still lacks, and what blocks each.
