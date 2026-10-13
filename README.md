# Tablo demo

The [Filament demo](https://github.com/filamentphp/demo) rebuilt on
[Tablo](../filament-topcoat/tablo): the same models, relations, resources,
tables, forms, actions and dashboards, served by Topcoat and persisted with
Toasty.

Two panels are mounted, as in the Filament demo:

| Panel | URL | What it is |
| --- | --- | --- |
| `admin` | `/admin` | every resource: the blog, the shop, HR and projects, plus the shop and HR dashboards |
| `app` | `/app` | the tenant panel: sign-up, a team switcher and the settings page |

Sign in with `admin@filamentphp.com` / `demo.Filament@2021!`.

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

## What it covers

- **Resources** for twenty-one tables across four domains, each with a list, a
  record form, a detail page and its relations.
- **Tables** with search, sort, filters, grouping, pagination and CSV export.
- **Forms** with choice enums, relationship choices, dependent choices,
  conditional fields, embedded values, `has_many` and `belongs_to` writes, and
  file uploads.
- **Actions**, record and bulk, with input dialogs, confirmation and success
  notifications.
- **Dashboards** that read the seeded data and render their own figures and
  charts.

[GAPS.md](GAPS.md) lists everything in the Filament demo that Tablo has no
equivalent for, and what this demo does instead.
