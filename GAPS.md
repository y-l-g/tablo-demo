# Gaps

What the [Filament demo](https://github.com/filamentphp/demo) does that this demo
cannot, and what blocks it.

## Data

| Gap | Blocker |
| --- | --- |
| Polymorphic relations (addresses, comments), so no Attach/Detach on addresses | Toasty has no polymorphic relation; two nullable keys instead |
| `Customer::payments` (through orders) is not shown | Tablo's relations hold rows by a foreign key or a join model, with no through relation |
| Money columns are not sortable (prices, totals, salaries) | SQLite stores a `Decimal` as text, so `ORDER BY` sorts by spelling |

## Tables

| Gap | Blocker |
| --- | --- |
| Summaries (`Sum`, `Average`, `Count`, `Range`) | no summarizer in Tablo |
| Product `QueryBuilder` | no query builder; fixed filters instead |
| Row colours on leave requests and expenses (`recordClasses`) | a row colour refuses a page whose columns load employees, whose salary is restricted |
| Link action ("open product") | an action is a mutation, not a link |
| `weight` (font weight on a column) | no equivalent |
| `poll` | Topcoat has no interval primitive; Tablo has no polling |
| `deferFilters`, collapsible filters, `stackedOnMobile` | no equivalent |

## Forms

| Gap | Blocker |
| --- | --- |
| Reactive fields (slug from title, derived amounts) | no `Set`/`Get` |
| Nested relationship forms (order items, expense lines) | a repeater edits a `#[document]` list, not related rows |
| Wizards (create order, create expense) | no wizard |
| Project plan (`Builder`) | no block builder field |
| Masked phone and hours | no mask control; a custom `Control` is needed |
| `required()` on nullable columns (product slug, SKU, barcode; shipping price) | an `Option` field always has a blank answer |
| "Only a top-level category may be a parent" | a dependent choice narrows by a field value, not a predicate |

## Resources and actions

| Gap | Blocker |
| --- | --- |
| Nested URLs (`shop/orders`) | a slug is one URL segment in Tablo |
| Employee "view profile" slide-over | no slide-over; the detail page instead |

## Dashboards and shell

| Gap | Blocker |
| --- | --- |
| Widgets that follow the list's table state or the dashboard's filters | a widget reads no request state: a list's widgets rerun with its table but show the same figures under any filter, and a page has no filters to follow |
| Brand wordmark logo with a dark-mode variant | `Brand::logo` takes one image |
| `topbar(false)`, Vite theme | the shell is Tablo's |
| `command+k` for global search | no keyboard shortcut in Tablo; no command palette or shortcut primitive in Topcoat |
| Importer column mapping | an import reads the create form's columns |
