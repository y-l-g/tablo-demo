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
| Summaries other than a count (`Sum`, `Average`, `Range`) | Toasty has no other aggregate, so `Table::summary` only counts |
| Product `QueryBuilder` | no query builder; fixed filters instead |
| `poll` | Topcoat has no interval primitive; Tablo has no polling |
| Collapsible filters, `stackedOnMobile` | no equivalent |

## Forms

| Gap | Blocker |
| --- | --- |
| Reactive fields (slug from title, derived amounts) | no `Set`/`Get` |
| Nested relationship forms (order items, expense lines) | a repeater edits a `#[document]` list, not related rows |
| Wizards (create order, create expense) | no wizard |
| Project plan (`Builder`) | no block builder field |
| Masked phone and hours | no mask control; a custom `Control` is needed |

## Resources and actions

| Gap | Blocker |
| --- | --- |
| Nested URLs (`shop/orders`) | a slug is one URL segment in Tablo |
| Employee "view profile" slide-over | no slide-over; the detail page instead |

## Dashboards and shell

| Gap | Blocker |
| --- | --- |
| Widgets that follow the list's table state | a list's widgets rerun with its table but read none of its filters, so they show the same figures under any filter |
| `topbar(false)`, Vite theme | the shell is Tablo's |
| Importer column mapping | an import reads the create form's columns |
