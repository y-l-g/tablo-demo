# Roadmap

| Gap | Blocker |
| --- | --- |
| Polymorphic relations (addresses, comments), so no Attach/Detach on addresses | Toasty has no polymorphic relation; two nullable keys instead |
| `Customer::payments` (through orders) is not shown | Tablo's relations hold rows by a foreign key or a join model, with no through relation |
| Summaries other than a count (`Sum`, `Average`, `Range`) | Toasty has no other aggregate, so `Table::summary` only counts |
| Product `QueryBuilder` | no query builder; fixed filters instead |
| Reactive fields (slug from title, derived amounts) | no field reacts to another before the form is posted |
| Nested relationship forms (order items, expense lines) | a repeater edits a `#[document]` list, not related rows |
| Project plan (`Builder`) | no block builder field |
| Masked phone and hours | no mask control; a mask needs a script, and a custom `Control` renders markup only |
| Extra dialog footer buttons (Ship and notify) | an action's dialog has one submit button |
| Manage records in a list modal (Authors) | create and edit are pages, not dialogs on the list |
| Price filters (`?f.price.min=..`) | `NumberFilter` takes integers and floats, not `Decimal`; the products filter by stock instead |
| A project's progress in the reader's locale (`45 %`) | a computed column's closure gets the record alone, not the request, so `Number::percent_in` cannot reach it; the dashboards spell their percentages in the reader's locale |
