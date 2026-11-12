# Tablo demo

A demo of [Tablo](https://github.com/y-l-g/tablo): models, relations, resources,
tables, forms, actions and dashboards, served by Topcoat and persisted with
Toasty.

Three panels are mounted:

| Panel   | URL      | What it is                                                                           |
| ------- | -------- | ------------------------------------------------------------------------------------ |
| `admin` | `/admin` | every resource: the blog, the shop, HR and projects, plus the shop and HR dashboards. Staff only |
| `app`   | `/app`   | the tenant panel: every auth page, a team switcher and a settings area               |
| `partners` | `/partners` | the blog's authors and categories, to read, behind the demo's own sign-in page, not a Tablo form |

The page at `/` introduces them, with a button per demo account that signs in to
the first two, a tour of what Tablo covers, and switches for the language and the theme that
the panels share. It is
public, rendered in the admin panel's document and catalog through its
`PanelHandle`, in [`src/welcome.rs`](src/welcome.rs).

The seed creates one account per staff role, each signing in with `password`;
the welcome page signs in as any of them in one click, through the panel's own
login form. What each role may do is [`src/access.rs`](src/access.rs): a policy per
domain, field access on what employees are paid, and the dashboards and the
audit log gated to the roles that read them.

| Account              | Role          | In `/admin`                                                  |
| -------------------- | ------------- | ------------------------------------------------------------ |
| `admin@example.com`  | Administrator | everything, the audit log and acting as a colleague included |
| `shop@example.com`   | Shop manager  | the shop and the blog; the projects read-only; no HR         |
| `hr@example.com`     | HR manager    | HR and the projects, pay included; the blog read-only; no shop |
| `viewer@example.com` | Viewer        | every domain read-only, without pay                          |

An account that signs up on `/app` holds no role: it verifies its address,
registers a team of its own, and `/admin` refuses it.

## Running it

Tablo comes from its [Git repository](https://github.com/y-l-g/tablo) at the
last commit of its `master` branch. Topcoat and Toasty come from crates.io, so
no other checkout is needed.
`Cargo.lock` records the commit a build uses; `cargo update -p tablo` moves
Tablo to the newest one. The demo builds on Topcoat 0.12, so the commit cannot
be older than Tablo's `0c43305e`.

The asset bundle has to come from the same Topcoat release, so install its CLI
at that version:

```sh
cargo install topcoat-cli --version 0.12.0 --locked
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
  cells: the visibility of a product, a brand or a category, a leave request's
  status. Cancelled
  orders, pending and rejected leave, and submitted and rejected expenses are
  tinted. The order and product lists count their records under the rows, the
  product filters wait for an Apply button, each brand links to its website, and
  order numbers, SKUs and emails copy with a click, and a long task title cuts to
  forty characters, its whole text shown on hover. Products filter by a range of
  stock, and customers by what their email contains, ends with or is, and by
  whether they gave a phone number.
- **Forms** with choice enums, relationship choices, a parent category chosen
  among the top-level ones only, required nullable fields, dependent choices,
  conditional fields, tabs, embedded values, `has_many` and `belongs_to` writes, a
  shipping address as a has-one form inside the order, a customer created in place
  from the order, many-to-many tags a post shares with other posts, key-value rows
  (an employee's metadata), colour inputs, skills and a task's labels as lists with
  suggestions, and file uploads, a product's gallery of up to four images among them,
  checked by extension and size before they are stored and
  served only at signed links that expire.
- **Actions**, record and bulk, with input dialogs, confirmation, success
  notifications, and an icon and a warning tint on the price adjustment. An employee's
  activation button reads Activate or Deactivate, with its icon, for that employee.
  The leave list's header button approves every pending request, confirming first,
  and shows how many wait. An address's Show on map lands on OpenStreetMap, another
  site the `admin` panel's content security policy lets a form reach.
- **Dashboards** built from Tablo's widgets, each page a grid of them, the shop's
  under a period filter its sales follow: figures
  as stat tiles with trends and sparklines, linked to the lists they count and
  to their filtered lists; line and bar charts over the last twelve weeks, of
  revenue, orders placed and hours booked, billable and not, and of each
  department's headcount against its seats; and tables of the newest posts
  and orders, and of the leave and expenses waiting for a decision, each row
  linking to its record.
- **List widgets** above three lists, the orders, leave requests and posts: the
  order pipeline, the leave requests by status, and the articles by publishing
  state, each counting the rows its list shows under the current tab, search and
  filters. The order and leave figures link to their list tabs.
- **Themes**, one of Tablo's on each panel through `Brand::theme`: Jade on
  `admin`, Iris on `app` and Amber, close to Filament's look, on `partners`,
  each in light and dark, with a logo that turns light on a dark page.
  Every page, sign-in included, wears a "Demo" ribbon, and its tab title starts
  with `[Demo]`.
- **Numbers in the reader's locale**: amounts and counts in the lists and on the
  dashboards read `$1,234.50` in English and `1 234,50 $` in French, and the
  dashboards' percentages `12.5%` and `12,5 %`.
- **An audit log**, Tablo's own, which administrators read at `/admin/audit-log`:
  every write of the `admin` panel, in the write's transaction, with the fields
  it changed before and after.
- **Acting as a colleague**: the administrator's Staff page, with Tablo's
  impersonation button per colleague, opens the `admin` panel as the shop
  manager, the HR manager or the viewer sees it, for half an hour or until Stop,
  and the audit log names the administrator on every change made meanwhile.
- **Navigation** that swaps the page in place, and an icon of its own for every
  sidebar entry, from a Lucide subset the build stages.
- **Areas**: the `app` panel's settings live under `/app/settings`, one sidebar
  entry that opens a sidebar of their own, with the profile and two-factor pages
  in an Account group and the team's page in a group headed by the team's name.
- **Auth** on the `app` panel: sign-up, password reset, email verification, a
  profile page that changes the name, password and email (the new address
  confirmed by a mailed link), two-factor sign-in with an authenticator app or
  mailed codes and recovery codes, and creating and renaming a team. Tablo's own
  messages call the tenant a team, through the catalogs' `-tenant` term. The
  `partners` panel has no login form: `Auth::external` trusts the demo's own
  session, from [`src/partners.rs`](src/partners.rs), and Sign out ends both.
- **Global search** in the topbar, over every resource, with results that respect
  the list's tenancy, trash and field access and open the record's page.
- **Translations** in English and French, the browser's `Accept-Language`
  choosing until a visitor picks the other from the language menu in the
  sidebar footer. A signed-in account keeps its pick, which follows it to
  another browser and into its mail. The demo's own words are in
  [`locales/`](locales/), beside Tablo's catalog.

## Layout

The code follows the domains the sidebar shows, so a feature is found where its
menu entry is.

| Module                                           | Holds                                                                  |
| ------------------------------------------------ | ---------------------------------------------------------------------- |
| [`src/shop/`](src/shop), [`src/blog/`](src/blog), [`src/hr/`](src/hr), [`src/projects/`](src/projects) | a domain: its models, its resources, its dashboard and its list widgets |
| [`src/accounts/`](src/accounts)                  | the users and teams, sign-in, sign-up, the profile and the team pages, and the staff an administrator acts as |
| [`src/access.rs`](src/access.rs)                 | what each staff role may do, domain by domain                          |
| [`src/home.rs`](src/home.rs)                     | the dashboard the first two panels open on                                      |
| [`src/panels/`](src/panels)                      | the three panels, built from the domains, and the brand they share       |
| [`src/router.rs`](src/router.rs)                 | the routes: the welcome page, the panels and the assets                |
| [`src/shared/`](src/shared)                      | what several domains use: stats, uploads, notifications, custom inputs |
| [`src/seed/`](src/seed)                          | the demo data, a file per domain, all from one fixed-seed generator    |

A few conventions keep it that way:

- **A file per resource**, named after it; a resource that outgrows one file,
  past some 400 lines, becomes a folder, as the orders and expenses have.
- **A domain's models in its `models.rs`**, with its enums, and the business
  rules on the models themselves: a project's progress, a timesheet's cost and
  lock, the days a leave request takes, an expense line's amount. Resources
  call them, so a form, a column and the seed agree.
- **A child resource beside its parent**: an order's items, payments and
  shipping address live in `shop/order/`.
- **A list widget beside its list's resource**; a dashboard's widgets
  in the domain's `dashboard`.
- **The seed imports the domains, and none imports it.**
- **Icons inline**, as `iconify_icon!("lucide:…")` in each `declare()`.

[ROADMAP.md](ROADMAP.md) lists the features this demo still lacks, and what blocks each.
