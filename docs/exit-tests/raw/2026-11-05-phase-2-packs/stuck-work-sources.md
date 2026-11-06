# Context pack

Task: The stuck-work listing cannot tell "nothing is waiting" from "this source cannot count", and it names only the sources that returned records. Add an optional waiting total per record and a per-source summary to the PortListStuckWork response without breaking the wire, fill them in the server, and show them on both the Rust and the Node operations pages.

## Seeds

- `pkg:cargo/stuck-work` Package 1.00 (identifier `stuck-work`)
- `typescript:apps-web/causeway-web/src/pages/data/stuck-work.ts::countsOf` Function 0.90 (text)
- `rust:apps/causeway-server-web/src/views/platform/operations/stuck_work.rs::stuck_work_record` Function 0.88 (text)
- `rust:apps/causeway-server/src/routes/pages/metadata_ops/stuck_work.rs::stuck_work_record` Function 0.87 (text)
- `rust:apps/causeway-server-web/src/views/platform/operations/stuck_work.rs::StuckWorkListPage::new` Method 0.83 (text)
- `rust:crates/stuck-work/src/port.rs::StuckWorkSource::act` Method 0.82 (text)
- `rust:apps/causeway-server/src/metadata_admin/handlers.rs::stuck_work_record` Function 0.82 (text)
- `rust:crates/stuck-work/src/port.rs::StuckWorkSource::reclaim` Method 0.81 (text)
- `rust:apps/causeway-server-web/src/views/platform/operations/stuck_work.rs::StuckWorkListPage::narrowed` Method 0.80 (text)
- `rust:apps/causeway-server/src/metadata_admin/stuck_work.rs::StuckWorkCounts::in_review` Method 0.80 (text)
- `rust:apps/causeway-server/src/metadata_admin/stuck_work.rs::StuckWorkPage` Struct 0.80 (text)
- `python:services-py/relay-grpc/gen/causeway/metadata/v2/admin_pb2.pyi::PortListStuckWorkRequest` Class 0.40 (identifier `PortListStuckWork`)
- `python:services-py/relay-grpc/gen/causeway/metadata/v2/admin_pb2.pyi::PortListStuckWorkRequest::__init__` Method 0.40 (identifier `PortListStuckWork`)
- `python:services-py/relay-grpc/gen/causeway/metadata/v2/admin_pb2.pyi::PortListStuckWorkResponse` Class 0.40 (identifier `PortListStuckWork`)
- `typescript:services-node/relay-grpc/gen/causeway/metadata/v2/admin_pb.ts::PortListStuckWorkRequest` TypeAlias 0.40 (identifier `PortListStuckWork`)
- `typescript:services-node/relay-grpc/gen/causeway/metadata/v2/admin_pb.ts::PortListStuckWorkResponse` TypeAlias 0.40 (identifier `PortListStuckWork`)

Budget 30000 tokens, used 29973. Seeds 16, candidates 1456, included 85, left out 2311.

## Context

### 1. Package cargo/stuck-work
`pkg:cargo/stuck-work`  crates/stuck-work/Cargo.toml:2  pkg:cargo/stuck-work  score 3.25  
`cargo/stuck-work`  
members 3  
dependents 10: `cargo/causeway-server` (apps/causeway-server/Cargo.toml:63), `cargo/causeway-server-web` (apps/causeway-server-web/Cargo.toml:47), `cargo/governance-policy-component` (component/governance/policy/Cargo.toml:1-59) … 7 more  
dependencies 5: `cargo/tokio` (crates/stuck-work/Cargo.toml:20), `cargo/chrono` (crates/stuck-work/Cargo.toml:1-20), `cargo/serde` (crates/stuck-work/Cargo.toml:1-20) … 2 more  
designs 1: `docs/rfd/RFD-2n2tcfe2-operational-resilience-and-the-stuck-work-surface.md` (docs/rfd/RFD-2n2tcfe2-operational-resilience-and-the-stuck-work-surface.md:250)  
documents 1: `docs/plans/OPERATIONAL-RESILIENCE-WORKPLAN.md` (docs/plans/OPERATIONAL-RESILIENCE-WORKPLAN.md:34)  
importers 23: `apps/causeway-server/src/metadata_admin/handlers/caller_tests.rs` (apps/causeway-server/src/metadata_admin/handlers/caller_tests.rs:11-14), `component/platform/coherence/src/bin/causeway_coherence.rs` (component/platform/coherence/src/bin/causeway_coherence.rs:28), `component/platform/event-operations/src/lib.rs` (component/platform/event-operations/src/lib.rs:13-16) … 20 more  

### 2. Function stuck_work_record
`rust:apps/causeway-server-web/src/views/platform/operations/stuck_work.rs::stuck_work_record`  apps/causeway-server-web/src/views/platform/operations/stuck_work.rs:176-206  pkg:cargo/causeway-server-web  score 2.74  
`pub fn stuck_work_record(page: &StuckWorkRecordPage) -> Markup`  
The detail: what the work was, what waits on it, and the actions.  
callers 1: `stuck_work_record` (apps/causeway-server/src/routes/pages/metadata_ops/stuck_work.rs:251)  
calls 11: `tone_for` (apps/causeway-server-web/src/views/platform/operations/stuck_work.rs:185), `payload_section` (apps/causeway-server-web/src/views/platform/operations/stuck_work.rs:191), `actions` (apps/causeway-server-web/src/views/platform/operations/stuck_work.rs:202) … 8 more  
uses_types 1: `StuckWorkRecordPage` (apps/causeway-server-web/src/views/platform/operations/stuck_work.rs:176)  
apps/causeway-server-web/src/views/platform/operations/stuck_work.rs:176-206
```rust
pub fn stuck_work_record(page: &StuckWorkRecordPage) -> Markup {
    let record = &page.detail.record;
    let ended = record.lifecycle.has_ended();
    html! {
        div style=(PAGE_WRAPPER) {
            div style="margin-bottom:20px;" {
                h1 style=(H1) { (record.kind) " · " (record.subject.id) }
                p style=(LEAD) {
                    (record.source.as_str()) " · record " (record.id.as_str()) " · "
                    (status_pill(record.lifecycle.as_str(), tone_for(record.lifecycle.as_str())))
                }
            }
            (feedback(page.notice.as_deref(), page.error.as_deref()))
            (blast_radius(record))
            (failure_history(page))
            (payload_section(&page.detail.payload))
            @if ended {
                section style=(CARD) {
                    p style="margin:0;color:var(--fg-2);font-size:13px;" {
                        "This record has ended: "
                        @if let Some(resolution) = record.lifecycle.resolution() {
                            (resolution.by.as_str()) " — " (resolution.reason.as_str())
                        }
                    }
                }
            } @else {
                (actions(page))
            }
        }
    }
}
```

### 3. Function stuck_work_record
`rust:apps/causeway-server/src/routes/pages/metadata_ops/stuck_work.rs::stuck_work_record`  apps/causeway-server/src/routes/pages/metadata_ops/stuck_work.rs:216-256  pkg:cargo/causeway-server  score 2.16  
`pub async fn stuck_work_record( State(state): State<AppState>, Path((source, id)): Path<(String, String)>, Query(params): Query<StuckWorkRecordQuery>, Extension(caller): Extension<BrowserCaller>, method: Method, uri: Uri, headers: HeaderMap, ) -> Response`  
One record, with what waits on it and what may be done about it, (doc continues, 89 bytes)  
calls 16: `render` (apps/causeway-server/src/routes/pages/metadata_ops/stuck_work.rs:244), `stuck_work_record` (apps/causeway-server/src/routes/pages/metadata_ops/stuck_work.rs:251), `SourceId::parse` (apps/causeway-server/src/routes/pages/metadata_ops/stuck_work.rs:235) … 13 more  
importers 1: `apps/causeway-server/src/routes/pages.rs` (apps/causeway-server/src/routes/pages.rs:53-58)  
referenced_by 1: `routes` (apps/causeway-server/src/routes/app/operations.rs:35)  
uses_types 3: `AppState` (apps/causeway-server/src/routes/pages/metadata_ops/stuck_work.rs:217), `BrowserCaller` (apps/causeway-server/src/routes/pages/metadata_ops/stuck_work.rs:220), `StuckWorkRecordQuery` (apps/causeway-server/src/routes/pages/metadata_ops/stuck_work.rs:219)  
apps/causeway-server/src/routes/pages/metadata_ops/stuck_work.rs:216-256
```rust
pub async fn stuck_work_record(
    State(state): State<AppState>,
    Path((source, id)): Path<(String, String)>,
    Query(params): Query<StuckWorkRecordQuery>,
    Extension(caller): Extension<BrowserCaller>,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
) -> Response {
    if let Some(refusal) = refused_unless_admitted(
        &state,
        &caller,
        (&method, &uri, &headers),
        &record_breadcrumbs(),
    )
    .await
    {
        return refusal;
    }
    let (Ok(source_id), Ok(record_id)) = (SourceId::parse(&source), RecordId::parse(&id)) else {
        return refused_to_list("a record is named by its source and its id");
    };
    let page = match registry::page(state.stuck_work_sources(), &source_id, &record_id).await {
        Ok(page) => page,
        Err(error) => return refused_to_list(&error.to_string()),
    };
    let view = StuckWorkRecordPage::new(page.detail, page.corrections, page.failing_seat)
        .with_feedback(params.notice, params.error);
    render(
        &state,
        "Stuck work",
        "workflows",
        record_breadcrumbs(),
        Vec::new(),
        "",
        causeway_server_web::views::platform::operations::stuck_work_record(&view),
        None,
    )
    .await
    .into_response()
}
```

### 4. Function countsOf
`typescript:apps-web/causeway-web/src/pages/data/stuck-work.ts::countsOf`  apps-web/causeway-web/src/pages/data/stuck-work.ts:111-121  pkg:npm/@causeway/web  score 1.91  
`function countsOf(records: readonly StuckWorkRecord[]): readonly SourceCount[]`  
Each source's open records and, of those, the ones waiting on a person.  
callers 2: `stuckWork` (apps-web/causeway-web/src/pages/data/stuck-work.ts:148), `the stuck-work filters, as the shell's `admits`::counts open and in-review work per source` (apps-web/causeway-web/test/app-operations.test.ts:196)  
importers 1: `apps-web/causeway-web/test/app-operations.test.ts` (apps-web/causeway-web/test/app-operations.test.ts:10-15)  
tests 1: `the stuck-work filters, as the shell's `admits`::counts open and in-review work per source` (apps-web/causeway-web/test/app-operations.test.ts:196)  
apps-web/causeway-web/src/pages/data/stuck-work.ts:111-121
```typescript
export function countsOf(records: readonly StuckWorkRecord[]): readonly SourceCount[] {
  const sources = [...new Set(records.map((record) => record.source))];
  return sources.map((source) => {
    const own = records.filter((record) => record.source === source);
    return {
      source,
      open: own.length,
      inReview: own.filter((record) => record.status === "in_review").length,
    };
  });
}
```

### 5. Method StuckWorkSource::act
`rust:crates/stuck-work/src/port.rs::StuckWorkSource::act`  crates/stuck-work/src/port.rs:860-865  pkg:cargo/stuck-work  score 1.89  
`fn act( &self, id: &RecordId, action: &StuckWorkAction, actor: &Actor, ) -> StuckWorkFuture<'_, Result<StuckWorkRecord, StuckWorkError>>;`  
Acts on one record, returning it as it now stands.  
dispatches_to 9: `IngressDeadLetterStuckWork::act` (component/platform/event-operations/src/stuck_work.rs:393-403), `CorrectiveDeferrals::act` (component/governance/policy/src/deferral.rs:301-389), `tests::a_source_that_shares_no_payload_still_has_a_detail::Bare::act` (crates/stuck-work/src/port.rs:1139-1146) … 6 more  
uses_types 6: `StuckWorkFuture` (crates/stuck-work/src/port.rs:865), `Actor` (crates/stuck-work/src/port.rs:864), `StuckWorkRecord` (crates/stuck-work/src/port.rs:865) … 3 more  
crates/stuck-work/src/port.rs:860-865
```rust
    fn act(
        &self,
        id: &RecordId,
        action: &StuckWorkAction,
        actor: &Actor,
    ) -> StuckWorkFuture<'_, Result<StuckWorkRecord, StuckWorkError>>;
```

### 6. Struct StuckWorkRecord
`rust:crates/stuck-work/src/port.rs::StuckWorkRecord`  crates/stuck-work/src/port.rs:198-219  pkg:cargo/stuck-work  score 1.81  via `StuckWorkListPage::new` -> USES_TYPE (apps/causeway-server-web/src/views/platform/operations/stuck_work.rs:53, 1.00)  
`pub struct StuckWorkRecord`  
One piece of work that could not finish, as every source reports it.  
members 5  
importers 10: `tests` (component/platform/supervision/src/pass.rs:238-241), `component/metadata/core/src/stuck_work.rs` (component/metadata/core/src/stuck_work.rs:14-19), `apps/causeway-server/src/metadata_admin/stuck_work.rs` (apps/causeway-server/src/metadata_admin/stuck_work.rs:13-16) … 7 more  
used_by 64: `record_from_row` (component/metadata/core/src/stuck_work.rs:332), `CorrectiveDeferrals::get` (component/governance/policy/src/deferral.rs:293), `tests::record` (crates/stuck-work/src/port.rs:960) … 61 more  
uses_types 8: `Attempts` (crates/stuck-work/src/port.rs:210), `Subject` (crates/stuck-work/src/port.rs:206), `Reason` (crates/stuck-work/src/port.rs:214) … 5 more  
crates/stuck-work/src/port.rs:198-219
```rust
pub struct StuckWorkRecord {
    /// Which subsystem owns it.
    pub source: SourceId,
    /// Its identity there.
    pub id: RecordId,
    /// What the work was: `projection`, `search-index`, `sniff`.
    pub kind: String,
    /// What it is about.
    pub subject: Subject,
    /// When it first failed.
    pub first_failed_at: DateTime<Utc>,
    /// How often it has been tried.
    pub attempts: Attempts,
    /// Whether trying again could help, when the source has judged it.
    pub failure_class: Option<FailureClass>,
    /// What went wrong, in words an operator can act on.
    pub reason: Option<Reason>,
    /// Where it is in its life.
    pub lifecycle: Lifecycle,
    /// What waits on it.
    pub blast_radius: BlastRadius,
}
```

### 7. Method StuckWorkListPage::new
`rust:apps/causeway-server-web/src/views/platform/operations/stuck_work.rs::StuckWorkListPage::new`  apps/causeway-server-web/src/views/platform/operations/stuck_work.rs:52-69  pkg:cargo/causeway-server-web  score 1.79  
`pub fn new( records: Vec<StuckWorkRecord>, counts: Vec<SourceCount>, unavailable: Vec<String>, ) -> Self`  
The page over what the registry returned.  
callers 1: `stuck_work` (apps/causeway-server/src/routes/pages/metadata_ops/stuck_work.rs:176)  
uses_types 2: `SourceCount` (apps/causeway-server-web/src/views/platform/operations/stuck_work.rs:54), `StuckWorkRecord` (apps/causeway-server-web/src/views/platform/operations/stuck_work.rs:53)  
apps/causeway-server-web/src/views/platform/operations/stuck_work.rs:52-69
```rust
    pub fn new(
        records: Vec<StuckWorkRecord>,
        counts: Vec<SourceCount>,
        unavailable: Vec<String>,
    ) -> Self {
        Self {
            records,
            counts,
            unavailable,
            source_filter: None,
            include_ended: false,
            subject_filter: None,
            class_filter: None,
            older_than_hours: None,
            notice: None,
            error: None,
        }
    }
```

### 8. File apps/causeway-server-web/src/views/platform/operations/stuck_work.rs
`file:apps/causeway-server-web/src/views/platform/operations/stuck_work.rs`  apps/causeway-server-web/src/views/platform/operations/stuck_work.rs  pkg:cargo/causeway-server-web  score 1.79  via `stuck_work_record` <- CONTAINS (apps/causeway-server-web/src/views/platform/operations/stuck_work.rs:176-206, 1.00)  
members 17  
designs 1: `docs/rfd/RFD-2n2tcfe2-operational-resilience-and-the-stuck-work-surface.md` (docs/rfd/RFD-2n2tcfe2-operational-resilience-and-the-stuck-work-surface.md:273)  
documents 1: `docs/plans/OPERATIONAL-RESILIENCE-WORKPLAN.md` (docs/plans/OPERATIONAL-RESILIENCE-WORKPLAN.md:81)  
imports 12: `Completeness` (apps/causeway-server-web/src/views/platform/operations/stuck_work.rs:11), `cargo/stuck-work` (apps/causeway-server-web/src/views/platform/operations/stuck_work.rs:11), `cargo/maud` (apps/causeway-server-web/src/views/platform/operations/stuck_work.rs:10) … 9 more  

### 9. Function stuck_work_record
`rust:apps/causeway-server/src/metadata_admin/handlers.rs::stuck_work_record`  apps/causeway-server/src/metadata_admin/handlers.rs:14-49  pkg:cargo/causeway-server  score 1.67  
`fn stuck_work_record(record: &::stuck_work::StuckWorkRecord) -> pb::StuckWorkRecord`  
One record on the wire. The shape is what every source shares; the (doc continues, 412 bytes)  
callers 1: `MetadataAdmin::port_act_on_stuck_work` (apps/causeway-server/src/metadata_admin/handlers.rs:605)  
references 3: `stuck_work` (apps/causeway-server/src/metadata_admin/handlers.rs:14), `stuck_work` (apps/causeway-server/src/metadata_admin/handlers.rs:14), `stuck_work` (apps/causeway-server/src/metadata_admin/handlers.rs:14)  
uses_types 1: `StuckWorkRecord` (apps/causeway-server/src/metadata_admin/handlers.rs:14)  
apps/causeway-server/src/metadata_admin/handlers.rs:14-49
```rust
fn stuck_work_record(record: &::stuck_work::StuckWorkRecord) -> pb::StuckWorkRecord {
    let resolution = record.lifecycle.resolution();
    pb::StuckWorkRecord {
        source: record.source.as_str().to_owned(),
        id: record.id.as_str().to_owned(),
        kind: record.kind.clone(),
        subject_kind: record.subject.kind.clone(),
        subject_id: record.subject.id.clone(),
        subject_label: record.subject.label.clone().unwrap_or_default(),
        first_failed_at: record.first_failed_at.to_rfc3339(),
        attempts: record.attempts.count(),
        failure_class: record
            .failure_class
            .map(|class| class.as_str().to_owned())
            .unwrap_or_default(),
        reason: record
            .reason
            .as_ref()
            .map(|reason| reason.as_str().to_owned())
            .unwrap_or_default(),
        status: record.lifecycle.as_str().to_owned(),
        resolved_by: resolution
            .map(|resolution| resolution.by.as_str().to_owned())
            .unwrap_or_default(),
        resolution_reason: resolution
            .map(|resolution| resolution.reason.as_str().to_owned())
            .unwrap_or_default(),
        resolved_at: resolution
            .map(|resolution| resolution.at.to_rfc3339())
            .unwrap_or_default(),
        waiting: record.blast_radius.waiting.clone(),
        waiting_count: record.blast_radius.count.unwrap_or_default(),
        residual: record.blast_radius.residual.clone().unwrap_or_default(),
        ..Default::default()
    }
}
```

### 10. Method StuckWorkSource::reclaim
`rust:crates/stuck-work/src/port.rs::StuckWorkSource::reclaim`  crates/stuck-work/src/port.rs:935-945  pkg:cargo/stuck-work  score 1.56  
`fn reclaim( &self, _id: &RecordId, ) -> StuckWorkFuture<'_, Result<StuckWorkRecord, StuckWorkError>>`  
Returns one abandoned record to its queue, counting an attempt (doc continues, 350 bytes)  
dispatches_to 3: `PolicyScanStuckWork::reclaim` (component/governance/policy/src/stuck_work.rs:561-595), `IngressDeadLetterStuckWork::reclaim` (component/platform/event-operations/src/stuck_work.rs:503-523), `tests::Fake::reclaim` (component/platform/supervision/src/pass.rs:336-345)  
uses_types 4: `StuckWorkError` (crates/stuck-work/src/port.rs:938), `StuckWorkRecord` (crates/stuck-work/src/port.rs:938), `StuckWorkFuture` (crates/stuck-work/src/port.rs:938) … 1 more  
crates/stuck-work/src/port.rs:935-945
```rust
    fn reclaim(
        &self,
        _id: &RecordId,
    ) -> StuckWorkFuture<'_, Result<StuckWorkRecord, StuckWorkError>> {
        Box::pin(async move {
            Err(StuckWorkError::NotApplicable {
                action: "reclaim",
                state: "a source that reports no abandoned work",
            })
        })
    }
```

### 11. Method StuckWorkListPage::narrowed
`rust:apps/causeway-server-web/src/views/platform/operations/stuck_work.rs::StuckWorkListPage::narrowed`  apps/causeway-server-web/src/views/platform/operations/stuck_work.rs:81-91  pkg:cargo/causeway-server-web  score 1.56  
`pub fn narrowed( mut self, subject: Option<String>, class: Option<String>, older_than_hours: Option<i64>, ) -> Self`  
The filters applied to what the sources returned.  
callers 1: `stuck_work` (apps/causeway-server/src/routes/pages/metadata_ops/stuck_work.rs:193)  
apps/causeway-server-web/src/views/platform/operations/stuck_work.rs:81-91
```rust
    pub fn narrowed(
        mut self,
        subject: Option<String>,
        class: Option<String>,
        older_than_hours: Option<i64>,
    ) -> Self {
        self.subject_filter = subject;
        self.class_filter = class;
        self.older_than_hours = older_than_hours;
        self
    }
```

### 12. Method StuckWorkCounts::in_review
`rust:apps/causeway-server/src/metadata_admin/stuck_work.rs::StuckWorkCounts::in_review`  apps/causeway-server/src/metadata_admin/stuck_work.rs:80-82  pkg:cargo/causeway-server  score 1.51  
`pub(crate) fn in_review(&self) -> u64`  
Work waiting on a person across every source that answered.  
callers 5: `tests::every_class_of_stuck_work_reaches_one_surface` (apps/causeway-server/src/metadata_admin/stuck_work.rs:531), `tests::an_empty_registry_lists_nothing_and_reports_nothing_unavailable` (apps/causeway-server/src/metadata_admin/stuck_work.rs:378), `tests::every_class_of_stuck_work_reaches_one_surface` (apps/causeway-server/src/metadata_admin/stuck_work.rs:533) … 2 more  
tests 2: `tests::every_class_of_stuck_work_reaches_one_surface` (apps/causeway-server/src/metadata_admin/stuck_work.rs:531), `tests::an_empty_registry_lists_nothing_and_reports_nothing_unavailable` (apps/causeway-server/src/metadata_admin/stuck_work.rs:378)  
apps/causeway-server/src/metadata_admin/stuck_work.rs:80-82
```rust
    pub(crate) fn in_review(&self) -> u64 {
        self.per_source.iter().map(|count| count.in_review).sum()
    }
```

### 13. File apps/causeway-server/src/routes/pages/metadata_ops/stuck_work.rs
`file:apps/causeway-server/src/routes/pages/metadata_ops/stuck_work.rs`  apps/causeway-server/src/routes/pages/metadata_ops/stuck_work.rs  pkg:cargo/causeway-server  score 1.51  via `stuck_work_record` <- CONTAINS (apps/causeway-server/src/routes/pages/metadata_ops/stuck_work.rs:216-256, 1.00)  
members 14  
designs 1: `docs/rfd/RFD-2n2tcfe2-operational-resilience-and-the-stuck-work-surface.md` (docs/rfd/RFD-2n2tcfe2-operational-resilience-and-the-stuck-work-surface.md:271)  
documents 2: `docs/plans/OPERATIONAL-RESILIENCE-WORKPLAN.md` (docs/plans/OPERATIONAL-RESILIENCE-WORKPLAN.md:81), `docs/DESIGN-LOG.md` (docs/DESIGN-LOG.md:105)  
imports 18: `cargo/serde` (apps/causeway-server/src/routes/pages/metadata_ops/stuck_work.rs:18), `StuckWorkRecordPage` (apps/causeway-server/src/routes/pages/metadata_ops/stuck_work.rs:15-17), `RecordId` (apps/causeway-server/src/routes/pages/metadata_ops/stuck_work.rs:19) … 15 more  

### 14. Struct StuckWorkPage
`rust:apps/causeway-server/src/metadata_admin/stuck_work.rs::StuckWorkPage`  apps/causeway-server/src/metadata_admin/stuck_work.rs:111-118  pkg:cargo/causeway-server  score 1.50  
`pub(crate) struct StuckWorkPage`  
One record with everything the detail page shows, including what this (doc continues, 107 bytes)  
used_by 1: `page` (apps/causeway-server/src/metadata_admin/stuck_work.rs:200)  
uses_types 2: `Correctable` (apps/causeway-server/src/metadata_admin/stuck_work.rs:115), `StuckWorkDetail` (apps/causeway-server/src/metadata_admin/stuck_work.rs:113)  
apps/causeway-server/src/metadata_admin/stuck_work.rs:111-118
```rust
pub(crate) struct StuckWorkPage {
    /// The record, its payload, and its failure history.
    pub(crate) detail: StuckWorkDetail,
    /// Whether this source takes a corrected payload back.
    pub(crate) corrections: Correctable,
    /// The seat that failing a record of this source requires.
    pub(crate) failing_seat: Option<String>,
}
```

### 15. Trait StuckWorkSource
`rust:crates/stuck-work/src/port.rs::StuckWorkSource`  crates/stuck-work/src/port.rs:843-954  pkg:cargo/stuck-work  score 1.42  via `StuckWorkSource::act` <- DEFINES (crates/stuck-work/src/port.rs:860-865, 1.00)  
`pub trait StuckWorkSource: Send + Sync + 'static`  
One subsystem's stuck work, read and acted on through one shape. (doc continues, 134 bytes)  
members 11  
implementations 9: `tests::a_source_that_shares_no_payload_still_has_a_detail::Bare#impl(StuckWorkSource)` (crates/stuck-work/src/port.rs:1120), `tests::Fake#impl(StuckWorkSource)` (component/platform/supervision/src/pass.rs:289), `tests::a_source_that_takes_no_correction_refuses_one_before_the_store::NoCorrections#impl(StuckWorkSource)` (apps/causeway-server/src/metadata_admin/stuck_work.rs:390) … 6 more  
importers 11: `apps/causeway-server/src/metadata_admin/handlers/caller_tests.rs` (apps/causeway-server/src/metadata_admin/handlers/caller_tests.rs:11-14), `component/platform/coherence/src/source.rs` (component/platform/coherence/src/source.rs:21-26), `component/metadata/core/src/stuck_work.rs` (component/metadata/core/src/stuck_work.rs:14-19) … 8 more  
used_by 30: `tests::a_source_that_takes_no_correction_refuses_one_before_the_store` (apps/causeway-server/src/metadata_admin/stuck_work.rs:434), `tests::bounded_work_waits_for_a_person_and_says_why` (component/platform/supervision/src/pass.rs:453), `tests::live_work_on_an_unfenced_source_is_left_alone` (component/platform/supervision/src/pass.rs:432) … 27 more  
crates/stuck-work/src/port.rs:843-922, cut
```rust
pub trait StuckWorkSource: Send + Sync + 'static {
    /// Which source this is.
    fn id(&self) -> SourceId;

    /// The records this source has, newest failure first.
    fn list(
        &self,
        query: &StuckWorkQuery,
    ) -> StuckWorkFuture<'_, Result<Vec<StuckWorkRecord>, StuckWorkError>>;

    /// One record.
    fn get(
        &self,
        id: &RecordId,
    ) -> StuckWorkFuture<'_, Result<Option<StuckWorkRecord>, StuckWorkError>>;

    /// Acts on one record, returning it as it now stands.
    fn act(
        &self,
        id: &RecordId,
        action: &StuckWorkAction,
        actor: &Actor,
    ) -> StuckWorkFuture<'_, Result<StuckWorkRecord, StuckWorkError>>;

    /// One record with what an operator needs before deciding: the
    /// payload and the failure history (determination 12).
    ///
    /// A source that shares neither is still a source; the default says
    /// so rather than leaving the surface to infer it from emptiness.
    fn detail(
        &self,
        id: &RecordId,
    ) -> StuckWorkFuture<'_, Result<Option<StuckWorkDetail>, StuckWorkError>> {
        let id = id.clone();
        Box::pin(async move {
            Ok(self.get(&id).await?.map(|record| {
                let note = record
                    .reason
                    .clone()
                    .map(|error| AttemptNote::new(record.attempts, error));
                StuckWorkDetail::new(
                    record,
                    Payload::absent("this source does not share payloads"),
                    FailureHistory::latest_only(note),
                )
            }))
        })
    }

    /// Whether this source takes a corrected payload back
    /// (determination 20).
    fn corrections(&self) -> Correctable {
        Correctable::No
    }

    /// The governing role a terminal action needs, when the source
    /// declares one (determination 7). `None` leaves it to Operations.
    fn failing_seat(&self) -> Option<&str> {
        None
    }

    /// How long this source keeps trying (determinations 5 and 17).
    ///
    /// There is no default. An unbounded retry loop is the defect this
    /// programme exists to retire, so a source that joins the surface
    /// says what its bounds are rather than inheriting a number someone
    /// else chose.
    fn bounds(&self) -> Bounds;

    /// Work whose claim has lapsed or run past its deadline
    /// (determination 16).
    ///
    /// The default is empty, and that is a real answer rather than a
    /// stub: a source whose queue re-arms a lapsed lease by itself has
    /// nothing for a sweep to reclaim, and a source that never claims
    /// anything has nothing to abandon. Saying so costs nothing and
    /// keeps the sweep from inventing work.
    fn abandoned(
        &self,
        _now: DateTime<Utc>,
```

### 16. Struct RecordId
`rust:crates/stuck-work/src/port.rs::RecordId`  crates/stuck-work/src/port.rs:65  pkg:cargo/stuck-work  score 1.36  via `StuckWorkSource::act` -> USES_TYPE (crates/stuck-work/src/port.rs:862, 1.00)  
`pub struct RecordId`  
A record's identity within its source, as the source spells it. (doc continues, 186 bytes)  
members 3  
importers 11: `component/platform/event-operations/src/stuck_work.rs` (component/platform/event-operations/src/stuck_work.rs:21-25), `apps/causeway-server/src/metadata_admin/stuck_work.rs` (apps/causeway-server/src/metadata_admin/stuck_work.rs:13-16), `database_tests` (component/metadata/core/src/maintenance.rs:475) … 8 more  
used_by 49: `MetadataQueueStuckWork::apply` (component/metadata/core/src/stuck_work.rs:153), `PolicyScanStuckWork::detail` (component/governance/policy/src/stuck_work.rs:438), `MetadataQueueStuckWork::read_detail` (component/metadata/core/src/stuck_work.rs:262) … 46 more  
crates/stuck-work/src/port.rs:65-65
```rust
pub struct RecordId(String);
```

### 17. TypeAlias StuckWorkFuture
`rust:crates/stuck-work/src/port.rs::StuckWorkFuture`  crates/stuck-work/src/port.rs:24  pkg:cargo/stuck-work  score 1.29  via `StuckWorkSource::act` -> USES_TYPE (crates/stuck-work/src/port.rs:865, 1.00)  
`pub type StuckWorkFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;`  
A future returned by a source, boxed so the port is object-safe.  
importers 8: `crates/stuck-work/src/lib.rs` (crates/stuck-work/src/lib.rs:30-35), `component/platform/event-operations/src/stuck_work.rs` (component/platform/event-operations/src/stuck_work.rs:21-25), `tests` (component/platform/supervision/src/pass.rs:238-241) … 5 more  
used_by 45: `Recording::get` (apps/causeway-server/src/metadata_admin/handlers/caller_tests.rs:58), `tests::a_source_that_takes_no_correction_refuses_one_before_the_store::NoCorrections::act` (apps/causeway-server/src/metadata_admin/stuck_work.rs:420), `StuckWorkSource::act` (crates/stuck-work/src/port.rs:865) … 42 more  
crates/stuck-work/src/port.rs:24-24
```rust
pub type StuckWorkFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;
```

### 18. Enum StuckWorkError
`rust:crates/stuck-work/src/port.rs::StuckWorkError`  crates/stuck-work/src/port.rs:609-629  pkg:cargo/stuck-work  score 1.21  via `StuckWorkSource::act` -> USES_TYPE (crates/stuck-work/src/port.rs:865, 1.00)  
`pub enum StuckWorkError`  
What a source could not do, reported rather than thrown away.  
members 3  
importers 8: `apps/causeway-server/src/metadata_admin/handlers/caller_tests.rs` (apps/causeway-server/src/metadata_admin/handlers/caller_tests.rs:11-14), `component/metadata/core/src/stuck_work.rs` (component/metadata/core/src/stuck_work.rs:14-19), `component/governance/policy/src/deferral.rs` (component/governance/policy/src/deferral.rs:13-17) … 5 more  
used_by 70: `Recording::act` (apps/causeway-server/src/metadata_admin/handlers/caller_tests.rs:67), `tests::a_source_that_takes_no_correction_refuses_one_before_the_store::NoCorrections::list` (apps/causeway-server/src/metadata_admin/stuck_work.rs:400), `MetadataQueueStuckWork::list` (component/metadata/core/src/stuck_work.rs:446) … 67 more  
crates/stuck-work/src/port.rs:609-629
```rust
pub enum StuckWorkError {
    /// The record is not there, or not there any more.
    #[error("{source_id} has no record {id}")]
    NotFound {
        /// The source asked.
        source_id: SourceId,
        /// The record asked for.
        id: RecordId,
    },
    /// The action does not apply to the record's state.
    #[error("{action} does not apply to a {state} record")]
    NotApplicable {
        /// What was asked.
        action: &'static str,
        /// What the record is.
        state: &'static str,
    },
    /// The source could not be read or written.
    #[error("{0}")]
    Unavailable(String),
}
```

### 19. Struct StuckWorkListPage
`rust:apps/causeway-server-web/src/views/platform/operations/stuck_work.rs::StuckWorkListPage`  apps/causeway-server-web/src/views/platform/operations/stuck_work.rs:26-47  pkg:cargo/causeway-server-web  score 1.19  via `StuckWorkListPage#impl` -> USES_TYPE (apps/causeway-server-web/src/views/platform/operations/stuck_work.rs:49-100, 1.00)  
`pub struct StuckWorkListPage`  
The list page.  
members 4  
importers 1: `apps/causeway-server/src/routes/pages/metadata_ops/stuck_work.rs` (apps/causeway-server/src/routes/pages/metadata_ops/stuck_work.rs:15-17)  
used_by 3: `stuck_work_list` (apps/causeway-server-web/src/views/platform/operations/stuck_work.rs:144), `filters` (apps/causeway-server-web/src/views/platform/operations/stuck_work.rs:249), `StuckWorkListPage#impl` (apps/causeway-server-web/src/views/platform/operations/stuck_work.rs:49-100)  
uses_types 2: `StuckWorkRecord` (apps/causeway-server-web/src/views/platform/operations/stuck_work.rs:28), `SourceCount` (apps/causeway-server-web/src/views/platform/operations/stuck_work.rs:30)  
apps/causeway-server-web/src/views/platform/operations/stuck_work.rs:26-47
```rust
pub struct StuckWorkListPage {
    /// The records, oldest failure first.
    pub records: Vec<StuckWorkRecord>,
    /// Per-source counts.
    pub counts: Vec<SourceCount>,
    /// Sources that could not be read, named rather than omitted.
    pub unavailable: Vec<String>,
    /// The source filter in force, when one is.
    pub source_filter: Option<String>,
    /// Whether ended records are being shown.
    pub include_ended: bool,
    /// A subject search, when one is in force.
    pub subject_filter: Option<String>,
    /// A failure-class filter: `transient`, `permanent` or `unjudged`.
    pub class_filter: Option<String>,
    /// Only records older than this many hours.
    pub older_than_hours: Option<i64>,
    /// What just happened, if anything.
    pub notice: Option<String>,
    /// What just failed to happen.
    pub error: Option<String>,
}
```

### 20. Package cargo/causeway-stuck-work-sweep
`pkg:cargo/causeway-stuck-work-sweep`  jobs/causeway-stuck-work-sweep/Cargo.toml:2  pkg:cargo/causeway-stuck-work-sweep  score 1.18  via `cargo/stuck-work` <- DEPENDS_ON (jobs/causeway-stuck-work-sweep/Cargo.toml:19, 1.00)  
`cargo/causeway-stuck-work-sweep`  
members 3  
dependencies 14: `cargo/event-operations-component` (jobs/causeway-stuck-work-sweep/Cargo.toml:26), `cargo/tokio` (jobs/causeway-stuck-work-sweep/Cargo.toml:1-37), `cargo/causeway-platform` (jobs/causeway-stuck-work-sweep/Cargo.toml:17) … 11 more  
designs 1: `docs/rfd/RFD-2n2tcfe2-operational-resilience-and-the-stuck-work-surface.md` (docs/rfd/RFD-2n2tcfe2-operational-resilience-and-the-stuck-work-surface.md:269)  
documents 1: `docs/plans/OPERATIONAL-RESILIENCE-WORKPLAN.md` (docs/plans/OPERATIONAL-RESILIENCE-WORKPLAN.md:133)  

### 21. Function stuck_work
`rust:apps/causeway-server/src/routes/pages/metadata_ops/stuck_work.rs::stuck_work`  apps/causeway-server/src/routes/pages/metadata_ops/stuck_work.rs:132-212  pkg:cargo/causeway-server  score 1.15  via `StuckWorkListPage::new` <- CALLS (apps/causeway-server/src/routes/pages/metadata_ops/stuck_work.rs:176, 1.00)  
`pub async fn stuck_work( State(state): State<AppState>, Query(params): Query<StuckWorkListQuery>, Extension(caller): Extension<BrowserCaller>, method: Method, uri: Uri, headers: HeaderMap, ) -> Response`  
Every source's open work, oldest failure first, for a caller the (doc continues, 135 bytes)  
calls 18: `render` (apps/causeway-server/src/routes/pages/metadata_ops/stuck_work.rs:200), `StuckWorkQuery::from_source` (apps/causeway-server/src/routes/pages/metadata_ops/stuck_work.rs:160), `stuck_work_list` (apps/causeway-server/src/routes/pages/metadata_ops/stuck_work.rs:207) … 15 more  
importers 1: `apps/causeway-server/src/routes/pages.rs` (apps/causeway-server/src/routes/pages.rs:53-58)  
referenced_by 2: `routes` (apps/causeway-server/src/routes/app/operations.rs:34), `stuck_work_record` (apps/causeway-server/src/metadata_admin/handlers.rs:14)  
uses_types 4: `BrowserCaller` (apps/causeway-server/src/routes/pages/metadata_ops/stuck_work.rs:135), `StuckWorkListQuery` (apps/causeway-server/src/routes/pages/metadata_ops/stuck_work.rs:134), `AppState` (apps/causeway-server/src/routes/pages/metadata_ops/stuck_work.rs:133) … 1 more  
apps/causeway-server/src/routes/pages/metadata_ops/stuck_work.rs:132-211, cut
```rust
pub async fn stuck_work(
    State(state): State<AppState>,
    Query(params): Query<StuckWorkListQuery>,
    Extension(caller): Extension<BrowserCaller>,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
) -> Response {
    if let Some(refusal) = refused_unless_admitted(
        &state,
        &caller,
        (&method, &uri, &headers),
        &list_breadcrumbs(),
    )
    .await
    {
        return refusal;
    }
    let include_ended = params
        .include_ended
        .as_deref()
        .is_some_and(|value| matches!(value, "true" | "on" | "1"));
    let source = params
        .source
        .as_deref()
        .map(str::trim)
        .filter(|source| !source.is_empty())
        .and_then(|source| SourceId::parse(source).ok());
    let query = StuckWorkQuery::from_source(source).including_ended(include_ended);

    let sources = state.stuck_work_sources();
    let listing = registry::list(sources, &query).await;
    let counted = registry::counts(sources).await;
    let mut unavailable = listing.unavailable;
    for reason in counted.unavailable {
        if !unavailable.contains(&reason) {
            unavailable.push(reason);
        }
    }
    let records = listing
        .records
        .into_iter()
        .filter(|record| admits(record, &params))
        .collect();
    let page = StuckWorkListPage::new(
        records,
        counted
            .per_source
            .into_iter()
            .map(|count| SourceCount {
                source: count.source,
                open: count.open,
                in_review: count.in_review,
            })
            .collect(),
        unavailable,
    )
    .filtered(
        params.source.clone().filter(|source| !source.is_empty()),
        include_ended,
    )
    .narrowed(
        params.subject.clone().filter(|value| !value.is_empty()),
        params.class.clone().filter(|value| !value.is_empty()),
        params.older_than_hours,
    )
    .with_feedback(params.notice, params.error);

    render(
        &state,
        "Stuck work",
        "workflows",
        list_breadcrumbs(),
        Vec::new(),
        "",
        causeway_server_web::views::platform::operations::stuck_work_list(&page),
        None,
    )
    .await
    .into_response()
```

### 22. Enum StuckWorkAction
`rust:crates/stuck-work/src/port.rs::StuckWorkAction`  crates/stuck-work/src/port.rs:551-576  pkg:cargo/stuck-work  score 1.14  via `StuckWorkSource::act` -> USES_TYPE (crates/stuck-work/src/port.rs:863, 1.00)  
`pub enum StuckWorkAction`  
What an operator asks a source to do with a record. (doc continues, 159 bytes)  
members 8  
importers 10: `crates/stuck-work/src/lib.rs` (crates/stuck-work/src/lib.rs:30-35), `component/platform/supervision/src/pass.rs` (component/platform/supervision/src/pass.rs:8), `apps/causeway-server/src/metadata_admin/stuck_work.rs` (apps/causeway-server/src/metadata_admin/stuck_work.rs:13-16) … 7 more  
used_by 17: `CorrectiveDeferrals::act` (component/governance/policy/src/deferral.rs:304), `CoherenceFindings::act` (component/platform/coherence/src/source.rs:286), `PolicyScanStuckWork::act` (component/governance/policy/src/stuck_work.rs:427) … 14 more  
crates/stuck-work/src/port.rs:551-576
```rust
pub enum StuckWorkAction {
    /// Put the work back on its queue. The outcome is the source's to
    /// report, so this does not claim success.
    Retry,
    /// A person has taken it.
    Acknowledge,
    /// Stop retrying; it waits for a person.
    Review {
        /// Why it is being held.
        cause: Reason,
    },
    /// It will not be done, and somebody with the seat said so.
    Fail {
        /// Why.
        reason: Reason,
    },
    /// The operator repaired what failed and asks for it again. The
    /// attempt count resets, because what is being retried is no longer
    /// what failed (determination 20).
    Correct {
        /// The repaired payload, in the shape the source published.
        payload: Payload,
        /// What was wrong with it, for whoever reads the record later.
        reason: Reason,
    },
}
```

### 23. Struct StuckWorkCounts
`rust:apps/causeway-server/src/metadata_admin/stuck_work.rs::StuckWorkCounts`  apps/causeway-server/src/metadata_admin/stuck_work.rs:66-71  pkg:cargo/causeway-server  score 1.14  via `StuckWorkCounts#impl` -> USES_TYPE (apps/causeway-server/src/metadata_admin/stuck_work.rs:73-83, 1.00)  
`pub(crate) struct StuckWorkCounts`  
The counts across the registry, and the sources that could not answer.  
members 2  
used_by 2: `counts` (apps/causeway-server/src/metadata_admin/stuck_work.rs:87), `StuckWorkCounts#impl` (apps/causeway-server/src/metadata_admin/stuck_work.rs:73-83)  
uses_types 1: `SourceCount` (apps/causeway-server/src/metadata_admin/stuck_work.rs:68)  
apps/causeway-server/src/metadata_admin/stuck_work.rs:66-71
```rust
pub(crate) struct StuckWorkCounts {
    /// Per source, in registration order.
    pub(crate) per_source: Vec<SourceCount>,
    /// Which sources failed, and why.
    pub(crate) unavailable: Vec<String>,
}
```

### 24. File apps-web/causeway-web/src/pages/data/stuck-work.ts
`file:apps-web/causeway-web/src/pages/data/stuck-work.ts`  apps-web/causeway-web/src/pages/data/stuck-work.ts  pkg:npm/@causeway/web  score 1.13  via `countsOf` <- CONTAINS (apps-web/causeway-web/src/pages/data/stuck-work.ts:111-121, 1.00)  
members 10  
calls 2: `list` (apps-web/causeway-web/src/pages/data/stuck-work.ts:35), `list` (apps-web/causeway-web/src/pages/data/stuck-work.ts:35)  
documents 1: `docs/DESIGN-LOG.md` (docs/DESIGN-LOG.md:2214)  
importers 4: `apps-web/causeway-web/test/app-slices-routing.test.ts` (apps-web/causeway-web/test/app-slices-routing.test.ts:19), `apps-web/causeway-web/src/pages/routes.tsx` (apps-web/causeway-web/src/pages/routes.tsx:52), `apps-web/causeway-web/src/pages/routes/StuckWork.tsx` (apps-web/causeway-web/src/pages/routes/StuckWork.tsx:9-15) … 1 more  
imports 8: `apps-web/causeway-web/src/pages/data/wire.ts` (apps-web/causeway-web/src/pages/data/stuck-work.ts:15), `present` (apps-web/causeway-web/src/pages/data/stuck-work.ts:15), `AccessToken` (apps-web/causeway-web/src/pages/data/stuck-work.ts:14) … 5 more  

### 25. Repository causeway
`repo:causeway`  score 1.12  via `cargo/stuck-work` <- CONTAINS (crates/stuck-work/Cargo.toml:2, 1.00)  
members 279  

### 26. File apps/causeway-server/src/metadata_admin/stuck_work.rs
`file:apps/causeway-server/src/metadata_admin/stuck_work.rs`  apps/causeway-server/src/metadata_admin/stuck_work.rs  pkg:cargo/causeway-server  score 1.10  via `StuckWorkPage` <- CONTAINS (apps/causeway-server/src/metadata_admin/stuck_work.rs:111-118, 1.00)  
members 16  
documents 2: `docs/plans/OPERATIONAL-RESILIENCE-WORKPLAN.md` (docs/plans/OPERATIONAL-RESILIENCE-WORKPLAN.md:57), `docs/DESIGN-LOG.md` (docs/DESIGN-LOG.md:96)  
imports 13: `RecordId` (apps/causeway-server/src/metadata_admin/stuck_work.rs:13-16), `Correctable::Payload` (apps/causeway-server/src/metadata_admin/stuck_work.rs:13-16), `StuckWorkAction` (apps/causeway-server/src/metadata_admin/stuck_work.rs:13-16) … 10 more  

### 27. Impl StuckWorkListPage#impl
`rust:apps/causeway-server-web/src/views/platform/operations/stuck_work.rs::StuckWorkListPage#impl`  apps/causeway-server-web/src/views/platform/operations/stuck_work.rs:49-100  pkg:cargo/causeway-server-web  score 1.01  via `StuckWorkListPage::new` <- DEFINES (apps/causeway-server-web/src/views/platform/operations/stuck_work.rs:52-69, 1.00)  
`impl StuckWorkListPage`  
members 4  
uses_types 1: `StuckWorkListPage` (apps/causeway-server-web/src/views/platform/operations/stuck_work.rs:49-100)  

### 28. Struct StuckWorkRecordPage
`rust:apps/causeway-server-web/src/views/platform/operations/stuck_work.rs::StuckWorkRecordPage`  apps/causeway-server-web/src/views/platform/operations/stuck_work.rs:103-114  pkg:cargo/causeway-server-web  score 0.98  via `stuck_work_record` -> USES_TYPE (apps/causeway-server-web/src/views/platform/operations/stuck_work.rs:176, 1.00)  
`pub struct StuckWorkRecordPage`  
The detail page for one record.  
members 2  
importers 1: `apps/causeway-server/src/routes/pages/metadata_ops/stuck_work.rs` (apps/causeway-server/src/routes/pages/metadata_ops/stuck_work.rs:15-17)  
used_by 4: `failure_history` (apps/causeway-server-web/src/views/platform/operations/stuck_work.rs:370), `stuck_work_record` (apps/causeway-server-web/src/views/platform/operations/stuck_work.rs:176), `StuckWorkRecordPage#impl` (apps/causeway-server-web/src/views/platform/operations/stuck_work.rs:116-140) … 1 more  
uses_types 2: `Correctable` (apps/causeway-server-web/src/views/platform/operations/stuck_work.rs:107), `StuckWorkDetail` (apps/causeway-server-web/src/views/platform/operations/stuck_work.rs:105)  
apps/causeway-server-web/src/views/platform/operations/stuck_work.rs:103-114
```rust
pub struct StuckWorkRecordPage {
    /// The record, its payload, and its failure history.
    pub detail: StuckWorkDetail,
    /// Whether this source takes a corrected payload back.
    pub corrections: Correctable,
    /// The governing seat failing this record requires.
    pub failing_seat: Option<String>,
    /// What just happened, if anything.
    pub notice: Option<String>,
    /// What just failed to happen.
    pub error: Option<String>,
}
```

### 29. Package cargo/causeway-server
`pkg:cargo/causeway-server`  apps/causeway-server/Cargo.toml:2  pkg:cargo/causeway-server  score 0.98  via `cargo/stuck-work` <- DEPENDS_ON (apps/causeway-server/Cargo.toml:63, 1.00)  
`cargo/causeway-server`  
members 296  
dependencies 126: `cargo/object_store` (apps/causeway-server/Cargo.toml:140), `cargo/clap` (apps/causeway-server/Cargo.toml:1-225), `cargo/temp-env` (apps/causeway-server/Cargo.toml:190) … 123 more  
designs 4: `docs/adrs/ADR-nbmt040e-cross-slice-read-boundaries.md` (docs/adrs/ADR-nbmt040e-cross-slice-read-boundaries.md:19), `docs/rfd/RFD-fpg4tnq2-ephemeral-infrastructure-for-ci-integration-tests.md` (docs/rfd/RFD-fpg4tnq2-ephemeral-infrastructure-for-ci-integration-tests.md:322), `docs/rfd/RFD-c4jbh7qa-typescript-and-node-web-tier-over-the-rust-core.md` (docs/rfd/RFD-c4jbh7qa-typescript-and-node-web-tier-over-the-rust-core.md:14) … 1 more  
documents 7: `docs/archive/requirements/REQ-011-rust-binary-classification-and-target-workspace-layout.md` (docs/archive/requirements/REQ-011-rust-binary-classification-and-target-workspace-layout.md:78), `docs/plans/SERVICE-BOUNDARIES-WORKPLAN.md` (docs/plans/SERVICE-BOUNDARIES-WORKPLAN.md:43), `docs/plans/RESTATE-REMEDIATION-WORKPLAN.md` (docs/plans/RESTATE-REMEDIATION-WORKPLAN.md:553) … 4 more  
importers 1: `apps/causeway-server/src/main.rs` (apps/causeway-server/src/main.rs:4)  

### 30. Document Operational resilience workplan
`doc:docs/plans/OPERATIONAL-RESILIENCE-WORKPLAN.md`  docs/plans/OPERATIONAL-RESILIENCE-WORKPLAN.md:1-208  pkg:npm/causeway-monorepo  score 0.95  via `cargo/stuck-work` -> DOCUMENTED_BY (docs/plans/OPERATIONAL-RESILIENCE-WORKPLAN.md:34, 1.00)  
`Operational resilience workplan`  
This plan implements [RFD-2n2tcfe2](../rfd/RFD-2n2tcfe2-operational-resilience-and-the-stuck-work-surface.md).  
headings: Governing decision
Scope
Dependencies
Ordered work
Slice 0. The vocabulary, proved in the queue that already works
Slice 1. The port and the aggregate read
Slice 2. The surface in metadata-admin
Slice 3. Bounds and classification in the producing subsystems
Slice 4. The supervisory sweep
Slice 5. Coherence as a capability of the system
Slice 6. Closure
The programme is closed  
documents_for 23: `apps/causeway-server/src/routes/pages/metadata_ops/stuck_work.rs` (docs/plans/OPERATIONAL-RESILIENCE-WORKPLAN.md:81), `cargo/stuck-work` (docs/plans/OPERATIONAL-RESILIENCE-WORKPLAN.md:34), `cargo/causeway-stuck-work-sweep` (docs/plans/OPERATIONAL-RESILIENCE-WORKPLAN.md:133) … 20 more  

### 31. Class PortListStuckWorkRequest
`python:services-py/relay-grpc/gen/causeway/metadata/v2/admin_pb2.pyi::PortListStuckWorkRequest`  services-py/relay-grpc/gen/causeway/metadata/v2/admin_pb2.pyi:600-610  pkg:pypi/python-grpc  score 0.90  
`class PortListStuckWorkRequest(_message.Message)`  
members 1  

### 32. Method PortListStuckWorkRequest::__init__
`python:services-py/relay-grpc/gen/causeway/metadata/v2/admin_pb2.pyi::PortListStuckWorkRequest::__init__`  services-py/relay-grpc/gen/causeway/metadata/v2/admin_pb2.pyi:610  pkg:pypi/python-grpc  score 0.90  
`def __init__(self, source: _Optional[str] = ..., subject_kind: _Optional[str] = ..., include_ended: bool = ..., limit: _Optional[int] = ...) -> None`  
services-py/relay-grpc/gen/causeway/metadata/v2/admin_pb2.pyi:610-610
```python
    def __init__(self, source: _Optional[str] = ..., subject_kind: _Optional[str] = ..., include_ended: bool = ..., limit: _Optional[int] = ...) -> None: ...
```

### 33. Package cargo/chrono
`pkg:cargo/chrono`  pkg:cargo/chrono  score 0.90  via `cargo/stuck-work` -> DEPENDS_ON (crates/stuck-work/Cargo.toml:1-20, 1.00)  
dependents 53: `cargo/workflow-read-model` (crates/workflow-read-model/Cargo.toml:10), `cargo/dbt-demo-wasm` (ffi/wasm-dbt-demo/crates/dbt-demo-wasm/Cargo.toml:1-31), `cargo/causeway-metadata-admin-host` (apps/causeway-metadata-admin-host/Cargo.toml:1-59) … 50 more  
importers 225: `write_reason_tests` (component/catalog/core/src/publication.rs:859), `component/orchestration/workflows/src/tasks.rs` (component/orchestration/workflows/src/tasks.rs:24), `crates/causeway-metadata-ops/src/rights_drafts.rs` (crates/causeway-metadata-ops/src/rights_drafts.rs:16) … 222 more  

### 34. Package cargo/serde
`pkg:cargo/serde`  pkg:cargo/serde  score 0.90  via `cargo/stuck-work` -> DEPENDS_ON (crates/stuck-work/Cargo.toml:1-20, 1.00)  
dependents 138: `cargo/people-component` (component/people/core/Cargo.toml:1-27), `cargo/causeway-policy-bundle` (crates/causeway-policy-bundle/Cargo.toml:1-24), `cargo/conversations-component` (component/collaboration/conversations/Cargo.toml:1-49) … 135 more  
importers 378: `tools/prism-lsp/src/custom.rs` (tools/prism-lsp/src/custom.rs:10), `tests` (crates/drive-core/src/lib.rs:2466), `crates/ingest-core/src/error.rs` (crates/ingest-core/src/error.rs:30) … 375 more  

### 35. Design Operational resilience: stuck work, poison records, and the operator's surface
`doc:docs/rfd/RFD-2n2tcfe2-operational-resilience-and-the-stuck-work-surface.md`  docs/rfd/RFD-2n2tcfe2-operational-resilience-and-the-stuck-work-surface.md:1-296  pkg:npm/causeway-monorepo  score 0.88  via `cargo/stuck-work` -> DESIGNED_BY (docs/rfd/RFD-2n2tcfe2-operational-resilience-and-the-stuck-work-surface.md:250, 1.00)  
`Operational resilience: stuck work, poison records, and the operator's surface`  
Causeway will treat work that cannot finish as first-class state rather than as an exception that each subsystem handles privately. Every asynchronous path records why it could not finish, in a shared shape; every retry loop is bounded and moves into review instead of looping; a hand-off to a service that is down defers rather than failing the work that already succeeded; and Platform Administrators and Operations get one surface — in metadata-admin, behind its own authentication boundary and audit — that lists stuck work across every subsystem, shows what depends on it, and offers retry, review, and fail as recorded decisions. Broken metadata is repaired there first; a repair that carries a governance consequence is surfaced for governance afterwards, and never the other way round. Records close on a reported success rather than on inferred quiet, and the coherence checks that find broken records move out of the build and into the system that owns the data.  
headings: 1. What "stuck" means
2. One store or many
3. A shared vocabulary
4. Bounds, and what happens at the bound
5. Who may do what
6. The surface
7. Health, not just lists
8. Work that goes silent
One central failure queue
An operator console in the product app
Alerting instead of a surface
Automatic failure after N attempts
Fail the producer when the consumer is down  
designs_for 29: `component/governance/policy/migrations/0002_scan_queue_bounds.sql` (docs/rfd/RFD-2n2tcfe2-operational-resilience-and-the-stuck-work-surface.md:259), `component/platform/coherence/src/source.rs` (docs/rfd/RFD-2n2tcfe2-operational-resilience-and-the-stuck-work-surface.md:266), `component/platform/event-operations/src/stuck_work.rs` (docs/rfd/RFD-2n2tcfe2-operational-resilience-and-the-stuck-work-surface.md:264) … 26 more  

### 36. Impl StuckWorkCounts#impl
`rust:apps/causeway-server/src/metadata_admin/stuck_work.rs::StuckWorkCounts#impl`  apps/causeway-server/src/metadata_admin/stuck_work.rs:73-83  pkg:cargo/causeway-server  score 0.88  via `StuckWorkCounts::in_review` <- DEFINES (apps/causeway-server/src/metadata_admin/stuck_work.rs:80-82, 1.00)  
`impl StuckWorkCounts`  
members 2  
uses_types 1: `StuckWorkCounts` (apps/causeway-server/src/metadata_admin/stuck_work.rs:73-83)  

### 37. File crates/stuck-work/src/port.rs
`file:crates/stuck-work/src/port.rs`  crates/stuck-work/src/port.rs  pkg:cargo/stuck-work  score 0.87  via `StuckWorkSource` <- CONTAINS (crates/stuck-work/src/port.rs:843-954, 1.00)  
members 39  
documents 1: `docs/plans/OPERATIONAL-RESILIENCE-WORKPLAN.md` (docs/plans/OPERATIONAL-RESILIENCE-WORKPLAN.md:57)  
imports 8: `cargo/serde` (crates/stuck-work/src/port.rs:19), `Actor` (crates/stuck-work/src/port.rs:21), `FailureClass` (crates/stuck-work/src/port.rs:21) … 5 more  

### 38. Struct StuckWorkDetail
`rust:crates/stuck-work/src/port.rs::StuckWorkDetail`  crates/stuck-work/src/port.rs:506-513  pkg:cargo/stuck-work  score 0.87  via `StuckWorkPage` -> USES_TYPE (apps/causeway-server/src/metadata_admin/stuck_work.rs:113, 1.00)  
`pub struct StuckWorkDetail`  
A record with what an operator needs before deciding: what the work (doc continues, 96 bytes)  
members 1  
importers 7: `crates/stuck-work/src/lib.rs` (crates/stuck-work/src/lib.rs:30-35), `component/platform/event-operations/src/stuck_work.rs` (component/platform/event-operations/src/stuck_work.rs:21-25), `component/governance/policy/src/stuck_work.rs` (component/governance/policy/src/stuck_work.rs:13-18) … 4 more  
used_by 10: `CoherenceFindings::detail` (component/platform/coherence/src/source.rs:375), `MetadataQueueStuckWork::detail` (component/metadata/core/src/stuck_work.rs:477), `StuckWorkRecordPage::new` (apps/causeway-server-web/src/views/platform/operations/stuck_work.rs:120) … 7 more  
uses_types 3: `FailureHistory` (crates/stuck-work/src/port.rs:512), `Payload` (crates/stuck-work/src/port.rs:510), `StuckWorkRecord` (crates/stuck-work/src/port.rs:508)  

### 39. Struct Actor
`rust:crates/stuck-work/src/lib.rs::Actor`  crates/stuck-work/src/lib.rs:52  pkg:cargo/stuck-work  score 0.85  via `StuckWorkSource::act` -> USES_TYPE (crates/stuck-work/src/port.rs:864, 1.00)  
`pub struct Actor`  
Who took an action: a person, a seat, or a named process. (doc continues, 264 bytes)  
members 3  
importers 10: `component/platform/event-operations/src/stuck_work.rs` (component/platform/event-operations/src/stuck_work.rs:21-25), `component/platform/supervision/src/pass.rs` (component/platform/supervision/src/pass.rs:8), `database_tests` (component/metadata/core/src/maintenance.rs:475) … 7 more  
used_by 24: `CorrectiveDeferrals::act` (component/governance/policy/src/deferral.rs:305), `IngressDeadLetterStuckWork::apply` (component/platform/event-operations/src/stuck_work.rs:148), `tests::a_source_that_takes_no_correction_refuses_one_before_the_store::NoCorrections::act` (apps/causeway-server/src/metadata_admin/stuck_work.rs:419) … 21 more  
uses_types 2: `Actor` (crates/stuck-work/src/lib.rs:52), `Actor` (crates/stuck-work/src/lib.rs:52)  
crates/stuck-work/src/lib.rs:52-52
```rust
pub struct Actor(String);
```

### 40. Package cargo/tokio
`pkg:cargo/tokio`  pkg:cargo/tokio  score 0.84  via `cargo/stuck-work` -> DEPENDS_ON (crates/stuck-work/Cargo.toml:20, 1.00)  
dependents 111: `cargo/cw-host` (bootstrap/cw-host/Cargo.toml:28), `cargo/causeway-capability` (crates/causeway-capability/Cargo.toml:16), `cargo/catalog-query-understanding` (component/discovery/search/query-understanding/Cargo.toml:20) … 108 more  
importers 92: `dev_dependencies` (crates/ingest-corpus/src/lib.rs:41), `crates/causeway-metadata-ops/src/materializer.rs` (crates/causeway-metadata-ops/src/materializer.rs:16), `tests` (component/discovery/search-feedback/src/lib.rs:877) … 89 more  

### 41. Struct StuckWorkRecordQuery
`rust:apps/causeway-server/src/routes/pages/metadata_ops/stuck_work.rs::StuckWorkRecordQuery`  apps/causeway-server/src/routes/pages/metadata_ops/stuck_work.rs:83-88  pkg:cargo/causeway-server  score 0.84  via `stuck_work_record` -> USES_TYPE (apps/causeway-server/src/routes/pages/metadata_ops/stuck_work.rs:219, 1.00)  
`pub(crate) struct StuckWorkRecordQuery`  
The feedback one record's page was redirected back with.  
used_by 1: `stuck_work_record` (apps/causeway-server/src/routes/pages/metadata_ops/stuck_work.rs:219)  

### 42. Test the stuck-work filters, as the shell's `admits`::counts open and in-review work per source
`typescript:apps-web/causeway-web/test/app-operations.test.ts::the stuck-work filters, as the shell's `admits`::counts open and in-review work per source`  apps-web/causeway-web/test/app-operations.test.ts:194-205  pkg:npm/@causeway/web  score 0.83  via `countsOf` -> TESTED_BY (apps-web/causeway-web/test/app-operations.test.ts:196, 1.00)  
`it("counts open and in-review work per source", () => { expect( countsOf([ record({ source: "a" }), record({ source: "a", status: "in_review" }), record({ source: "b" }), ]), ).toEqual([ { source: "a", open: 2, inReview: 1 }, { source: "b", open: 1, inReview: 0 }, ]); })`  
calls 4: `the stuck-work filters, as the shell's `admits`::record` (apps-web/causeway-web/test/app-operations.test.ts:198), `the stuck-work filters, as the shell's `admits`::record` (apps-web/causeway-web/test/app-operations.test.ts:199), `the stuck-work filters, as the shell's `admits`::record` (apps-web/causeway-web/test/app-operations.test.ts:197) … 1 more  
tests_for 2: `the stuck-work filters, as the shell's `admits`::record` (apps-web/causeway-web/test/app-operations.test.ts:197), `countsOf` (apps-web/causeway-web/test/app-operations.test.ts:196)  

### 43. Function stuckWork
`typescript:apps-web/causeway-web/src/pages/data/stuck-work.ts::stuckWork`  apps-web/causeway-web/src/pages/data/stuck-work.ts:128-152  pkg:npm/@causeway/web  score 0.83  via `countsOf` <- CALLS (apps-web/causeway-web/src/pages/data/stuck-work.ts:148, 1.00)  
`function stuckWork( rpc: ShellRpc, bearer: AccessToken, filter: StuckWorkFilter, now: Date, ): Effect.Effect<StuckWorkList, ShellRpcError>`  
The page's list, as `filter` narrows it. The twin is asked once with the port's filters (source, ended) and, when those narrow anything, once more with none for the counts, as the shell reads every source for them.  
members 1  
callers 1: `createPages` (apps-web/causeway-web/src/pages/routes.tsx:567)  
calls 5: `stuckWork::read` (apps-web/causeway-web/src/pages/data/stuck-work.ts:143), `countsOf` (apps-web/causeway-web/src/pages/data/stuck-work.ts:148), `stuckWork::read` (apps-web/causeway-web/src/pages/data/stuck-work.ts:138) … 2 more  
importers 1: `apps-web/causeway-web/src/pages/routes.tsx` (apps-web/causeway-web/src/pages/routes.tsx:52)  
uses_types 3: `ShellRpc` (apps-web/causeway-web/src/pages/data/stuck-work.ts:129), `StuckWorkFilter` (apps-web/causeway-web/src/pages/data/stuck-work.ts:131), `AccessToken` (apps-web/causeway-web/src/pages/data/stuck-work.ts:130)  

### 44. Class PortListStuckWorkResponse
`python:services-py/relay-grpc/gen/causeway/metadata/v2/admin_pb2.pyi::PortListStuckWorkResponse`  services-py/relay-grpc/gen/causeway/metadata/v2/admin_pb2.pyi:612-618  pkg:pypi/python-grpc  score 0.80  
`class PortListStuckWorkResponse(_message.Message)`  
members 1  

### 45. TypeAlias PortListStuckWorkRequest
`typescript:services-node/relay-grpc/gen/causeway/metadata/v2/admin_pb.ts::PortListStuckWorkRequest`  services-node/relay-grpc/gen/causeway/metadata/v2/admin_pb.ts:1441-1463  pkg:npm/ts-grpc  score 0.80  
`type PortListStuckWorkRequest = Message<"causeway.metadata.v2.PortListStuckWorkRequest"> & { /** * Empty lists every registered source. * * @generated from field: string source = 1; */ source: string; /** * @generated from field: string subject_kind = 2; */ subjectKind: string; /** * @generated from…`  
@generated from message causeway.metadata.v2.PortListStuckWorkRequest  

### 46. TypeAlias PortListStuckWorkResponse
`typescript:services-node/relay-grpc/gen/causeway/metadata/v2/admin_pb.ts::PortListStuckWorkResponse`  services-node/relay-grpc/gen/causeway/metadata/v2/admin_pb.ts:1475-1488  pkg:npm/ts-grpc  score 0.80  
`type PortListStuckWorkResponse = Message<"causeway.metadata.v2.PortListStuckWorkResponse"> & { /** * @generated from field: repeated causeway.metadata.v2.StuckWorkRecord records = 1; */ records: StuckWorkRecord[]; /** * Sources that could not be read, named rather than omitted: one * unavailable sou…`  
@generated from message causeway.metadata.v2.PortListStuckWorkResponse  

### 47. Struct SourceCount
`rust:apps/causeway-server-web/src/views/platform/operations/stuck_work.rs::SourceCount`  apps/causeway-server-web/src/views/platform/operations/stuck_work.rs:16-23  pkg:cargo/causeway-server-web  score 0.79  via `StuckWorkListPage::new` -> USES_TYPE (apps/causeway-server-web/src/views/platform/operations/stuck_work.rs:54, 1.00)  
`pub struct SourceCount`  
One source's count, as the header reports it.  
importers 1: `apps/causeway-server/src/routes/pages/metadata_ops/stuck_work.rs` (apps/causeway-server/src/routes/pages/metadata_ops/stuck_work.rs:15-17)  
used_by 5: `counts_header` (apps/causeway-server-web/src/views/platform/operations/stuck_work.rs:208), `SourceCount` (apps/causeway-server/src/metadata_admin/stuck_work.rs:56), `stuck_work` (apps/causeway-server/src/routes/pages/metadata_ops/stuck_work.rs:181) … 2 more  
uses_types 1: `SourceCount` (apps/causeway-server-web/src/views/platform/operations/stuck_work.rs:16)  

### 48. Test tests::every_class_of_stuck_work_reaches_one_surface
`rust:apps/causeway-server/src/metadata_admin/stuck_work.rs::tests::every_class_of_stuck_work_reaches_one_surface`  apps/causeway-server/src/metadata_admin/stuck_work.rs:480-536  pkg:cargo/causeway-server  score 0.79  via `StuckWorkCounts::in_review` -> TESTED_BY (apps/causeway-server/src/metadata_admin/stuck_work.rs:531, 1.00)  
`async fn every_class_of_stuck_work_reaches_one_surface()`  
One workspace, one record of every class, read through the one (doc continues, 553 bytes)  
calls 16: `in_workspace` (apps/causeway-server/src/metadata_admin/stuck_work.rs:481), `tests::seed_one_of_each` (apps/causeway-server/src/metadata_admin/stuck_work.rs:488), `StuckWorkCounts::in_review` (apps/causeway-server/src/metadata_admin/stuck_work.rs:531) … 13 more  
tests_for 14: `counts` (apps/causeway-server/src/metadata_admin/stuck_work.rs:528), `StuckWorkCounts::in_review` (apps/causeway-server/src/metadata_admin/stuck_work.rs:531), `tests::every_source` (apps/causeway-server/src/metadata_admin/stuck_work.rs:490) … 11 more  

### 49. Package cargo/thiserror
`pkg:cargo/thiserror`  pkg:cargo/thiserror  score 0.77  via `cargo/stuck-work` -> DEPENDS_ON (crates/stuck-work/Cargo.toml:1-20, 1.00)  
dependents 109: `cargo/causeway-oidc-keys` (torrentfx/crates/causeway-oidc-keys/Cargo.toml:21), `cargo/people-rpc` (crates/rpc/people-rpc/Cargo.toml:15), `cargo/metadata-rpc` (crates/rpc/metadata-rpc/Cargo.toml:15) … 106 more  
importers 60: `crates/causeway-testkit/tests/volumes.rs` (crates/causeway-testkit/tests/volumes.rs:35), `crates/causeway-testkit/tests/object_store.rs` (crates/causeway-testkit/tests/object_store.rs:29), `crates/causeway-runtime/src/resource.rs` (crates/causeway-runtime/src/resource.rs:23) … 57 more  

### 50. Function actions
`rust:apps/causeway-server-web/src/views/platform/operations/stuck_work.rs::actions`  apps/causeway-server-web/src/views/platform/operations/stuck_work.rs:426-497  pkg:cargo/causeway-server-web  score 0.75  via `stuck_work_record` -> CALLS (apps/causeway-server-web/src/views/platform/operations/stuck_work.rs:202, 1.00)  
`fn actions(page: &StuckWorkRecordPage) -> Markup`  
callers 3: `stuck_work_record` (apps/causeway-server-web/src/views/platform/operations/stuck_work.rs:202), `tests::the_research_examples_read_and_the_revision_adds_one_permission` (crates/rights-odrl/src/lib.rs:536), `tests::the_research_examples_read_and_the_revision_adds_one_permission` (crates/rights-odrl/src/lib.rs:536)  
calls 2: `Correctable::is_offered` (apps/causeway-server-web/src/views/platform/operations/stuck_work.rs:455), `Payload::is_correctable` (apps/causeway-server-web/src/views/platform/operations/stuck_work.rs:455)  
references 1: `Payload::Json` (apps/causeway-server-web/src/views/platform/operations/stuck_work.rs:463)  
tests 1: `tests::the_research_examples_read_and_the_revision_adds_one_permission` (crates/rights-odrl/src/lib.rs:536)  
uses_types 1: `StuckWorkRecordPage` (apps/causeway-server-web/src/views/platform/operations/stuck_work.rs:426)  

### 51. Enum Correctable
`rust:crates/stuck-work/src/port.rs::Correctable`  crates/stuck-work/src/port.rs:530-535  pkg:cargo/stuck-work  score 0.75  via `StuckWorkPage` -> USES_TYPE (apps/causeway-server/src/metadata_admin/stuck_work.rs:115, 1.00)  
`pub enum Correctable`  
Whether a source can take a corrected payload back (determination 20).  
members 3  
importers 6: `crates/stuck-work/src/lib.rs` (crates/stuck-work/src/lib.rs:30-35), `apps/causeway-server/src/metadata_admin/stuck_work.rs` (apps/causeway-server/src/metadata_admin/stuck_work.rs:13-16), `apps/causeway-server-web/src/views/platform/operations/stuck_work.rs` (apps/causeway-server-web/src/views/platform/operations/stuck_work.rs:11) … 3 more  
used_by 8: `CoherenceFindings::corrections` (component/platform/coherence/src/source.rs:408), `Correctable#impl` (crates/stuck-work/src/port.rs:537-543), `StuckWorkPage` (apps/causeway-server/src/metadata_admin/stuck_work.rs:115) … 5 more  

### 52. Function failure_history
`rust:apps/causeway-server-web/src/views/platform/operations/stuck_work.rs::failure_history`  apps/causeway-server-web/src/views/platform/operations/stuck_work.rs:370-402  pkg:cargo/causeway-server-web  score 0.74  via `stuck_work_record` -> CALLS (apps/causeway-server-web/src/views/platform/operations/stuck_work.rs:190, 1.00)  
`fn failure_history(page: &StuckWorkRecordPage) -> Markup`  
callers 1: `stuck_work_record` (apps/causeway-server-web/src/views/platform/operations/stuck_work.rs:190)  
calls 3: `FailureHistory::notes` (apps/causeway-server-web/src/views/platform/operations/stuck_work.rs:381), `FailureHistory::notes` (apps/causeway-server-web/src/views/platform/operations/stuck_work.rs:377), `FailureHistory::completeness` (apps/causeway-server-web/src/views/platform/operations/stuck_work.rs:395)  
references 1: `Completeness::LatestOnly` (apps/causeway-server-web/src/views/platform/operations/stuck_work.rs:395)  
uses_types 1: `StuckWorkRecordPage` (apps/causeway-server-web/src/views/platform/operations/stuck_work.rs:370)  

### 53. Function record_breadcrumbs
`rust:apps/causeway-server/src/routes/pages/metadata_ops/stuck_work.rs::record_breadcrumbs`  apps/causeway-server/src/routes/pages/metadata_ops/stuck_work.rs:122-128  pkg:cargo/causeway-server  score 0.72  via `stuck_work_record` -> CALLS (apps/causeway-server/src/routes/pages/metadata_ops/stuck_work.rs:248, 1.00)  
`fn record_breadcrumbs() -> Vec<Breadcrumb>`  
callers 2: `stuck_work_record` (apps/causeway-server/src/routes/pages/metadata_ops/stuck_work.rs:229), `stuck_work_record` (apps/causeway-server/src/routes/pages/metadata_ops/stuck_work.rs:248)  
calls 3: `Breadcrumb::current` (apps/causeway-server/src/routes/pages/metadata_ops/stuck_work.rs:126), `Breadcrumb::link` (apps/causeway-server/src/routes/pages/metadata_ops/stuck_work.rs:124), `Breadcrumb::link` (apps/causeway-server/src/routes/pages/metadata_ops/stuck_work.rs:125)  
uses_types 1: `Breadcrumb` (apps/causeway-server/src/routes/pages/metadata_ops/stuck_work.rs:122)  

### 54. Function page
`rust:apps/causeway-server/src/metadata_admin/stuck_work.rs::page`  apps/causeway-server/src/metadata_admin/stuck_work.rs:196-214  pkg:cargo/causeway-server  score 0.72  via `StuckWorkPage` <- USES_TYPE (apps/causeway-server/src/metadata_admin/stuck_work.rs:200, 1.00)  
`pub(crate) async fn page( sources: &[Arc<dyn StuckWorkSource>], source_id: &SourceId, id: &RecordId, ) -> Result<StuckWorkPage, ActionError>`  
One record of one source, with what that source will accept.  
callers 1: `tests::every_class_of_stuck_work_reaches_one_surface` (apps/causeway-server/src/metadata_admin/stuck_work.rs:520)  
calls 6: `failing_seat` (apps/causeway-server/src/metadata_admin/stuck_work.rs:212), `source_of` (apps/causeway-server/src/metadata_admin/stuck_work.rs:201), `PolicyScanStuckWork::corrections` (apps/causeway-server/src/metadata_admin/stuck_work.rs:211) … 3 more  
tests 1: `tests::every_class_of_stuck_work_reaches_one_surface` (apps/causeway-server/src/metadata_admin/stuck_work.rs:520)  
uses_types 5: `SourceId` (apps/causeway-server/src/metadata_admin/stuck_work.rs:198), `StuckWorkSource` (apps/causeway-server/src/metadata_admin/stuck_work.rs:197), `ActionError` (apps/causeway-server/src/metadata_admin/stuck_work.rs:200) … 2 more  

### 55. Method tests::a_source_that_shares_no_payload_still_has_a_detail::Bare::act
`rust:crates/stuck-work/src/port.rs::tests::a_source_that_shares_no_payload_still_has_a_detail::Bare::act`  crates/stuck-work/src/port.rs:1139-1146  pkg:cargo/stuck-work  score 0.70  via `StuckWorkSource::act` -> DISPATCHES_TO (crates/stuck-work/src/port.rs:1139-1146, 1.00)  
`fn act( &self, _id: &RecordId, _action: &StuckWorkAction, _actor: &Actor, ) -> StuckWorkFuture<'_, Result<StuckWorkRecord, StuckWorkError>>`  
dispatched_from 1: `StuckWorkSource::act` (crates/stuck-work/src/port.rs:1139-1146)  
uses_types 6: `StuckWorkRecord` (crates/stuck-work/src/port.rs:1144), `Actor` (crates/stuck-work/src/port.rs:1143), `StuckWorkError` (crates/stuck-work/src/port.rs:1144) … 3 more  

### 56. Method tests::a_source_that_takes_no_correction_refuses_one_before_the_store::NoCorrections::act
`rust:apps/causeway-server/src/metadata_admin/stuck_work.rs::tests::a_source_that_takes_no_correction_refuses_one_before_the_store::NoCorrections::act`  apps/causeway-server/src/metadata_admin/stuck_work.rs:415-423  pkg:cargo/causeway-server  score 0.69  via `StuckWorkSource::act` -> DISPATCHES_TO (apps/causeway-server/src/metadata_admin/stuck_work.rs:415-423, 1.00)  
`fn act( &self, _id: &RecordId, _action: &StuckWorkAction, _actor: &Actor, ) -> stuck_work::StuckWorkFuture<'_, Result<StuckWorkRecord, stuck_work::StuckWorkError>>`  
callers 3: `act` (apps/causeway-server/src/metadata_admin/stuck_work.rs:311), `tests::a_source_that_takes_no_correction_refuses_one_before_the_store` (apps/causeway-server/src/metadata_admin/stuck_work.rs:441), `tests::acting_on_an_unregistered_source_says_which_one` (apps/causeway-server/src/metadata_admin/stuck_work.rs:455)  
dispatched_from 1: `StuckWorkSource::act` (apps/causeway-server/src/metadata_admin/stuck_work.rs:415-423)  
tests 2: `tests::acting_on_an_unregistered_source_says_which_one` (apps/causeway-server/src/metadata_admin/stuck_work.rs:455), `tests::a_source_that_takes_no_correction_refuses_one_before_the_store` (apps/causeway-server/src/metadata_admin/stuck_work.rs:441)  
uses_types 6: `StuckWorkFuture` (apps/causeway-server/src/metadata_admin/stuck_work.rs:420), `Actor` (apps/causeway-server/src/metadata_admin/stuck_work.rs:419), `RecordId` (apps/causeway-server/src/metadata_admin/stuck_work.rs:417) … 3 more  

### 57. Package cargo/serde_json
`pkg:cargo/serde_json`  pkg:cargo/serde_json  score 0.69  via `cargo/stuck-work` -> DEPENDS_ON (crates/stuck-work/Cargo.toml:1-20, 1.00)  
dependents 145: `cargo/py-drive` (ffi/py-drive/Cargo.toml:22), `cargo/causeway-testkit` (crates/causeway-testkit/Cargo.toml:1-87), `cargo/coherence-component` (component/platform/coherence/Cargo.toml:1-44) … 142 more  
importers 136: `crates/platform-flags/tests/repository_group_rules.rs` (crates/platform-flags/tests/repository_group_rules.rs:19), `tests` (crates/causeway-metadata-ops/src/submit.rs:496), `tests` (crates/drive-runtime/src/schema_args.rs:178) … 133 more  

### 58. File crates/stuck-work/src/lib.rs
`file:crates/stuck-work/src/lib.rs`  crates/stuck-work/src/lib.rs  pkg:cargo/stuck-work  score 0.68  via `cargo/stuck-work` -> CONTAINS (crates/stuck-work/Cargo.toml:2, 1.00)  
members 21  
imports 23: `StuckWorkFuture` (crates/stuck-work/src/lib.rs:30-35), `StuckWorkAction` (crates/stuck-work/src/lib.rs:30-35), `BlastRadius` (crates/stuck-work/src/lib.rs:30-35) … 20 more  

### 59. Method IngressDeadLetterStuckWork::reclaim
`rust:component/platform/event-operations/src/stuck_work.rs::IngressDeadLetterStuckWork::reclaim`  component/platform/event-operations/src/stuck_work.rs:503-523  pkg:cargo/event-operations-component  score 0.68  via `StuckWorkSource::reclaim` -> DISPATCHES_TO (component/platform/event-operations/src/stuck_work.rs:503-523, 1.00)  
`fn reclaim( &self, id: &RecordId, ) -> StuckWorkFuture<'_, Result<StuckWorkRecord, StuckWorkError>>`  
Returns an abandoned replay to `open`. (doc continues, 296 bytes)  
calls 6: `IngressDeadLetterStuckWork::source_id` (component/platform/event-operations/src/stuck_work.rs:519), `not_applicable_or_unavailable` (component/platform/event-operations/src/stuck_work.rs:513), `IngressDeadLetterStuckWork::read` (component/platform/event-operations/src/stuck_work.rs:514) … 3 more  
dispatched_from 1: `StuckWorkSource::reclaim` (component/platform/event-operations/src/stuck_work.rs:503-523)  
uses_types 5: `StuckWorkFuture` (component/platform/event-operations/src/stuck_work.rs:506), `StuckWorkError` (component/platform/event-operations/src/stuck_work.rs:506), `RecordId` (component/platform/event-operations/src/stuck_work.rs:505) … 2 more  

### 60. Package cargo/causeway-metadata-ops
`pkg:cargo/causeway-metadata-ops`  crates/causeway-metadata-ops/Cargo.toml:8  pkg:cargo/causeway-metadata-ops  score 0.67  via `cargo/stuck-work` <- DEPENDS_ON (crates/causeway-metadata-ops/Cargo.toml:1-119, 1.00)  
`cargo/causeway-metadata-ops`  
members 69  
dependents 10: `cargo/causeway-content-integrity-smoke` (jobs/causeway-content-integrity-smoke/Cargo.toml:13), `cargo/causeway-seed` (jobs/causeway-seed/Cargo.toml:21), `cargo/causeway-metadata-outbox-relay` (jobs/causeway-metadata-outbox-relay/Cargo.toml:14) … 7 more  
dependencies 70: `cargo/workflows-component` (crates/causeway-metadata-ops/Cargo.toml:98), `cargo/catalog-domain` (crates/causeway-metadata-ops/Cargo.toml:42), `cargo/contracts-component` (crates/causeway-metadata-ops/Cargo.toml:58) … 67 more  
designs 2: `docs/rfd/RFD-fpg4tnq2-ephemeral-infrastructure-for-ci-integration-tests.md` (docs/rfd/RFD-fpg4tnq2-ephemeral-infrastructure-for-ci-integration-tests.md:110), `docs/rfd/RFD-h34cm94x-semantic-resource-identity-and-rdf-projection.md` (docs/rfd/RFD-h34cm94x-semantic-resource-identity-and-rdf-projection.md:331)  
documents 2: `docs/plans/SERVICE-BOUNDARIES-WORKPLAN.md` (docs/plans/SERVICE-BOUNDARIES-WORKPLAN.md:128), `docs/METADATA-INGESTION-PATH.md` (docs/METADATA-INGESTION-PATH.md:6)  
importers 21: `jobs/causeway-metadata-outbox-relay/src/main.rs` (jobs/causeway-metadata-outbox-relay/src/main.rs:8), `jobs/causeway-fibo-ingest/src/column_claims.rs` (jobs/causeway-fibo-ingest/src/column_claims.rs:14), `jobs/causeway-metadata-worker/src/ownership_restate.rs` (jobs/causeway-metadata-worker/src/ownership_restate.rs:6-8) … 18 more  

### 61. Function blast_radius
`rust:apps/causeway-server-web/src/views/platform/operations/stuck_work.rs::blast_radius`  apps/causeway-server-web/src/views/platform/operations/stuck_work.rs:348-368  pkg:cargo/causeway-server-web  score 0.67  via `stuck_work_record` -> CALLS (apps/causeway-server-web/src/views/platform/operations/stuck_work.rs:189, 1.00)  
`fn blast_radius(record: &StuckWorkRecord) -> Markup`  
callers 1: `stuck_work_record` (apps/causeway-server-web/src/views/platform/operations/stuck_work.rs:189)  
uses_types 1: `StuckWorkRecord` (apps/causeway-server-web/src/views/platform/operations/stuck_work.rs:348)  

### 62. File apps/causeway-server/src/routes/pages.rs
`file:apps/causeway-server/src/routes/pages.rs`  apps/causeway-server/src/routes/pages.rs  pkg:cargo/causeway-server  score 0.67  via `stuck_work_record` <- IMPORTS (apps/causeway-server/src/routes/pages.rs:53-58, 1.00)  
members 33  
documents 2: `docs/DESIGN-LOG.md` (docs/DESIGN-LOG.md:834), `docs/plans/ODRL-RIGHTS-WORKPLAN.md` (docs/plans/ODRL-RIGHTS-WORKPLAN.md:71)  
imports 103: `contract_review_submit` (apps/causeway-server/src/routes/pages.rs:30-32), `glossary_term_by_id` (apps/causeway-server/src/routes/pages.rs:36-42), `Breadcrumb` (apps/causeway-server/src/routes/pages.rs:10) … 100 more  

### 63. Package cargo/metadata-component
`pkg:cargo/metadata-component`  component/metadata/core/Cargo.toml:2  pkg:cargo/metadata-component  score 0.66  via `cargo/stuck-work` <- DEPENDS_ON (component/metadata/core/Cargo.toml:1-61, 1.00)  
`cargo/metadata-component`  
members 27  
dependents 7: `cargo/causeway-seed` (jobs/causeway-seed/Cargo.toml:22), `cargo/causeway-policy-host` (apps/causeway-policy-host/Cargo.toml:35), `cargo/causeway-policy-scan` (apps/causeway-policy-scan/Cargo.toml:31) … 4 more  
dependencies 19: `cargo/tokio` (component/metadata/core/Cargo.toml:1-61), `cargo/sha2` (component/metadata/core/Cargo.toml:1-61), `cargo/chrono` (component/metadata/core/Cargo.toml:1-61) … 16 more  
documents 3: `docs/plans/OPERATIONAL-RESILIENCE-WORKPLAN.md` (docs/plans/OPERATIONAL-RESILIENCE-WORKPLAN.md:59), `docs/METADATA-INGESTION-PATH.md` (docs/METADATA-INGESTION-PATH.md:6), `docs/plans/VSA-WORKPLAN.md` (docs/plans/VSA-WORKPLAN.md:61)  
importers 42: `jobs/causeway-fibo-ingest/src/term_governance_claims.rs` (jobs/causeway-fibo-ingest/src/term_governance_claims.rs:24), `crates/causeway-metadata-ops/src/resolve.rs` (crates/causeway-metadata-ops/src/resolve.rs:6), `apps/causeway-server/src/composition/state/service_ports.rs` (apps/causeway-server/src/composition/state/service_ports.rs:16) … 39 more  

### 64. Function refused_to_list
`rust:apps/causeway-server/src/routes/pages/metadata_ops/stuck_work.rs::refused_to_list`  apps/causeway-server/src/routes/pages/metadata_ops/stuck_work.rs:367-374  pkg:cargo/causeway-server  score 0.66  via `stuck_work_record` -> CALLS (apps/causeway-server/src/routes/pages/metadata_ops/stuck_work.rs:240, 1.00)  
`fn refused_to_list(message: &str) -> Response`  
callers 3: `stuck_work_act` (apps/causeway-server/src/routes/pages/metadata_ops/stuck_work.rs:274), `stuck_work_record` (apps/causeway-server/src/routes/pages/metadata_ops/stuck_work.rs:236), `stuck_work_record` (apps/causeway-server/src/routes/pages/metadata_ops/stuck_work.rs:240)  

### 65. Method IngressDeadLetterStuckWork::act
`rust:component/platform/event-operations/src/stuck_work.rs::IngressDeadLetterStuckWork::act`  component/platform/event-operations/src/stuck_work.rs:393-403  pkg:cargo/event-operations-component  score 0.66  via `StuckWorkSource::act` -> DISPATCHES_TO (component/platform/event-operations/src/stuck_work.rs:393-403, 1.00)  
`fn act( &self, id: &RecordId, action: &StuckWorkAction, actor: &Actor, ) -> StuckWorkFuture<'_, Result<StuckWorkRecord, StuckWorkError>>`  
calls 1: `IngressDeadLetterStuckWork::apply` (component/platform/event-operations/src/stuck_work.rs:402)  
dispatched_from 1: `StuckWorkSource::act` (component/platform/event-operations/src/stuck_work.rs:393-403)  
uses_types 6: `Actor` (component/platform/event-operations/src/stuck_work.rs:397), `RecordId` (component/platform/event-operations/src/stuck_work.rs:395), `StuckWorkRecord` (component/platform/event-operations/src/stuck_work.rs:398) … 3 more  

### 66. Package cargo/causeway-policy-scan
`pkg:cargo/causeway-policy-scan`  apps/causeway-policy-scan/Cargo.toml:2  pkg:cargo/causeway-policy-scan  score 0.66  via `cargo/stuck-work` <- DEPENDS_ON (apps/causeway-policy-scan/Cargo.toml:41, 1.00)  
`cargo/causeway-policy-scan`  
members 7  
dependencies 41: `cargo/policy-engine` (apps/causeway-policy-scan/Cargo.toml:29), `cargo/rdkafka` (apps/causeway-policy-scan/Cargo.toml:1-69), `cargo/causeway-policy-bundle` (apps/causeway-policy-scan/Cargo.toml:14) … 38 more  
documents 1: `docs/plans/ODRL-RIGHTS-WORKPLAN.md` (docs/plans/ODRL-RIGHTS-WORKPLAN.md:58)  

### 67. File apps/causeway-server/src/metadata_admin/handlers.rs
`file:apps/causeway-server/src/metadata_admin/handlers.rs`  apps/causeway-server/src/metadata_admin/handlers.rs  pkg:cargo/causeway-server  score 0.66  via `stuck_work_record` <- CONTAINS (apps/causeway-server/src/metadata_admin/handlers.rs:14-49, 1.00)  
members 6  
documents 1: `docs/DESIGN-LOG.md` (docs/DESIGN-LOG.md:93)  
imports 2: `MetadataReasonDetails` (apps/causeway-server/src/metadata_admin/handlers.rs:5), `cargo/metadata-domain` (apps/causeway-server/src/metadata_admin/handlers.rs:5)  

### 68. Method PolicyScanStuckWork::reclaim
`rust:component/governance/policy/src/stuck_work.rs::PolicyScanStuckWork::reclaim`  component/governance/policy/src/stuck_work.rs:561-595  pkg:cargo/governance-policy-component  score 0.66  via `StuckWorkSource::reclaim` -> DISPATCHES_TO (component/governance/policy/src/stuck_work.rs:561-595, 1.00)  
`fn reclaim( &self, id: &RecordId, ) -> StuckWorkFuture<'_, Result<StuckWorkRecord, StuckWorkError>>`  
Returns an abandoned claim to the ready queue. (doc continues, 382 bytes)  
members 1  
calls 4: `PolicyScanStuckWork::read` (component/governance/policy/src/stuck_work.rs:586), `PolicyScanStuckWork::source_id` (component/governance/policy/src/stuck_work.rs:591), `PolicyScanStuckWork::source_id` (component/governance/policy/src/stuck_work.rs:569) … 1 more  
dispatched_from 1: `StuckWorkSource::reclaim` (component/governance/policy/src/stuck_work.rs:561-595)  
uses_types 5: `RecordId` (component/governance/policy/src/stuck_work.rs:563), `NotFound` (component/governance/policy/src/stuck_work.rs:568), `StuckWorkRecord` (component/governance/policy/src/stuck_work.rs:564) … 2 more  
writes 1: `public.policy_scan_queue` (component/governance/policy/src/stuck_work.rs:573)  

### 69. Method StuckWorkRecordPage::new
`rust:apps/causeway-server-web/src/views/platform/operations/stuck_work.rs::StuckWorkRecordPage::new`  apps/causeway-server-web/src/views/platform/operations/stuck_work.rs:119-131  pkg:cargo/causeway-server-web  score 0.64  via `stuck_work_record` -> CALLS (apps/causeway-server/src/routes/pages/metadata_ops/stuck_work.rs:242, 1.00)  
`pub const fn new( detail: StuckWorkDetail, corrections: Correctable, failing_seat: Option<String>, ) -> Self`  
The page over one record.  
callers 1: `stuck_work_record` (apps/causeway-server/src/routes/pages/metadata_ops/stuck_work.rs:242)  
uses_types 2: `StuckWorkDetail` (apps/causeway-server-web/src/views/platform/operations/stuck_work.rs:120), `Correctable` (apps/causeway-server-web/src/views/platform/operations/stuck_work.rs:121)  

### 70. Method CoherenceFindings::act
`rust:component/platform/coherence/src/source.rs::CoherenceFindings::act`  component/platform/coherence/src/source.rs:283-370  pkg:cargo/coherence-component  score 0.64  via `StuckWorkSource::act` -> DISPATCHES_TO (component/platform/coherence/src/source.rs:283-370, 1.00)  
`fn act( &self, id: &RecordId, action: &StuckWorkAction, actor: &Actor, ) -> StuckWorkFuture<'_, Result<StuckWorkRecord, StuckWorkError>>`  
callers 2: `database_tests::a_finding_is_listed_reviewed_and_re_asked` (component/platform/coherence/src/source.rs:552), `database_tests::a_finding_is_listed_reviewed_and_re_asked` (component/platform/coherence/src/source.rs:569)  
calls 11: `CoherenceFindings::get` (component/platform/coherence/src/source.rs:365), `CoherenceFindings::split_id` (component/platform/coherence/src/source.rs:293), `run_one` (component/platform/coherence/src/source.rs:301) … 8 more  
dispatched_from 1: `StuckWorkSource::act` (component/platform/coherence/src/source.rs:283-370)  
references 3: `StuckWorkAction::Acknowledge` (component/platform/coherence/src/source.rs:330), `StuckWorkAction::Retry` (component/platform/coherence/src/source.rs:300), `StuckWorkAction::Retry` (component/platform/coherence/src/source.rs:345)  
tests 1: `database_tests::a_finding_is_listed_reviewed_and_re_asked` (component/platform/coherence/src/source.rs:552)  
uses_types 7: `StuckWorkRecord` (component/platform/coherence/src/source.rs:288), `Actor` (component/platform/coherence/src/source.rs:287), `StuckWorkAction` (component/platform/coherence/src/source.rs:286) … 4 more  

### 71. Method MetadataAdmin::port_act_on_stuck_work
`rust:apps/causeway-server/src/metadata_admin/handlers.rs::MetadataAdmin::port_act_on_stuck_work`  apps/causeway-server/src/metadata_admin/handlers.rs:554-610  pkg:cargo/causeway-server  score 0.64  via `stuck_work_record` <- CALLS (apps/causeway-server/src/metadata_admin/handlers.rs:605, 1.00)  
`async fn port_act_on_stuck_work( &self, ctx: Context, request: OwnedView<pb::PortActOnStuckWorkRequestView<'static>>, ) -> Result<(pb::PortActOnStuckWorkResponse, Context), ConnectError>`  
callers 5: `a_stuck_work_action_is_recorded_against_the_verified_caller` (apps/causeway-server/src/metadata_admin/handlers/caller_tests.rs:160), `ending_a_stuck_record_takes_the_seat_its_source_declares` (apps/causeway-server/src/metadata_admin/handlers/caller_tests.rs:191), `a_stuck_work_action_is_recorded_against_the_verified_caller` (apps/causeway-server/src/metadata_admin/handlers/caller_tests.rs:169) … 2 more  
calls 34: `act` (apps/causeway-server/src/metadata_admin/handlers.rs:594), `invalid_argument` (apps/causeway-server/src/metadata_admin/handlers.rs:576), `RecordId::parse` (apps/causeway-server/src/metadata_admin/handlers.rs:563) … 31 more  
references 3: `SeatRefusal` (apps/causeway-server/src/metadata_admin/handlers.rs:586), `SeatRefusal::Action` (apps/causeway-server/src/metadata_admin/handlers.rs:592), `SeatRefusal::DirectoryUnavailable` (apps/causeway-server/src/metadata_admin/handlers.rs:589)  
tests 2: `ending_a_stuck_record_takes_the_seat_its_source_declares` (apps/causeway-server/src/metadata_admin/handlers/caller_tests.rs:191), `a_stuck_work_action_is_recorded_against_the_verified_caller` (apps/causeway-server/src/metadata_admin/handlers/caller_tests.rs:152)  
uses_types 1: `Context` (apps/causeway-server/src/metadata_admin/handlers.rs:556)  

### 72. Method PolicyScanStuckWork::act
`rust:component/governance/policy/src/stuck_work.rs::PolicyScanStuckWork::act`  component/governance/policy/src/stuck_work.rs:424-434  pkg:cargo/governance-policy-component  score 0.64  via `StuckWorkSource::act` -> DISPATCHES_TO (component/governance/policy/src/stuck_work.rs:424-434, 1.00)  
`fn act( &self, id: &RecordId, action: &StuckWorkAction, actor: &Actor, ) -> StuckWorkFuture<'_, Result<StuckWorkRecord, StuckWorkError>>`  
callers 1: `database_tests::a_correction_replaces_the_trigger_and_puts_the_scan_back` (component/governance/policy/src/stuck_work.rs:748)  
calls 1: `PolicyScanStuckWork::apply` (component/governance/policy/src/stuck_work.rs:433)  
dispatched_from 1: `StuckWorkSource::act` (component/governance/policy/src/stuck_work.rs:424-434)  
tests 1: `database_tests::a_correction_replaces_the_trigger_and_puts_the_scan_back` (component/governance/policy/src/stuck_work.rs:748)  
uses_types 6: `StuckWorkError` (component/governance/policy/src/stuck_work.rs:429), `StuckWorkRecord` (component/governance/policy/src/stuck_work.rs:429), `StuckWorkAction` (component/governance/policy/src/stuck_work.rs:427) … 3 more  

### 73. Method MetadataQueueStuckWork::act
`rust:component/metadata/core/src/stuck_work.rs::MetadataQueueStuckWork::act`  component/metadata/core/src/stuck_work.rs:462-472  pkg:cargo/metadata-component  score 0.64  via `StuckWorkSource::act` -> DISPATCHES_TO (component/metadata/core/src/stuck_work.rs:462-472, 1.00)  
`fn act( &self, id: &RecordId, action: &StuckWorkAction, actor: &Actor, ) -> StuckWorkFuture<'_, Result<StuckWorkRecord, StuckWorkError>>`  
callers 5: `database_tests::a_correction_replaces_the_payload_and_puts_the_work_back` (component/metadata/core/src/stuck_work.rs:826), `database_tests::a_quarantined_row_reads_and_acts_through_the_port` (component/metadata/core/src/stuck_work.rs:761), `database_tests::a_quarantined_row_reads_and_acts_through_the_port` (component/metadata/core/src/stuck_work.rs:748) … 2 more  
calls 1: `MetadataQueueStuckWork::apply` (component/metadata/core/src/stuck_work.rs:471)  
dispatched_from 1: `StuckWorkSource::act` (component/metadata/core/src/stuck_work.rs:462-472)  
tests 3: `database_tests::a_correction_replaces_the_payload_and_puts_the_work_back` (component/metadata/core/src/stuck_work.rs:826), `database_tests::a_quarantined_row_reads_and_acts_through_the_port` (component/metadata/core/src/stuck_work.rs:748), `database_tests::a_tombstone_closes_the_subjects_open_stuck_work_as_superseded` (component/metadata/core/src/maintenance.rs:501)  
uses_types 6: `StuckWorkAction` (component/metadata/core/src/stuck_work.rs:465), `StuckWorkRecord` (component/metadata/core/src/stuck_work.rs:467), `StuckWorkError` (component/metadata/core/src/stuck_work.rs:467) … 3 more  

### 74. Function refused_unless_admitted
`rust:apps/causeway-server/src/routes/pages/metadata_ops/stuck_work.rs::refused_unless_admitted`  apps/causeway-server/src/routes/pages/metadata_ops/stuck_work.rs:101-113  pkg:cargo/causeway-server  score 0.63  via `stuck_work_record` -> CALLS (apps/causeway-server/src/routes/pages/metadata_ops/stuck_work.rs:225, 1.00)  
`async fn refused_unless_admitted( state: &AppState, caller: &BrowserCaller, request: (&Method, &Uri, &HeaderMap), breadcrumbs: &[Breadcrumb], ) -> Option<Response>`  
The stuck-work page's answer when the plane does not admit `caller` (doc continues, 124 bytes)  
callers 2: `stuck_work` (apps/causeway-server/src/routes/pages/metadata_ops/stuck_work.rs:140), `stuck_work_record` (apps/causeway-server/src/routes/pages/metadata_ops/stuck_work.rs:225)  
calls 1: `decide` (apps/causeway-server/src/routes/pages/metadata_ops/stuck_work.rs:107)  
references 2: `Section::STUCK_WORK` (apps/causeway-server/src/routes/pages/metadata_ops/stuck_work.rs:110), `STUCK_WORK_TWIN` (apps/causeway-server/src/routes/pages/metadata_ops/stuck_work.rs:107)  
uses_types 3: `AppState` (apps/causeway-server/src/routes/pages/metadata_ops/stuck_work.rs:102), `Breadcrumb` (apps/causeway-server/src/routes/pages/metadata_ops/stuck_work.rs:105), `BrowserCaller` (apps/causeway-server/src/routes/pages/metadata_ops/stuck_work.rs:103)  

### 75. Method SourceId::parse
`rust:crates/stuck-work/src/port.rs::SourceId::parse`  crates/stuck-work/src/port.rs:38-44  pkg:cargo/stuck-work  score 0.62  via `stuck_work_record` -> CALLS (apps/causeway-server/src/routes/pages/metadata_ops/stuck_work.rs:235, 1.00)  
`pub fn parse(value: impl Into<String>) -> Result<Self, BlankField>`  
A source identifier. (doc continues, 81 bytes)  
callers 24: `tests::Fake::id` (component/platform/supervision/src/pass.rs:291), `Recording::id` (apps/causeway-server/src/metadata_admin/handlers/caller_tests.rs:45), `tests::a_source_that_takes_no_correction_refuses_one_before_the_store` (apps/causeway-server/src/metadata_admin/stuck_work.rs:443) … 21 more  
tests 3: `tests::a_source_that_takes_no_correction_refuses_one_before_the_store` (apps/causeway-server/src/metadata_admin/stuck_work.rs:443), `tests::acting_on_an_unregistered_source_says_which_one` (apps/causeway-server/src/metadata_admin/stuck_work.rs:457), `tests::a_query_narrows_by_source_and_subject_kind` (crates/stuck-work/src/port.rs:1002)  
uses_types 1: `BlankField` (crates/stuck-work/src/port.rs:38)  

### 76. Package cargo/supervision-component
`pkg:cargo/supervision-component`  component/platform/supervision/Cargo.toml:2  pkg:cargo/supervision-component  score 0.62  via `cargo/stuck-work` <- DEPENDS_ON (component/platform/supervision/Cargo.toml:1-34, 1.00)  
`cargo/supervision-component`  
members 5  
dependents 2: `cargo/causeway-stuck-work-sweep` (jobs/causeway-stuck-work-sweep/Cargo.toml:20), `cargo/causeway-server` (apps/causeway-server/Cargo.toml:108)  
dependencies 10: `cargo/tokio` (component/platform/supervision/Cargo.toml:31), `cargo/thiserror` (component/platform/supervision/Cargo.toml:1-34), `cargo/chrono` (component/platform/supervision/Cargo.toml:1-34) … 7 more  
documents 1: `docs/plans/OPERATIONAL-RESILIENCE-WORKPLAN.md` (docs/plans/OPERATIONAL-RESILIENCE-WORKPLAN.md:133)  
importers 1: `jobs/causeway-stuck-work-sweep/src/main.rs` (jobs/causeway-stuck-work-sweep/src/main.rs:25)  

### 77. Method Recording::act
`rust:apps/causeway-server/src/metadata_admin/handlers/caller_tests.rs::Recording::act`  apps/causeway-server/src/metadata_admin/handlers/caller_tests.rs:62-73  pkg:cargo/causeway-server  score 0.62  via `StuckWorkSource::act` -> DISPATCHES_TO (apps/causeway-server/src/metadata_admin/handlers/caller_tests.rs:62-73, 1.00)  
`fn act( &self, _id: &RecordId, _action: &StuckWorkAction, actor: &Actor, ) -> StuckWorkFuture<'_, Result<StuckWorkRecord, StuckWorkError>>`  
callers 5: `ending_a_stuck_record_takes_the_seat_its_source_declares` (apps/causeway-server/src/metadata_admin/handlers/caller_tests.rs:191), `a_stuck_work_action_is_recorded_against_the_verified_caller` (apps/causeway-server/src/metadata_admin/handlers/caller_tests.rs:152), `a_stuck_work_action_is_recorded_against_the_verified_caller` (apps/causeway-server/src/metadata_admin/handlers/caller_tests.rs:160) … 2 more  
calls 1: `record` (apps/causeway-server/src/metadata_admin/handlers/caller_tests.rs:72)  
dispatched_from 1: `StuckWorkSource::act` (apps/causeway-server/src/metadata_admin/handlers/caller_tests.rs:62-73)  
tests 2: `ending_a_stuck_record_takes_the_seat_its_source_declares` (apps/causeway-server/src/metadata_admin/handlers/caller_tests.rs:191), `a_stuck_work_action_is_recorded_against_the_verified_caller` (apps/causeway-server/src/metadata_admin/handlers/caller_tests.rs:152)  
uses_types 6: `StuckWorkError` (apps/causeway-server/src/metadata_admin/handlers/caller_tests.rs:67), `StuckWorkAction` (apps/causeway-server/src/metadata_admin/handlers/caller_tests.rs:65), `StuckWorkRecord` (apps/causeway-server/src/metadata_admin/handlers/caller_tests.rs:67) … 3 more  

### 78. Function routes
`rust:apps/causeway-server/src/routes/app/operations.rs::routes`  apps/causeway-server/src/routes/app/operations.rs:10-42  pkg:cargo/causeway-server  score 0.62  via `stuck_work_record` <- REFERENCES (apps/causeway-server/src/routes/app/operations.rs:35, 1.00)  
`pub(super) fn routes() -> Router<AppState>`  
callers 1: `routes` (apps/causeway-server/src/routes/app.rs:21)  
references 13: `stuck_work_record` (apps/causeway-server/src/routes/app/operations.rs:35), `metadata_quarantine_replay` (apps/causeway-server/src/routes/app/operations.rs:23), `finops` (apps/causeway-server/src/routes/app/operations.rs:41) … 10 more  
uses_types 1: `AppState` (apps/causeway-server/src/routes/app/operations.rs:10)  

### 79. File crates/stuck-work/Cargo.toml
`file:crates/stuck-work/Cargo.toml`  crates/stuck-work/Cargo.toml  pkg:cargo/stuck-work  score 0.61  via `cargo/stuck-work` -> CONTAINS (crates/stuck-work/Cargo.toml:2, 1.00)  

### 80. Package cargo/governance-policy-component
`pkg:cargo/governance-policy-component`  component/governance/policy/Cargo.toml:2  pkg:cargo/governance-policy-component  score 0.61  via `cargo/stuck-work` <- DEPENDS_ON (component/governance/policy/Cargo.toml:1-59, 1.00)  
`cargo/governance-policy-component`  
members 23  
dependents 6: `cargo/causeway-server` (apps/causeway-server/Cargo.toml:67), `cargo/causeway-stuck-work-sweep` (jobs/causeway-stuck-work-sweep/Cargo.toml:25), `cargo/causeway-policy-scan` (apps/causeway-policy-scan/Cargo.toml:23) … 3 more  
dependencies 28: `cargo/connect-server-ext` (component/governance/policy/Cargo.toml:1-59), `cargo/uuid` (component/governance/policy/Cargo.toml:1-59), `cargo/connectrpc` (component/governance/policy/Cargo.toml:19) … 25 more  
designs 1: `docs/rfd/RFD-h34cm94x-semantic-resource-identity-and-rdf-projection.md` (docs/rfd/RFD-h34cm94x-semantic-resource-identity-and-rdf-projection.md:340)  
importers 25: `apps/causeway-policy-host/src/warehouse_access.rs` (apps/causeway-policy-host/src/warehouse_access.rs:13), `apps/causeway-policy-scan/src/warehouse_publication.rs` (apps/causeway-policy-scan/src/warehouse_publication.rs:16-20), `tests::the_reconciler_reaches_a_relay_with_a_token_mokta_minted` (apps/causeway-policy-host/src/warehouse.rs:782) … 22 more  

### 81. Function feedback
`rust:apps/causeway-server-web/src/views/platform/operations/stuck_work.rs::feedback`  apps/causeway-server-web/src/views/platform/operations/stuck_work.rs:499-512  pkg:cargo/causeway-server-web  score 0.60  via `stuck_work_record` -> CALLS (apps/causeway-server-web/src/views/platform/operations/stuck_work.rs:188, 1.00)  
`fn feedback(notice: Option<&str>, error: Option<&str>) -> Markup`  
callers 2: `stuck_work_record` (apps/causeway-server-web/src/views/platform/operations/stuck_work.rs:188), `stuck_work_list` (apps/causeway-server-web/src/views/platform/operations/stuck_work.rs:157)  

### 82. Package cargo/event-operations-component
`pkg:cargo/event-operations-component`  component/platform/event-operations/Cargo.toml:2  pkg:cargo/event-operations-component  score 0.60  via `cargo/stuck-work` <- DEPENDS_ON (component/platform/event-operations/Cargo.toml:1-20, 1.00)  
`cargo/event-operations-component`  
members 8  
dependents 4: `cargo/causeway-stuck-work-sweep` (jobs/causeway-stuck-work-sweep/Cargo.toml:26), `cargo/causeway-server` (apps/causeway-server/Cargo.toml:105), `cargo/causeway-metadata-ops` (crates/causeway-metadata-ops/Cargo.toml:68) … 1 more  
dependencies 7: `cargo/causeway-db-core` (component/platform/event-operations/Cargo.toml:1-20), `cargo/chrono` (component/platform/event-operations/Cargo.toml:1-20), `cargo/uuid` (component/platform/event-operations/Cargo.toml:1-20) … 4 more  
importers 3: `tools/causeway-metadata/src/render.rs` (tools/causeway-metadata/src/render.rs:30), `crates/causeway-metadata-ops/src/dead_letter.rs` (crates/causeway-metadata-ops/src/dead_letter.rs:21), `crates/causeway-metadata-ops/tests/event_bus_recovery_acceptance.rs` (crates/causeway-metadata-ops/tests/event_bus_recovery_acceptance.rs:40-43)  

### 83. Struct AppState
`rust:apps/causeway-server/src/composition/state.rs::AppState`  apps/causeway-server/src/composition/state.rs:277-330  pkg:cargo/causeway-server  score 0.60  via `stuck_work_record` -> USES_TYPE (apps/causeway-server/src/routes/pages/metadata_ops/stuck_work.rs:217, 1.00)  
`pub(crate) struct AppState`  
members 156  
importers 145: `apps/causeway-server/src/agent/ws.rs` (apps/causeway-server/src/agent/ws.rs:11), `apps/causeway-server/src/explorer/open.rs` (apps/causeway-server/src/explorer/open.rs:20), `apps/causeway-server/src/routes/pages/query.rs` (apps/causeway-server/src/routes/pages/query.rs:8) … 142 more  
references 6: `AppState::browser_auth` (apps/causeway-server/src/composition/state.rs:329), `access` (apps/causeway-server/src/composition/state.rs:307), `services` (apps/causeway-server/src/composition/state.rs:326) … 3 more  
used_by 349: `explorer_opened` (apps/causeway-server/src/transport/explorer_session_rpc.rs:266), `task_complete` (apps/causeway-server/src/routes/pages/tasks/complete.rs:19), `tasks` (apps/causeway-server/src/routes/pages/tasks/list.rs:29) … 346 more  
uses_types 21: `PlatformRuntime` (apps/causeway-server/src/composition/state.rs:287), `SessionRuntime` (apps/causeway-server/src/composition/state.rs:278), `QueryRuntime` (apps/causeway-server/src/composition/state.rs:291) … 18 more  

### 84. Package cargo/coherence-component
`pkg:cargo/coherence-component`  component/platform/coherence/Cargo.toml:2  pkg:cargo/coherence-component  score 0.60  via `cargo/stuck-work` <- DEPENDS_ON (component/platform/coherence/Cargo.toml:1-44, 1.00)  
`cargo/coherence-component`  
members 9  
dependents 1: `cargo/causeway-server` (apps/causeway-server/Cargo.toml:112)  
dependencies 11: `cargo/serde_json` (component/platform/coherence/Cargo.toml:1-44), `cargo/serde` (component/platform/coherence/Cargo.toml:1-44), `cargo/stuck-work` (component/platform/coherence/Cargo.toml:1-44) … 8 more  
designs 1: `docs/rfd/RFD-2n2tcfe2-operational-resilience-and-the-stuck-work-surface.md` (docs/rfd/RFD-2n2tcfe2-operational-resilience-and-the-stuck-work-surface.md:292)  
documents 4: `docs/runbooks/stuck-work.md` (docs/runbooks/stuck-work.md:171), `docs/plans/OPERATIONAL-RESILIENCE-WORKPLAN.md` (docs/plans/OPERATIONAL-RESILIENCE-WORKPLAN.md:156), `docs/TASKS.md` (docs/TASKS.md:187) … 1 more  
importers 1: `component/platform/coherence/src/bin/causeway_coherence.rs` (component/platform/coherence/src/bin/causeway_coherence.rs:33)  

### 85. Method CorrectiveDeferrals::act
`rust:component/governance/policy/src/deferral.rs::CorrectiveDeferrals::act`  component/governance/policy/src/deferral.rs:301-389  pkg:cargo/governance-policy-component  score 0.58  via `StuckWorkSource::act` -> DISPATCHES_TO (component/governance/policy/src/deferral.rs:301-389, 1.00)  
`fn act( &self, id: &RecordId, action: &StuckWorkAction, actor: &Actor, ) -> StuckWorkFuture<'_, Result<StuckWorkRecord, StuckWorkError>>`  
calls 13: `CorrectiveDeferrals::source_id` (component/governance/policy/src/deferral.rs:313), `deferrals` (component/governance/policy/src/deferral.rs:369), `CorrectiveDeferrals::source_id` (component/governance/policy/src/deferral.rs:322) … 10 more  
dispatched_from 1: `StuckWorkSource::act` (component/governance/policy/src/deferral.rs:301-389)  
references 2: `StuckWorkAction::Retry` (component/governance/policy/src/deferral.rs:330), `StuckWorkAction::Acknowledge` (component/governance/policy/src/deferral.rs:331)  
uses_types 7: `Actor` (component/governance/policy/src/deferral.rs:305), `StuckWorkAction` (component/governance/policy/src/deferral.rs:304), `StuckWorkRecord` (component/governance/policy/src/deferral.rs:306) … 4 more  

## Left out

36 included nodes have no source window for want of budget.  
- `rust:component/platform/supervision/src/pass.rs::tests::Fake::act` Method 0.58 (over budget, 286 tokens)
- `rust:apps/causeway-server-web/src/views/platform/operations/stuck_work.rs::payload_section` Function 0.58 (over budget, 202 tokens)
- `rust:apps/causeway-server-web/src/views/platform/operations/stuck_work.rs::tone_for` Function 0.58 (over budget, 148 tokens)
- and 2308 more

## Unresolved references in the seeds' files

- `getTime` at apps-web/causeway-web/src/pages/data/stuck-work.ts:105 (no-declaration)
- `parseInt` at apps-web/causeway-web/src/pages/data/stuck-work.ts:75 (no-declaration)
- `else` at apps/causeway-server-web/src/views/platform/operations/stuck_work.rs:167 (no-declaration)
- `resolution` at apps/causeway-server-web/src/views/platform/operations/stuck_work.rs:196 (ambiguous)
- `var` at apps/causeway-server-web/src/views/platform/operations/stuck_work.rs:225 (no-declaration)
- `li` at apps/causeway-server-web/src/views/platform/operations/stuck_work.rs:241 (no-declaration)
- `var` at apps/causeway-server-web/src/views/platform/operations/stuck_work.rs:313 (no-declaration)
- `color` at apps/causeway-server-web/src/views/platform/operations/stuck_work.rs:316 (no-declaration)
- `var` at apps/causeway-server-web/src/views/platform/operations/stuck_work.rs:329 (no-declaration)
- `to_string_pretty` at apps/causeway-server-web/src/views/platform/operations/stuck_work.rs:412 (no-declaration)

