# What did not translate

This demo is the [Filament demo](https://github.com/filamentphp/demo) rebuilt on
[Tablo](../filament-topcoat/tablo). The models, the relations, the resources, the
tables, the forms, the actions and the dashboards follow it, but Tablo is a
different framework with a smaller surface, and some of what Filament does has
no equivalent here.

This document lists every gap found while translating, what the demo does
instead, and why. Nothing here is a workaround: where Tablo has no equivalent,
the demo either omits the feature or uses the closest declaration it does have.

## Schema and models

| Filament | Here | Why |
| --- | --- | --- |
| Polymorphic `morphTo` / `morphToMany` | two nullable foreign keys | Toasty has no polymorphic relation; the [design](https://github.com/tokio-rs/toasty/blob/main/docs/dev/design/polymorphic-relations.md) is a roadmap item, and today's documented shape is one nullable key per owner |
| Spatie media library (`media` table, collections, conversions) | one `image` column holding the uploaded path | Tablo's file field binds one `String` column |
| Spatie tags (`tags` + `taggables`, JSON names) | `posts.tags`, a comma-separated string | there is no tag input and no polymorphic pivot |
| Soft deletes | a nullable `deleted_at` column, scoped out in the resource's `query`, set by `delete_record` | kept, but without the filter and the actions below |
| `full_address`, a generated column | dropped | Toasty declares no generated column; the demo used it only as a relation manager's record title |
| `hasManyThrough` (`Customer::payments`) | not rendered | Toasty has no through relation |
| Pivot columns (`id`, timestamps on `product_category_product`, `team_user`) | dropped; a join model is keyed by its two foreign keys | a link Tablo writes sets the two keys and nothing else |
| Auto-increment team ids | `Team.id` is a UUID | Tablo's tenant id is a `uuid::Uuid` |
| `decimal(10,2)` money columns | `f64` | Tablo binds no decimal type in a form or a table, and an app cannot add one: `TypedValue` is foreign to both `rust_decimal` and the app |
| `json` columns | `Vec<String>` for skills and labels, `Json<Value>` for metadata and the project plan | Toasty stores lists and JSON directly |
| `email_verified_at`, `remember_token` | carried, unused | Tablo ships no email verification or "remember me" |
| `failed_jobs`, `job_batches`, `personal_access_tokens`, `settings` | dropped | infrastructure tables: the demo queues nothing, exposes no API and stores no settings |
| Spatie's media relation on `Brand` and `ProductCategory` | dropped | neither registers a collection and no form, table or factory uploads for them |
| `expenses.receipt_path`, `employees.skills`, `tasks.labels` nullable | not null, defaulting to empty | a Tablo file field binds a `String`, and a list column is always present |

Details worth knowing:

- **Addresses.** `addressables` becomes `addresses.customer_id` and
  `addresses.brand_id`. The seeded rows each belong to one owner anyway, so the
  data matches, but the relation managers' **Attach** and **Detach** are gone:
  the address table on a customer is a plain owned list with Create, Edit and
  Delete.
- **Comments.** `commentable` becomes `comments.post_id` and
  `comments.product_id`, so a post and a product each keep their own relation
  table. A comment can name both, which the pivot's type column made impossible.
- **Order addresses.** `addressable` becomes `order_addresses.order_id`, and
  `Order::address` is a `has_one`.
- **The delete that does not delete.** The Filament demo overrides Delete to show
  a warning instead of deleting. Tablo's Delete is built in and cannot be
  replaced, so the demo deletes for real. Every soft-deleted table instead keeps
  the row and hides it.
- **Team membership does not gate anything**, exactly as in the Filament demo,
  where `User::getTenants` returns every team and `canAccessTenant` answers
  `true`. The `team_user` rows are seeded and the memberships are all teams.

## Resources

- **Six extra sidebar entries.** Tablo renders a relation as the related
  resource's own table, so the related resource must be registered, and every
  registered resource gets a sidebar entry. Filament needs no resource for a
  relation manager, so the demo there has no Comments, Addresses, Order
  addresses, Payments, Order items or Expense lines pages; here they exist, in
  the Shop and HR groups.
- **Every resource has a detail page.** A Tablo resource has a list, a detail
  page and a form unless it declares `Detail::empty()`, so `Brand`, `Customer`,
  `Order`, `Product`, `Employee`, `Task` and `Timesheet` gain a read-only page
  the Filament demo does not have. The demo keeps them: that page is where a
  record's related tables render, which Filament shows on the edit page instead.
- **Relations render on the detail page, not on the edit page.** Filament puts a
  relation manager on whatever page declares it, usually edit; a Tablo related
  table is on the detail page, because a change to one reruns the page and would
  reset the form's unsaved fields.
- **Nested slugs.** Filament mounts `blog/posts`, `shop/orders`, `hr/employees`.
  A Tablo slug is one URL segment, so every list lives directly under the panel
  prefix: `/admin/posts`, `/admin/orders`, `/admin/employees`.
- **No default sort.** A Tablo table orders by its first sortable column,
  ascending, until the reader sorts. `defaultSort('name')` is matched by
  declaration order; `defaultSort('created_at', 'desc')` is not expressible.
- **No inline editing.** `SelectColumn` (the leave request status in the list) and
  `TextColumn::toggleable()`/`toggledHiddenByDefault` have no equivalent: every
  declared column shows, and a value is edited on the form.
- **No summarizers.** `Sum`, `Average`, `Count` and `Range` on a column are not
  translated; there is no aggregate row.
- **No query builder.** The product list's `QueryBuilder` with arbitrary
  constraints becomes a fixed set of select, ternary and date filters. Search
  still covers the string columns.
- **A broken page cursor renders a retry card.** Filament paginates with a page
  number and ignores a bad one; Tablo's list carries a cursor, and a request with
  a cursor it cannot read answers 200 with a "couldn't load, retry" card rather
  than the first page.
- **One grouping per table.** Filament declares several groupings on the task
  and timesheet lists (project, status, priority, assignee; employee, project,
  date); a Tablo table groups by one lens, so the demo keeps the status grouping
  on tasks and the date grouping on timesheets.
- **No `->required()` over a nullable column.** A Tablo field is required when
  it has no blank answer, which an `Option` column always has, so the columns
  Filament marks required but declares nullable (a product's slug, SKU and
  barcode, an order's shipping price) stay optional here. `.unique()` is kept on
  the three email addresses.
- **No `TrashedFilter`.** A filter narrows a query, so it cannot re-include the
  rows `query()` hides. For the same reason there is no Restore or ForceDelete
  action: a hidden row has no page to run one from.
- **No has-many-through in a relation table.** A customer's relation table is
  its addresses; payments hang off the order they belong to, so a customer's
  payments are reachable only through that order's page.
- **No reactive fields.** Filament derives the slug from the title, the days from
  the dates, the cost from hours × rate and the line amount from quantity × price
  in the browser. Tablo has no `Set`/`Get` pair, so slugs are typed, and the
  derived amounts are recomputed server-side in `create_record`/`update_record`.
- **No `disabled()->dehydrated()`.** Tablo's `.disabled()` means the browser posts
  nothing and an edit keeps the stored value, so a column that Filament disables
  and re-posts (`slug`, `expense_number`, `days_requested`, `total_cost`) is an
  ordinary field here, or one the server recomputes.
- **No nested relationship forms.** `AddressForm` inside the order form is a
  has-one sub-form, and `Repeater::relationship('orderItems')` /
  `('expenseLines')` edit related rows. A Tablo repeater edits a `#[document]`
  list on the row itself, so order items, order addresses and expense lines are
  resources of their own, listed on the parent's detail page.
- **No `KeyValue` and no `Builder`.** The employee metadata and the project plan
  are carried in their columns and are not editable.
- **No checkbox list over a `Vec<String>`.** A multiple choice in a resource form
  must name a many-to-many field, so `skills` and `labels` are carried but not
  editable.
- **No colour picker, rich editor, tag input or masked input.** Colours are text
  fields, rich text is a textarea (and the demo's HTML shows as text), tags are
  comma-separated, and a phone number is not masked.
- **No tabs.** Filament's `Tabs` become sections one after another, and the list
  pages' status tabs (`getTabs()` with badges on orders, leave requests and
  expenses) become select filters.
- **`hiddenOn('create')`** has no equivalent: a condition watches a field, not the
  operation, so the leave request's Review section shows on create too.
- **No create-in-place.** `createOptionForm` on the order's customer select has no
  counterpart; create the customer in its own resource first.
- **No wizards.** The create-order and create-expense steps become one form.
- **No dependent predicate.** A Tablo dependent choice narrows by another field's
  value; "only a top-level category may be a parent" is a predicate, so every
  category is offered as a parent.
- **No `->hidden()` or `->format()` on a form field**, no `prefix`/`suffix`,
  `minValue`/`maxValue` or `rules()`; ranges belong in `validate_record`. A
  placeholder is available (`.placeholder(..)`) but is not set on every field
  Filament sets one on.
- **File fields are single-file.** `multiple()`, `maxFiles(5)` and `reorderable()`
  on the product images are not translated; a product has one image.

## Actions

- **Link actions are gone.** "View GitHub", "View Twitter" and "Visit website" are
  `->url()` actions that open a new tab; a Tablo action is a mutation, and a
  column renders text rather than a link. `public_link` is the one link Tablo
  adds, and it points at the record's own public page.
- **No replicate and no restore.** `ReplicateAction` and `RestoreAction` have no
  counterpart. A replicate would be an action with input that creates a row; the
  demo does not add one.
- **No read-only action page.** The employee's "view profile" slide-over shows an
  infolist; the demo's detail page is the read-only view.
- **A refused `run` shows a notification.** Filament's `before()` can throw a
  validation error into the modal; a Tablo action that returns an error rolls the
  transaction back and reports it as an error notification.
- **`disabled()` on an action hides it.** Tablo's `can_run` decides whether the
  button renders at all, so "toggle billable" disappears on an old timesheet
  rather than greying out.
- **No row classes.** `recordClasses` (the red rows for over-budget departments
  and overdue tasks) has no equivalent.
- **No navigation badges, no global search, no database notifications.** Tablo
  shows one toast per write; the sidebar carries no counts, there is no
  `command+k` search, and nothing is stored for later.

## Auth and panels

- **One session per browser.** Tablo stores the session under the panel that
  signed the user in, so signing in to `/app` ends the `/admin` session. The
  Filament demo behaves the same way in practice, and the demo's screenshots use
  one panel at a time.
- **No password reset and no email verification**, and no per-tenant panel access.
- **No tenant registration page.** Filament's `->tenantRegistration()` asks for a
  team after sign-up; Tablo's registrar creates the panel user in one step, so
  the demo's sign-up form creates the account and the tenant switcher lists the
  teams that exist.
- **The shell is Tablo's.** `->spa()`, `->topbar(false)`, `->colors()`, a Vite
  theme and brand logos are not translated. The demo sets the brand, the login
  hint, the Albert Sans font and the blue primary colour.
- **A page with a table does not hydrate.** The bulk-select box binds its checked
  and indeterminate states with expressions that call `.some()` and `.every()` on
  the runtime's `Vec` value, which is not a JavaScript array, so hydrating any
  list or detail page throws `t.some is not a function` and the runtime stops
  before it wires up navigation: every link then loads the whole document, and
  the table's own sort, paging and selection are dead with it. The dashboard and
  settings pages, which hold no table, navigate in place. `->spa()` has no
  counterpart until this is fixed, and the demo cannot fix it.
- **No icon registry.** Tablo ships eight navigation icons and resolves the rest
  from the set a build script stages, so an app with more entries names its own:
  the demo vendors the twenty-five Lucide icons it uses and picks one per entry.
  The Filament demo's heroicon aliases are not reproduced.
- **Options carry a label, not a colour.** `HasColor`/`HasIcon` on the demo's enums
  have no counterpart, so a status renders as its label in a text column rather
  than as a coloured badge.

## Imports and exports

- **No CSV import.** `ImportAction` and the two importers have no counterpart.
- **Export is built in.** Every list exports its current search, filters and sort
  as CSV through the list header, so the demo needs no exporter classes.

## Dashboards

Tablo has no widget system: no `StatsOverviewWidget`, no `ChartWidget`, no
`TableWidget`, no `getHeaderWidgets`, no `InteractsWithPageTable` and no
`InteractsWithPageFilters`. Each Filament dashboard is a `Page` here, and the
widgets become sections on it:

- a **stat** is a card, rendered from the same figures the widget computes;
- a **table widget** is a card listing its rows (flagged orders, the five oldest
  first, as a label and a value rather than the widget's six columns);
- a **chart** has no counterpart, and Tablo has no charting of its own, so the
  demo drops the chart widgets rather than draw them;
- a widget that follows the list's table state or the dashboard's filter form has
  no equivalent: a page cannot read either.

The figures themselves follow the widgets they come from, with two deliberate
differences: revenue and the product ranking leave out cancelled orders, which
the widgets count, and the product-margin scatter and the feature index on the
home page have no counterpart.

## Smaller differences

- `->copyable()`, `->limit()`, `->tooltip()`, `->weight()`, `->alignEnd()`,
  `->money()`, `->badge()`, `->prose()` and `->markdown()` are not translated.
  A text column formats its value with one closure; money is one of them.
- `->poll('30s')` on the expenses list has no counterpart: a Tablo table updates
  when it is asked to.
- `->deferFilters()`, `FiltersLayout::AboveContentCollapsible`,
  `->stackedOnMobile()` and `->reorderable('sort')` (drag to reorder brands) have
  no counterpart.
- Time is typed and shown as UTC: a `datetime-local` control carries no zone.
- **A date column has no date type.** `TextColumn` over a `jiff` date or
  timestamp renders the stored spelling, so a timestamp reads
  `2026-10-14T09:38:54.016162841Z` and a nullable one prints nothing. There is no
  column that spells the day, so the demo shows the stored value.
- **A false boolean is a bare cross.** `BooleanColumn` draws a circled check in
  the primary colour for true, and an uncircled cross in the muted colour for
  false. The demo shows both as Tablo draws them; the mark belongs to the column.
- **A labeled row action overflows its column.** The row-actions column takes a
  share of the table by link count, not by the width of the labels it holds, so
  "Toggle visibility", "Adjust price" and "Adjust stock" on the product list are
  painted over the security stock and publishing date columns; a wide column
  whose header is longer than its share ("Security stock") runs into its
  neighbour for the same reason. The demo can size its own columns but not the
  actions column, and neither width is a demo decision.
- **A unique value a hidden row holds answers 500.** The unique check probes
  through the resource's `query`, so a soft-deleted row is invisible to it, while
  the index the write then hits is not: reusing a deleted order's number, or a
  deleted customer's email, answers "internal server error" instead of coming
  back with "has already been taken" under the field. Four resources are affected
  (orders, customers, employees, projects), and a value a *visible* row holds
  behaves correctly. Tablo names this case and asks for a constraint scoped like
  the query, which a soft delete cannot do, so the demo leaves it documented
  rather than probing the table a second time by hand.
- **A count over a relation includes hidden rows.** The department list's
  headcount is `CountColumn`, which counts the related rows directly rather than
  through the related resource's query, so a soft-deleted employee is still
  counted there while the employee list hides it.
