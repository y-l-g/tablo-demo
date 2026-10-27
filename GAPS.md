# Gaps

What the [Filament demo](https://github.com/filamentphp/demo) does that this demo
cannot, and what blocks it. Everything not listed here is translated.

## Data

| Gap | Blocker |
| --- | --- |
| Polymorphic relations (addresses, comments): two nullable foreign keys instead, so no Attach/Detach on addresses | Toasty has no polymorphic relation |
| `Customer::payments` (has-many-through) is not shown | Toasty has no through relation |
| Money is `f64`, not `decimal(10,2)` | Tablo binds no decimal type, and an app cannot add one |
| Several images per product (`multiple`, `maxFiles`, `reorderable`) | a file field binds one `String` column |
| Post tags are a comma-separated string | no tag input, no polymorphic pivot |

## Tables

| Gap | Blocker |
| --- | --- |
| Summaries (`Sum`, `Average`, `Count`, `Range`) | no summarizer |
| Several groupings per table (tasks, timesheets) | `group_by` takes one lens |
| 5 default sorts (brands by `sort`; tasks, expenses, comments, payments by `created_at`) | a default sort must name a sortable column the table shows |
| Inline editing (`SelectColumn` on leave status) | no editable column |
| Product `QueryBuilder` | no query builder; fixed filters instead |
| Status tabs with badges (orders, employees, leave requests, expenses) | no list tabs; select filters instead |
| Row colours (`recordClasses`) | no row classes |
| Link columns and actions (GitHub, Twitter, website, "open product") | an action is a mutation; a column renders no link |
| Per-row currency on payments | `money` takes one currency per column |
| Post status badge | a `ComputedColumn` renders no badge |
| Option icons (`HasIcon`) | options carry a colour, not an icon |
| `copyable`, `limit`, `tooltip`, `weight`, empty-value `placeholder` | no equivalent |
| `poll`, `deferFilters`, collapsible filters, `stackedOnMobile` | no equivalent |
| Drag to reorder brands | no reorderable table |

## Forms

| Gap | Blocker |
| --- | --- |
| Reactive fields (slug from title, derived amounts) | no `Set`/`Get`; slugs are typed, amounts recomputed server-side |
| Nested relationship forms (order address, order items, expense lines) | a repeater edits a `#[document]` list, not related rows; they are separate resources on the parent's detail page |
| Wizards (create order, create expense) | no wizard |
| `createOptionForm` on the order's customer | no create-in-place |
| `hiddenOn('create')` (leave request review) | a condition watches a field, not the operation |
| Skills and labels checkbox lists | a multiple choice must bind a many-to-many field, not `Vec<String>` |
| Employee metadata (`KeyValue`), project plan (`Builder`) | no key-value or block builder field |
| Colour picker, masked phone and hours | no colour or mask control |
| `required()` on nullable columns (product slug, SKU, barcode; shipping price) | an `Option` field always has a blank answer |
| "Only a top-level category may be a parent" | a dependent choice narrows by a field value, not a predicate |

## Resources and actions

| Gap | Blocker |
| --- | --- |
| Relation tables are on the detail page, not the edit page; Comments, Addresses, Order items, Order addresses, Payments and Expense lines get sidebar entries | a relation renders a registered resource, every registered resource has a sidebar entry, and a related table on the edit page would reset the form |
| Nested URLs (`shop/orders`) | a slug is one URL segment |
| Delete shows a "don't be cheeky" warning instead of deleting | the built-in Delete cannot be replaced |
| "Toggle billable" greys out on an old timesheet | `can_run` hides the button instead |
| Employee "view profile" slide-over | no slide-over; the detail page instead |

## Dashboards and shell

| Gap | Blocker |
| --- | --- |
| Charts other than a sparkline | a stat tile draws a sparkline, and Tablo draws no other chart: no axes, bars or legends |
| Table widgets on a dashboard (latest orders, top products) | Tablo ships stat tiles as its only widget, so a table of records is a list page's, and the dashboard links to it |
| Widgets that follow the list's table state or the dashboard's filters | a widget reads no request state, and a page has no filters for them to follow |
| Brand wordmark logo with a dark-mode variant | `Brand::logo` is one square mark beside the name |
| `topbar(false)`, Vite theme | the shell is Tablo's |
| `command+k` for global search | no shortcut |
| Importer column mapping | an import reads the create form's columns |

## Access and tenancy

Three places where this demo leaves the Filament demo on purpose, because
copying it would hand a self-registered account the whole admin panel.

| What the Filament demo does | What this demo does instead |
| --- | --- |
| `User::canAccessPanel` answers `true`, so an account that signs up on `/app` opens `/admin` too | `User::is_admin` gates `/admin`; `/app` stays open to every account, and the seed hands the role out |
| `User::getTenants` answers with every `Team`, ignoring the `team_user` pivot it defines | memberships are read through the pivot, so a new account holds no team until it registers one |
| the panels' onboarding differs (`/app` verifies an address and requires a team; `/admin` requires neither) and a shared password reaches the laxer one | `/admin` admits administrators alone, so its lack of onboarding is no longer a way around `/app`'s |

What is still not multi-tenancy: no model carries a tenant column and no
resource declares a [`Tenancy`][tenancy], so every row the demo shows belongs to
every team, and switching from Acme Inc. to Globex changes the name above the
sidebar and nothing else. The Filament demo is the same, and mounts no resource
on its app panel at all. Demonstrating real scoping would mean giving `Project`
and `Task` a `tenant_id` and seeding each team its own rows.

[tenancy]: https://y-l.fr/tablo/nightly/api/tablo/struct.Tenancy.html
