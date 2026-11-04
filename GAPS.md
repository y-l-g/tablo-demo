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
| Dynamic action state: icon, label and colour per record (Employees) | an action's `label()` takes no record, and `ICON` is a constant |
| Action colour (Adjust price) | an action takes an icon but no colour; `DESTRUCTIVE` is the only tint |
| Extra dialog footer buttons (Ship and notify) | an action's dialog has one submit button |
| Header action count badge (Leave requests) | a header action has no badge; navigation items do |
| Tooltips on custom row actions (Brands) | custom actions have no tooltip; the built-in View, Edit and Delete have one |
| Manage records in a list modal (Authors) | create and edit are pages, not dialogs on the list |
