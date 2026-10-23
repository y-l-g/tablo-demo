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
| Tabs in the employee and project forms | no tabs layout; sections instead |
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
| Charts | Tablo has no charting |
| Widgets that follow the list's table state or the dashboard's filters | no widget system; a page reads neither |
| Brand wordmark logo with a dark-mode variant | `Brand::logo` is one square mark beside the name |
| `topbar(false)`, Vite theme | the shell is Tablo's |
| `command+k` for global search | no shortcut |
| Importer column mapping | an import reads the create form's columns |
