| Gap | Blocker |
| --- | --- |
| Polymorphic relations (addresses, comments), so no Attach/Detach on addresses | Toasty has no polymorphic relation; two nullable keys instead |
| `Customer::payments` (through orders) is not shown | Tablo's relations hold rows by a foreign key or a join model, with no through relation |
| Summaries other than a count (`Sum`, `Average`, `Range`) | Toasty has no other aggregate, so `Table::summary` only counts |
| Product `QueryBuilder` | no query builder; fixed filters instead |
| Column layouts, split and stack (Authors) | a column holds one value, with no multi-row layout |
| Reactive fields (slug from title, derived amounts) | no `Set`/`Get` |
| Nested relationship forms (order items, expense lines) | a repeater edits a `#[document]` list, not related rows |
| Project plan (`Builder`) | no block builder field |
| Masked phone and hours | no mask control; a mask needs a script, and a custom `Control` renders markup only |
| Nested URLs (`shop/orders`) | a slug is one URL segment in Tablo |
| Employee "view profile" slide-over | no slide-over; the detail page instead |
| Extra dialog footer buttons (Ship and notify) | an action's dialog has one submit button |
| Manage records in a list modal (Authors) | create and edit are pages, not dialogs on the list |
