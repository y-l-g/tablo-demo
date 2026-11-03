//! Leave requests: the list with its figures by status, the form that refuses overlapping
//! dates, and the review.

use jiff::Timestamp;
use tablo::prelude::*;
use tablo::{FormScalar, SelectColumn, db};
use topcoat::{Result, context::Cx, icon::iconify::iconify_icon, view::View};

use super::{EmployeeResource, LeaveRequest, LeaveStatus, LeaveType};
use crate::access::{self, Domain};
use crate::shared::stats::{linked, percent, share};

/// Leave requests.
pub struct LeaveRequestResource;

impl Resource for LeaveRequestResource {
    type Model = LeaveRequest;
    type Form = LeaveRequestForm;

    fn declare() -> ResourceDef<Self> {
        let c = LeaveRequestForm::controls();
        let start_date = c.start_date;
        let end_date = c.end_date.gte(&start_date);
        ResourceDef::new()
            .icon(iconify_icon!("lucide:calendar-days"))
            .navigation_group(tablo::text!("navigation-hr"))
            .navigation_order(2)
            .navigation_badge(LeaveRequest::fields().status().eq(LeaveStatus::Pending))
            .policy(access::policy(Domain::Hr))
            .table(
                Table::new((
                    RelationColumn::of::<EmployeeResource>(relation!(LeaveRequest.employee)),
                    TextColumn::new(lens!(LeaveRequest.kind)).badge(),
                    SelectColumn::new(lens!(LeaveRequest.status)),
                    DateColumn::new(lens!(LeaveRequest.start_date)).sortable(),
                    DateColumn::new(lens!(LeaveRequest.end_date)).sortable(),
                    TextColumn::new(lens!(LeaveRequest.days_requested))
                        .numeric(1)
                        .sortable(),
                    RelationColumn::of::<EmployeeResource>(relation!(LeaveRequest.approver))
                        .toggleable()
                        .hidden_by_default(),
                ))
                .default_sort_desc(lens!(LeaveRequest.start_date))
                .action_layout(ActionLayout::Menu)
                // The colour reads only the request's status, so it reads a copy without the
                // employee the first column loads, and their pay stays out of its reach.
                .row_color(|leave: &LeaveRequest| match leave.status {
                    LeaveStatus::Pending => Some(Color::Warning),
                    LeaveStatus::Rejected => Some(Color::Danger),
                    _ => None,
                })
                .row_color_reading(Reads::of(lens!(LeaveRequest.status)))
                .option_tabs(lens!(LeaveRequest.status))
                .filters((SelectFilter::of(lens!(LeaveRequest.kind)),)),
            )
            .detail(Detail::new((
                Section::new(tablo::text!("section-leave-details")).columns(Grid::new(3).columns(
                    (
                        RelationColumn::of::<EmployeeResource>(relation!(LeaveRequest.employee)),
                        TextColumn::new(lens!(LeaveRequest.kind)).badge(),
                        TextColumn::new(lens!(LeaveRequest.status)).badge(),
                        DateColumn::new(lens!(LeaveRequest.start_date)),
                        DateColumn::new(lens!(LeaveRequest.end_date)),
                        TextColumn::new(lens!(LeaveRequest.days_requested)).format(|days| {
                            tablo::text!("value-days", value = format!("{days:.1}"))
                        }),
                        TextColumn::new(lens!(LeaveRequest.start_time)),
                        TextColumn::new(lens!(LeaveRequest.end_time)),
                        TextColumn::new(lens!(LeaveRequest.reason)),
                    ),
                )),
                Section::new(tablo::text!("section-review")).columns((
                    RelationColumn::of::<EmployeeResource>(relation!(LeaveRequest.approver)),
                    DateColumn::new(lens!(LeaveRequest.reviewed_at)),
                    TextColumn::new(lens!(LeaveRequest.reviewer_notes)),
                )),
            )))
            .form(Schema::new((
                Section::new(tablo::text!("section-leave-details")).schema(Grid::new(2).schema((
                    c.employee_id.searchable(),
                    c.kind.radio(),
                    start_date,
                    end_date,
                    c.start_time,
                    c.end_time,
                    c.reason.multiline(4),
                ))),
                Section::new(tablo::text!("section-review")).schema(Grid::new(2).schema((
                    c.status.buttons(),
                    c.approver_id.searchable(),
                    c.reviewer_notes.multiline(3),
                ))),
            )))
            .record_title(lens!(LeaveRequest.reason))
            .create_column(lens!(LeaveRequest.days_requested))
            .action::<ApproveLeaveRequest>()
            .action::<RejectLeaveRequest>()
            .list_widget::<LeaveStatusWidget>()
    }

    /// Refuses dates that overlap another request of the employee, then stores the day count
    /// the submitted dates cover.
    async fn create_record(
        _cx: &Cx,
        form: LeaveRequestForm,
        ex: &mut dyn toasty::Executor,
    ) -> Result<LeaveRequest, WriteError<LeaveRequestFormField>> {
        refuse_overlap(
            &mut *ex,
            form.employee_id,
            form.start_date,
            form.end_date,
            None,
        )
        .await?;
        let days = LeaveRequest::days_between(form.start_date, form.end_date);
        Ok(form.into_create().days_requested(days).exec(ex).await?)
    }

    /// Refuses dates that overlap another request of the employee, then recomputes the day
    /// count from the stored dates. A refusal rolls the edit back.
    async fn update_record(
        cx: &Cx,
        record: LeaveRequest,
        posted: tablo::Posted<LeaveRequestForm>,
        ex: &mut dyn toasty::Executor,
    ) -> Result<LeaveRequest, WriteError<LeaveRequestFormField>> {
        let mut record = tablo::write_update::<Self>(cx, record, posted, ex).await?;
        refuse_overlap(
            &mut *ex,
            record.employee_id,
            record.start_date,
            record.end_date,
            Some(record.id),
        )
        .await?;
        let days = LeaveRequest::days_between(record.start_date, record.end_date);
        if record.days_requested != days {
            record.update().days_requested(days).exec(&mut *ex).await?;
            record.days_requested = days;
        }
        Ok(record)
    }
}

/// Refuses the dates `start` to `end` of `employee_id` when another request of that employee
/// covers any of them and is neither rejected nor cancelled. `except` is the request being
/// edited, which would otherwise overlap its own dates.
async fn refuse_overlap(
    ex: &mut dyn toasty::Executor,
    employee_id: u64,
    start: jiff::civil::Date,
    end: jiff::civil::Date,
    except: Option<u64>,
) -> Result<(), WriteError<LeaveRequestFormField>> {
    let fields = LeaveRequest::fields();
    let overlapping = LeaveRequest::filter(
        fields
            .employee_id()
            .eq(employee_id)
            .and(fields.start_date().le(end))
            .and(fields.end_date().ge(start))
            .and(fields.status().ne(LeaveStatus::Rejected))
            .and(fields.status().ne(LeaveStatus::Cancelled)),
    )
    .exec(&mut *ex)
    .await?;
    if overlapping.iter().any(|leave| Some(leave.id) != except) {
        return Err(WriteError::field(
            LeaveRequestFormField::StartDate,
            tablo::text!("error-leave-overlap"),
        ));
    }
    Ok(())
}

/// The leave request's record form.
#[derive(tablo::RecordForm)]
#[form(model = LeaveRequest)]
pub struct LeaveRequestForm {
    #[form(relationship = EmployeeResource)]
    pub employee_id: u64,
    #[form(options)]
    pub kind: LeaveType,
    #[form(options)]
    pub status: LeaveStatus,
    pub start_date: jiff::civil::Date,
    pub end_date: jiff::civil::Date,
    pub start_time: Option<jiff::civil::Time>,
    pub end_time: Option<jiff::civil::Time>,
    pub reason: String,
    pub reviewer_notes: Option<String>,
    #[form(relationship = EmployeeResource)]
    pub approver_id: Option<u64>,
}

/// What a rejection asks for.
#[derive(FormInput)]
pub struct Rejection {
    #[form(multiline = 3)]
    pub rejection_reason: String,
}

/// Approves a pending leave request, or every pending one of the selection, confirming first.
pub struct ApproveLeaveRequest;

impl Action<LeaveRequestResource> for ApproveLeaveRequest {
    type Input = ();
    const NAME: &'static str = "approve";
    const PLACES: Places = Places::ROW.with(Places::DETAIL).with(Places::BULK);
    const CONFIRM: bool = true;

    fn can_run(_cx: &Cx, request: &LeaveRequest) -> bool {
        request.status == LeaveStatus::Pending
    }

    async fn run(
        _cx: &Cx,
        requests: &[LeaveRequest],
        (): (),
        ex: &mut dyn toasty::Executor,
    ) -> Result<Outcome, WriteError<std::convert::Infallible>> {
        review(requests, LeaveStatus::Approved, None, ex).await
    }

    fn success(count: usize) -> Text {
        tablo::text!("flash-leaves-approved", count = count)
    }
}

/// Rejects a pending leave request, or every pending one of the selection, with the reason.
pub struct RejectLeaveRequest;

impl Action<LeaveRequestResource> for RejectLeaveRequest {
    type Input = Rejection;
    const NAME: &'static str = "reject";
    const PLACES: Places = Places::ROW.with(Places::DETAIL).with(Places::BULK);

    fn can_run(_cx: &Cx, request: &LeaveRequest) -> bool {
        request.status == LeaveStatus::Pending
    }

    async fn run(
        _cx: &Cx,
        requests: &[LeaveRequest],
        rejection: Rejection,
        ex: &mut dyn toasty::Executor,
    ) -> Result<Outcome, WriteError<RejectionField>> {
        review(
            requests,
            LeaveStatus::Rejected,
            Some(rejection.rejection_reason),
            ex,
        )
        .await
    }

    fn success(count: usize) -> Text {
        tablo::text!("flash-leaves-rejected", count = count)
    }
}

/// Records `status` and the review time on every pending request, with the reviewer's notes.
async fn review<K>(
    requests: &[LeaveRequest],
    status: LeaveStatus,
    notes: Option<String>,
    ex: &mut dyn toasty::Executor,
) -> Result<Outcome, WriteError<K>> {
    for request in requests {
        let mut update = LeaveRequest::filter_by_id(request.id).update();
        update = update.status(status).reviewed_at(Some(Timestamp::now()));
        // Approving keeps whatever notes the request already carries.
        if let Some(notes) = notes.clone() {
            update = update.reviewer_notes(Some(notes));
        }
        update.exec(&mut *ex).await?;
    }
    Ok(Outcome::done())
}

/// The leave requests by status, a tile per status linked to the requests in it, above the leave
/// list, following its tab, search and filters. Only a user who may open the leave list sees it.
pub struct LeaveStatusWidget;

impl Widget for LeaveStatusWidget {
    fn declare() -> WidgetDef {
        WidgetDef::new()
            .full_width()
            .access(can_list::<LeaveRequestResource>)
    }

    async fn render(cx: &Cx) -> Result<impl View> {
        let requests = list_query::<LeaveRequestResource>(cx).await?;
        let total = db::count(cx, requests.clone()).await?;
        let url = tablo::url::resource::<LeaveRequestResource>(cx);
        let mut tiles = Vec::new();
        for &status in LeaveStatus::ALL {
            let query = requests
                .clone()
                .filter(LeaveRequest::fields().status().eq(status));
            let count = db::count(cx, query).await?;
            let tile = Stat::new(status.label(), count.to_string())
                .detail(tablo::text!(
                    "stat-detail-share-requests",
                    share = percent(share(count as f64, total as f64))
                ))
                .color(status.color().unwrap_or(Color::Gray));
            tiles.push(linked(
                tile,
                url.as_ref()
                    .map(|url| format!("{url}?tab={}", status.value())),
            ));
        }
        Ok(tablo::stats(cx, tiles))
    }
}
