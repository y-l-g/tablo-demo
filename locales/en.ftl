# The demo's own English messages. Tablo's catalog holds the rest; a label
# Tablo derives from a name falls back to its English spelling.

## Navigation

navigation-blog = Blog
navigation-shop = Shop
navigation-hr = HR
navigation-projects = Projects

## Tenants
# The app panel calls its tenants teams: Tablo's messages that name a tenant read this one term.

-tenant = { $form ->
   *[lower] team
    [upper] Team
    [upper-plural] Teams
    [indefinite] a team
}

## Pages

page-welcome = Welcome
page-welcome-title = Welcome to the Tablo demo!
page-welcome-description = A demo application built on Tablo.
page-shop-dashboard = Shop dashboard
page-hr-dashboard = HR dashboard
page-settings = Settings
page-staff = Staff
page-staff-description = Open the panel as a colleague sees it, for half an hour or until you stop. The audit log names you on every change made meanwhile.

dashboard-shop-description = How the shop is doing, over every customer and order. A period compares its sales with as many days before it, and ends the charts on its last day.
dashboard-hr-description = The workforce, its leave, its projects and what they cost.
dashboard-filter-period = Period

settings-account = Account
settings-security = Security
settings-team = General

## Resources

resource-post = Post
    .plural = Posts
resource-product = Product
    .plural = Products
resource-order = Order
    .plural = Orders
resource-employee = Employee
    .plural = Employees
resource-task = Task
    .plural = Tasks
resource-timesheet = Timesheet
    .plural = Timesheets
resource-leave-request = Leave request
    .plural = Leave requests
resource-expense = Expense
    .plural = Expenses

## Dashboard figures

stat-revenue = Revenue, last { $days } days
stat-orders-placed = Orders, last { $days } days
stat-average-order = Average order, last { $days } days
stat-cancellation-rate = Cancellation rate, last { $days } days
stat-detail-change = { $change } on the { $days } days before
stat-detail-points = { $change } points on the { $days } days before
stat-detail-share = { $share } of orders
stat-detail-share-requests = { $share } of requests
stat-detail-share-articles = { $share } of articles
stat-flagged-orders = Waiting too long
stat-detail-flagged = New for { $new } days or more, processing for { $processing } or more

stat-repeat-customer-rate = Repeat customer rate
stat-detail-repeat = { $repeat } of { $total } customers
stat-avg-items-per-order = Avg items / order
stat-detail-items = { $items } items, { $orders } orders
stat-revenue-per-customer = Revenue / customer
stat-detail-revenue = { $total } total revenue
stat-avg-tenure = Avg tenure
stat-years = { $years } years
stat-detail-tenure = across the active workforce
stat-detail-active = { $count } active employees
stat-contractor-ratio = Contractor ratio
stat-detail-contractors = { $count } contractors
stat-open-capacity = Open capacity
stat-detail-seats = { $seats } seats across the active departments
stat-hours-booked = Hours, last 30 days
stat-billable-share = Billable share, last 30 days
stat-detail-billable = { $hours } h billable
stat-project-budget = Project budget used
stat-detail-budget = { $spent } of { $budget } spent
stat-leave-to-review = Leave to review
stat-detail-pending-leave = waiting for a decision
stat-expenses-to-review = Expenses to review
stat-detail-submitted = submitted, not yet decided
stat-reimbursed = Reimbursed expenses
stat-detail-reimbursed = { $count } claims reimbursed
stat-detail-posts = the blog's articles
stat-detail-products = what the shop sells
stat-detail-orders = placed by customers
stat-detail-employees = across every department
stat-detail-timesheets = booked entries

## Dashboard charts and tables

chart-revenue = Revenue by week
chart-hours = Hours booked by week
chart-weeks = The last twelve weeks, each labelled by the day it began
chart-week-start = { $month }/{ $day }
series-revenue = Revenue
series-billable = Billable
series-not-billable = Not billable
chart-orders = Orders by week
chart-headcount = Headcount by department
chart-headcount-description = Active employees against the seats each department may fill
series-orders = Orders
series-staffed = Staffed
series-seats = Seats
widget-latest-posts = Latest posts
widget-latest-orders = Latest orders
widget-top-products = Top products
widget-leave-queue = Leave to review
widget-expense-queue = Expenses to review
widget-queue-description = Oldest first

## Form tabs

tab-personal = Personal
tab-employment = Employment
tab-overview = Overview
tab-budget = Budget

## Wizard steps

step-details = Details
step-review = Review
step-order-details = Order details
step-notes = Notes

## List tabs


## List summaries

summary-orders = Orders
summary-cancelled = Cancelled
summary-products = Products
summary-low-stock = Under 10 in stock

## Sections

section-address = Address
section-associations = Associations
section-author = Author
section-brand = Brand
section-category = Category
section-comment = Comment
section-content = Content
section-customer = Customer
section-department = Department
section-expense-details = Expense details
section-expense-line = Expense line
section-image = Image
section-images = Images
section-inventory = Inventory
section-leave-details = Leave details
section-order = Order
section-order-address = Order address
section-order-item = Order item
section-payment = Payment
section-post = Post
section-pricing = Pricing
section-product = Product
section-receipt = Receipt
section-review = Review
section-shipping = Shipping
section-status = Status
section-task = Task
section-task-details = Task details
section-timesheet = Timesheet
section-timesheet-entry = Timesheet entry

## Labels

filter-phone-given = With a phone number
filter-phone-missing = Without a phone number
label-gallery = Gallery
label-active = Active
label-email = Email
label-headcount = Headcount
label-hidden = Hidden
label-inactive = Inactive
label-job-title = Job title
label-order-date = Order date
label-parent-category = Parent category
label-parent-department = Parent department
label-phone = Phone
label-price = Price
label-progress = Progress
label-public-visibility = Public visibility
label-shipping-address = Shipping address
label-sku = SKU
label-sku-stock-keeping-unit = SKU (Stock Keeping Unit)
label-status = Status
label-total = Total
label-visible = Visible

## Hints, units and values

hint-post-title = A title readers click
unit-hours = hours
unit-minutes = minutes
unit-per-hour = per hour
unit-per-year = per year
value-days = { $value } days
value-hours = { $value } hours

## Status

status-published = Published
status-draft = Draft

## Fields whose English differs from the name

field-country = Country
field-email = Email address
field-password_confirmation = Confirm password
field-old_price = Compare at price
field-qty = Quantity
field-body = Message
field-shipping_notes = Shipping notes
field-is_active = Active
field-is_visible = Visibility
field-rejection_reason = Reason for rejection
field-reason = Reason
field-subject = Subject
field-receipt_path = Receipt
field-shipping_price = Shipping cost
field-approver = Approver
field-assignee = Assignee
field-author = Author
field-barcode = Barcode (ISBN, UPC, GTIN, etc.)
field-brand = Brand
field-cost = Cost per item
field-customer = Customer
field-date = Date
field-department = Department
field-employee = Employee
field-end_time = End time (half days)
field-expense = Expense
field-gallery = Gallery
field-github_handle = GitHub handle
field-is_billable = Billable
field-kind = Type
field-labels = Labels
field-metadata = Metadata
field-number = Number
field-order = Order
field-parent = Parent
field-post_category = Category
field-post = Post
field-posts = Posts
field-product = Product
field-project = Project
field-published_at = Publishing date
field-reviewed_at = Reviewed at
field-skills = Skills
field-sku = SKU
field-start_time = Start time (half days)
field-state = State / Province
field-street = Street address
field-tags = Tags
field-task = Task
field-twitter_handle = Twitter handle
field-unit_price = Unit price
field-units_sold = Units sold
field-updated_at = Last modified at
field-zip = Zip / Postal code
field-at = When
field-resource = Resource
field-change = Change
field-record = Record
field-user = By

## Options

option-country-code =
    .us = United States
    .gb = United Kingdom
    .de = Germany
    .fr = France
    .ca = Canada
    .au = Australia
    .nl = Netherlands
    .br = Brazil
    .jp = Japan
    .in = India

option-currency-code =
    .usd = US Dollar
    .eur = Euro
    .gbp = British Pound
    .cad = Canadian Dollar
    .aud = Australian Dollar
    .jpy = Japanese Yen
    .brl = Brazilian Real
    .inr = Indian Rupee

option-employment-type =
    .full_time = Full Time
    .part_time = Part Time
    .contractor = Contractor
    .intern = Intern

option-expense-category =
    .travel = Travel
    .meals = Meals
    .supplies = Supplies
    .equipment = Equipment
    .software = Software
    .other = Other

option-expense-status =
    .draft = Draft
    .submitted = Submitted
    .approved = Approved
    .rejected = Rejected
    .reimbursed = Reimbursed

option-leave-status =
    .pending = Pending
    .approved = Approved
    .rejected = Rejected
    .taken = Taken
    .cancelled = Cancelled

option-leave-type =
    .annual = Annual Leave
    .sick = Sick Leave
    .personal = Personal Leave
    .unpaid = Unpaid Leave
    .parental = Parental Leave

option-order-status =
    .new = New
    .processing = Processing
    .shipped = Shipped
    .delivered = Delivered
    .cancelled = Cancelled

option-payment-method =
    .credit_card = Credit Card
    .bank_transfer = Bank Transfer
    .paypal = PayPal
    .apple_pay = Apple Pay
    .google_pay = Google Pay

option-project-status =
    .planning = Planning
    .active = Active
    .on_hold = On Hold
    .completed = Completed
    .cancelled = Cancelled

option-task-priority =
    .low = Low
    .medium = Medium
    .high = High
    .critical = Critical

option-task-status =
    .backlog = Backlog
    .todo = To Do
    .in_progress = In Progress
    .in_review = In Review
    .completed = Completed
    .cancelled = Cancelled

option-product-type =
    .deliverable = Deliverable
    .downloadable = Downloadable

## Flash messages

flash-budget-saved = Budget saved
flash-department-replicated = Department replicated
flash-employees-moved =
    { $count ->
        [one] Moved one employee
       *[other] Moved { $count } employees
    }
flash-employees-updated =
    { $count ->
        [one] Updated one employee
       *[other] Updated { $count } employees
    }
flash-leaves-approved =
    { $count ->
        [one] Approved one leave request
       *[other] Approved { $count } leave requests
    }
flash-leaves-rejected =
    { $count ->
        [one] Rejected one leave request
       *[other] Rejected { $count } leave requests
    }
flash-project-saved = Project status saved
flash-project-on-hold = Project put on hold
flash-project-resumed = Project resumed
flash-project-completed = Project completed
flash-task-started = Task started
flash-task-sent-to-review = Task sent to review
flash-task-completed = Task completed
flash-tasks-assigned =
    { $count ->
        [one] Assigned one task
       *[other] Assigned { $count } tasks
    }
flash-tasks-updated =
    { $count ->
        [one] Updated one task
       *[other] Updated { $count } tasks
    }
flash-task-replicated = Task replicated
flash-timesheets-billable =
    { $count ->
        [one] Marked one timesheet billable
       *[other] Marked { $count } timesheets billable
    }
flash-timesheets-not-billable =
    { $count ->
        [one] Marked one timesheet not billable
       *[other] Marked { $count } timesheets not billable
    }
flash-expenses-submitted =
    { $count ->
        [one] Submitted one expense for approval
       *[other] Submitted { $count } expenses for approval
    }
flash-expense-approved = Expense approved
flash-expense-rejected = Expense rejected
flash-expense-reimbursed = Expense reimbursed
flash-expense-flagged = Expense flagged and returned to draft
flash-email-sent =
    { $count ->
        [one] Email sent to one customer
       *[other] Email sent to { $count } customers
    }
flash-order-processing = Order is now processing
flash-order-shipped = Order shipped
flash-order-delivered = Order marked as delivered
flash-order-replicated = Order replicated
flash-order-cancelled = Order cancelled

## Validation

error-credentials-locked = Staff accounts cannot change their email or password
error-email-taken = Email has already been taken
error-team-name-required = Enter a name for your team
error-quantity-negative = Quantity must be zero or more
error-order-has-payments = An order with payments cannot be deleted; cancel it instead
reason-timesheet-locked = Only the last week's timesheets can change
reason-expense-no-amount = An expense needs an amount before it is submitted
error-leave-overlap = These dates overlap another leave request of this employee

## Notifications

notice-new-order = New order
notice-order-placed = { $customer } placed an order.
notice-order-placed-unknown = An order was placed.
notice-new-comment = New comment
notice-comment-left = { $customer } left a comment.
notice-comment-left-unknown = A comment was left.
notice-view = View

## Actions

action-activate = Activate
action-deactivate = Deactivate
action-adjust-budget = Adjust budget
action-adjust-price = Adjust price
action-adjust-stock = Adjust stock
action-approve = Approve
action-approve-pending = Approve pending
action-assign = Assign
action-bulk-set-active = Set active
action-bulk-set-status = Set status
action-cancel = Cancel
action-change-department = Change department
action-change-status = Change status
action-complete = Complete
action-deliver = Deliver
action-flag = Flag
action-mark-billable = Mark billable
action-mark-non-billable = Mark non billable
action-process = Process
action-put-on-hold = Put on hold
action-reimburse = Reimburse
action-reject = Reject
action-replicate = Replicate
action-resume = Resume
action-send-email = Send email
action-send-to-review = Send to review
action-set-priority = Set priority
action-set-status = Set status
action-set-visibility = Set visibility
action-move-up = Move up
action-move-down = Move down
action-add-metadata = Add entry
action-ship = Ship
action-show-on-map = Show on map
action-start = Start
action-submit = Submit
action-toggle-active = Toggle active
action-toggle-billable = Toggle billable
action-toggle-publish = Toggle publish
action-visit-website = Visit website

## Welcome page

home-lang = en
home-document-title = Tablo demo
home-title = Admin panels for Rust, declared beside your models.
home-lead = Three panels made with Tablo: resources, tables, forms, actions and dashboards from a few declarations.
home-cta-admin = Explore as administrator
home-cta-guide = Read the guide
home-admin-title = Back office
home-app-title = Customer app
home-sign-up = Create an account
home-features-title = What the demo covers
home-feature-resources = Resources
home-feature-resources-body = Twenty-one tables across a shop, HR, projects and a blog, each with list, form and detail.
home-feature-tables = Tables
home-feature-tables-body = Search, sort, filters, grouping, status tabs, CSV export and cells that save in place.
home-feature-forms = Forms
home-feature-forms-body = Choices, tags, dependent and conditional fields, tabs and image galleries.
home-feature-actions = Actions
home-feature-actions-body = Row and bulk actions with input dialogs, confirmations and notifications.
home-feature-dashboards = Dashboards
home-feature-dashboards-body = Stat tiles with trends and sparklines, line and bar charts, and queues to clear.
home-feature-auth = Auth and roles
home-feature-auth-body = Four staff roles, sign-up, email verification, two-factor sign-in and teams.
home-feature-search = Global search
home-feature-search-body = One box over every resource, respecting each list's tenancy and field access.
home-feature-i18n = Translations
home-feature-i18n-body = English and French throughout, picked from the browser or the language menu.
home-built-with = Built with Tablo, Topcoat and Toasty.
home-password-note = Each account below signs in with one click. Every password is
home-admin-body = The back office of a small company: shop, HR, projects and blog. Each staff role sees its own part.
home-app-body = The customer side of a SaaS: sign-up, two-factor sign-in, teams and profile.
home-sign-in-page = Sign-in page
home-partners-title = Partner portal
home-partners-body = A panel behind the host app's own sign-in, as an app with its own login page or SSO would mount it. There is no Tablo login form here.
home-partners-open = Open the portal
partner-sign-in-title = Sign in to the partner portal
partner-sign-in-body = This page belongs to the demo, not to Tablo. It keeps its own session, and the panel trusts whoever it names.
partner-sign-in-submit = Continue as { $name }
page-partner-home = Partner home
copy-partner-welcome = Signed in as { $name } by the host app.
copy-partner-body = Sign out ends the panel's session, then the host app's. Until then the panel asks the app who you are on every request.
home-role-administrator = Administrator
home-role-administrator-about = Everything, the audit log included, and acting as a colleague
home-role-shop-manager = Shop manager
home-role-shop-manager-about = Runs the shop and the blog; reads the projects; no HR
home-role-hr-manager = HR manager
home-role-hr-manager-about = Runs HR and the projects, pay included; no shop
home-role-viewer = Viewer
home-role-viewer-about = Reads everything but pay; changes nothing
home-role-member = Member of two teams
home-role-member-about = Switch teams, edit the profile, turn on two-factor
home-guide = Guide
home-source = Source

## Staff

staff-name = Name
staff-email = Email
staff-role = Role
