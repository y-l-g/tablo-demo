# Gaps

What the [Filament demo](https://github.com/filamentphp/demo) does that this demo
cannot, and what blocks it.

## Data

| Gap | Blocker |
| --- | --- |
| Polymorphic relations (addresses, comments), so no Attach/Detach on addresses | Toasty has no polymorphic relation; two nullable keys instead |
| `Customer::payments` (has-many-through) is not shown | Toasty has no through relation |
| Money is `f64`, not `decimal(10,2)` | SQLite stores a decimal as text, so a price or total column cannot sort numerically |
| Post tags are a comma-separated string | no tag input, no polymorphic pivot |

## Tables

| Gap | Blocker |
| --- | --- |
| Summaries (`Sum`, `Average`, `Count`, `Range`) | no summarizer |
| Several groupings per table (tasks, timesheets) | `group_by` takes one lens |
| Product `QueryBuilder` | no query builder; fixed filters instead |
| Row colours on leave requests and expenses (`recordClasses`) | a row colour refuses a page whose columns load employees, whose salary is restricted |
| Link action ("open product") | an action is a mutation, not a link |
| Post status badge, localised | a computed column's value is a plain `String` with no `cx`, so it cannot be localised |
| `weight` (font weight on a column) | no equivalent |
| `poll`, `deferFilters`, collapsible filters, `stackedOnMobile` | no equivalent |
| Drag to reorder brands | no reorderable table |

## Forms

| Gap | Blocker |
| --- | --- |
| Reactive fields (slug from title, derived amounts) | no `Set`/`Get`; slugs are typed, amounts recomputed server-side |
| Nested relationship forms (order address, order items, expense lines) | a repeater edits a `#[document]` list, not related rows |
| Wizards (create order, create expense) | no wizard |
| `createOptionForm` on the order's customer | no create-in-place |
| Skills and labels checkbox lists | a multiple choice must bind a many-to-many field, not `Vec<String>` |
| Employee metadata (`KeyValue`), project plan (`Builder`) | no key-value or block builder field |
| Colour picker, masked phone and hours | no colour or mask control |
| `required()` on nullable columns (product slug, SKU, barcode; shipping price) | an `Option` field always has a blank answer |
| "Only a top-level category may be a parent" | a dependent choice narrows by a field value, not a predicate |

## Resources and actions

| Gap | Blocker |
| --- | --- |
| Nested URLs (`shop/orders`) | a slug is one URL segment |
| Delete shows a "don't be cheeky" warning instead of deleting | the built-in Delete cannot be replaced |
| "Toggle billable" greys out on an old timesheet | `can_run` hides the button instead |
| Employee "view profile" slide-over | no slide-over; the detail page instead |

## Dashboards and shell

| Gap | Blocker |
| --- | --- |
| Top products on a dashboard | a record table orders by a column of the record, and a product keeps no count of what it sold |
| Widgets that follow the list's table state or the dashboard's filters | a widget reads no request state: a list's widgets rerun with its table but show the same figures under any filter, and a page has no filters to follow |
| Brand wordmark logo with a dark-mode variant | `Brand::logo` takes one image |
| `topbar(false)`, Vite theme | the shell is Tablo's |
| `command+k` for global search | no shortcut |
| Importer column mapping | an import reads the create form's columns |
