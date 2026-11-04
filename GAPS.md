# Gaps

What the [Filament demo](https://github.com/filamentphp/demo) does that this demo
cannot, and what blocks it. Each row is blocked by Tablo; a feature Tablo offers
is built here instead.

## Data

| Gap | Blocker |
| --- | --- |
| Polymorphic relations (addresses, comments), so no Attach/Detach on addresses | Toasty has no polymorphic relation; two nullable keys instead |
| `Customer::payments` (through orders) is not shown | Tablo's relations hold rows by a foreign key or a join model, with no through relation |

## Tables

| Gap | Blocker |
| --- | --- |
| Summaries other than a count (`Sum`, `Average`, `Range`) | Toasty has no other aggregate, so `Table::summary` only counts |
| Product `QueryBuilder` | no query builder; fixed filters instead |
| Column layouts, split and stack (Authors) | a column holds one value, with no multi-row layout |

## Forms

| Gap | Blocker |
| --- | --- |
| Reactive fields (slug from title, derived amounts) | no `Set`/`Get` |
| Nested relationship forms (order items, expense lines) | a repeater edits a `#[document]` list, not related rows |
| Project plan (`Builder`) | no block builder field |
| Masked phone and hours | no mask control; a mask needs a script, and a custom `Control` renders markup only |

## Resources and actions

| Gap | Blocker |
| --- | --- |
| Nested URLs (`shop/orders`) | a slug is one URL segment in Tablo |
| Employee "view profile" slide-over | no slide-over; the detail page instead |
| Colour per record (Employees) | an action's `COLOR` is a constant, so a button's tint cannot follow its record |
| Extra dialog footer buttons (Ship and notify) | an action's dialog has one submit button |
| Tooltips on menu items (Brands) | a menu item takes no tooltip; only an inline row button takes one |
| Manage records in a list modal (Authors) | create and edit are pages, not dialogs on the list |
