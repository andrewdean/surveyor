# Context pack

Task: When the open-tasks table cannot be read, the Rust /app/tasks page says "No open tasks" as if the list were empty, which hid a schema fault for days. Make the page say the tasks could not be loaded, the way the task detail page already tells a failed read from an empty one. The Node page already gets this right.

## Seeds

- `rust:apps/causeway-server/src/workflow_tasks/page.rs::load_task_detail_page` Function 0.90 (text)
- `tsx:apps-web/causeway-web/src/pages/routes/Tasks.tsx::taskHref` Function 0.86 (text)
- `rust:apps/causeway-server/src/routes/pages/twin.rs::LOAD_TASK_WITH_FORM` Const 0.86 (text)
- `rust:apps/causeway-server/src/workflow_tasks/page.rs::TaskDetailPageData::LoadFailed` Variant 0.84 (text)
- `rust:apps/causeway-server/src/routes/pages/refused.rs::Section::TASKS` Const 0.83 (text)
- `rust:apps/causeway-server/src/workflow_tasks/page.rs::load_task_list_page` Function 0.82 (text)
- `rust:apps/causeway-server/src/routes/pages/twin.rs::LIST_OPEN_TASKS` Const 0.81 (text)
- `rust:apps/causeway-server/src/routes/pages/tasks/detail.rs::task_detail` Function 0.81 (text)
- `rust:apps/causeway-server/src/routes/pages/tasks/list.rs::tasks` Function 0.80 (text)
- `rust:apps/causeway-server/src/workflow_tasks/page.rs::TaskDetailPageData` Enum 0.80 (text)
- `typescript:apps-web/causeway-web/test/app-tasks.test.ts::/app/tasks in Node` Test 0.40 (identifier `/app/tasks`)
- `typescript:apps-web/causeway-web/test/app-tasks.test.ts::/app/tasks in Node::leaves the page to the shell when the flag is closed or nobody is signed in` Test 0.40 (identifier `/app/tasks`)
- `typescript:apps-web/causeway-web/test/app-tasks.test.ts::/app/tasks in Node::passes the assignee filter on and says whose list is empty` Test 0.40 (identifier `/app/tasks`)
- `typescript:apps-web/causeway-web/test/app-tasks.test.ts::/app/tasks in Node::renders the open tasks inside the platform shell, as the signed-in person` Test 0.40 (identifier `/app/tasks`)
- `typescript:apps-web/causeway-web/test/app-tasks.test.ts::/app/tasks in Node::says the read failed rather than showing an empty list` Test 0.40 (identifier `/app/tasks`)

Budget 30000 tokens, used 29962. Seeds 15, candidates 463, included 83, left out 618.

## Context

### 1. Function load_task_detail_page
`rust:apps/causeway-server/src/workflow_tasks/page.rs::load_task_detail_page`  apps/causeway-server/src/workflow_tasks/page.rs:42-59  pkg:cargo/causeway-server  score 3.80  
`pub(crate) async fn load_task_detail_page( context: &TaskPageContext<'_>, id: &str, ) -> TaskDetailPageData`  
callers 1: `render_task_detail` (apps/causeway-server/src/routes/pages/tasks/detail.rs:55)  
importers 1: `apps/causeway-server/src/workflow_tasks.rs` (apps/causeway-server/src/workflow_tasks.rs:9-12)  
references 3: `TaskDetailPageData::Missing` (apps/causeway-server/src/workflow_tasks/page.rs:53), `TaskDetailPageData::Missing` (apps/causeway-server/src/workflow_tasks/page.rs:48), `TaskDetailPageData::LoadFailed` (apps/causeway-server/src/workflow_tasks/page.rs:56)  
uses_types 2: `TaskDetailPageData` (apps/causeway-server/src/workflow_tasks/page.rs:45), `TaskPageContext` (apps/causeway-server/src/workflow_tasks/page.rs:43)  
apps/causeway-server/src/workflow_tasks/page.rs:42-59
```rust
pub(crate) async fn load_task_detail_page(
    context: &TaskPageContext<'_>,
    id: &str,
) -> TaskDetailPageData {
    let task_uuid = match uuid::Uuid::parse_str(id) {
        Ok(uuid) => uuid,
        Err(_) => return TaskDetailPageData::Missing,
    };

    match context.tasks_read.load_task_with_form(task_uuid).await {
        Ok(Some(loaded)) => TaskDetailPageData::Found(Box::new(loaded)),
        Ok(None) => TaskDetailPageData::Missing,
        Err(error) => {
            tracing::warn!(error = %error, task_id = %id, "failed to load task + form");
            TaskDetailPageData::LoadFailed
        }
    }
}
```

### 2. Function task_detail
`rust:apps/causeway-server/src/routes/pages/tasks/detail.rs::task_detail`  apps/causeway-server/src/routes/pages/tasks/detail.rs:29-51  pkg:cargo/causeway-server  score 3.12  
`pub async fn task_detail( State(state): State<AppState>, Path(id): Path<String>, Query(query): Query<TaskDetailQuery>, Extension(caller): Extension<BrowserCaller>, method: Method, uri: Uri, headers: HeaderMap, ) -> Response`  
One task and its form, decided as its twin (doc continues, 82 bytes)  
calls 7: `task_breadcrumbs` (apps/causeway-server/src/routes/pages/tasks/detail.rs:44), `decide` (apps/causeway-server/src/routes/pages/tasks/detail.rs:38), `refused` (apps/causeway-server/src/routes/pages/tasks/detail.rs:39) … 4 more  
importers 2: `apps/causeway-server/src/routes/pages/tasks.rs` (apps/causeway-server/src/routes/pages/tasks.rs:9), `apps/causeway-server/src/routes/pages.rs` (apps/causeway-server/src/routes/pages.rs:74)  
referenced_by 1: `routes` (apps/causeway-server/src/routes/app/govern.rs:72)  
references 2: `Section::TASKS` (apps/causeway-server/src/routes/pages/tasks/detail.rs:41), `LOAD_TASK_WITH_FORM` (apps/causeway-server/src/routes/pages/tasks/detail.rs:38)  
uses_types 3: `AppState` (apps/causeway-server/src/routes/pages/tasks/detail.rs:30), `TaskDetailQuery` (apps/causeway-server/src/routes/pages/tasks/detail.rs:32), `BrowserCaller` (apps/causeway-server/src/routes/pages/tasks/detail.rs:33)  
apps/causeway-server/src/routes/pages/tasks/detail.rs:29-51
```rust
pub async fn task_detail(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<TaskDetailQuery>,
    Extension(caller): Extension<BrowserCaller>,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
) -> Response {
    if let Err(refusal) = twin::decide(&state, &caller, twin::LOAD_TASK_WITH_FORM) {
        return refused(
            &state,
            Section::TASKS,
            &refusal,
            (&method, &uri, &headers),
            &task_breadcrumbs(&id),
        )
        .await;
    }
    render_task_detail(&state, &id, &query)
        .await
        .into_response()
}
```

### 3. Enum TaskDetailPageData
`rust:apps/causeway-server/src/workflow_tasks/page.rs::TaskDetailPageData`  apps/causeway-server/src/workflow_tasks/page.rs:36-40  pkg:cargo/causeway-server  score 2.79  
`pub(crate) enum TaskDetailPageData`  
members 3  
importers 1: `apps/causeway-server/src/workflow_tasks.rs` (apps/causeway-server/src/workflow_tasks.rs:9-12)  
used_by 1: `load_task_detail_page` (apps/causeway-server/src/workflow_tasks/page.rs:45)  
apps/causeway-server/src/workflow_tasks/page.rs:36-40
```rust
pub(crate) enum TaskDetailPageData {
    Found(Box<super::LoadedTask>),
    Missing,
    LoadFailed,
}
```

### 4. Variant TaskDetailPageData::LoadFailed
`rust:apps/causeway-server/src/workflow_tasks/page.rs::TaskDetailPageData::LoadFailed`  apps/causeway-server/src/workflow_tasks/page.rs:39  pkg:cargo/causeway-server  score 2.75  
`LoadFailed`  
referenced_by 2: `render_task_detail` (apps/causeway-server/src/routes/pages/tasks/detail.rs:58), `load_task_detail_page` (apps/causeway-server/src/workflow_tasks/page.rs:56)  

### 5. Function tasks
`rust:apps/causeway-server/src/routes/pages/tasks/list.rs::tasks`  apps/causeway-server/src/routes/pages/tasks/list.rs:28-62  pkg:cargo/causeway-server  score 2.66  
`pub async fn tasks( State(state): State<AppState>, Query(query): Query<TasksQuery>, Extension(caller): Extension<BrowserCaller>, method: Method, uri: Uri, headers: HeaderMap, ) -> Response`  
The open tasks, decided as their twin `WorkflowService/ListOpenTasks` is.  
calls 13: `TasksQuery::assignee` (apps/causeway-server/src/routes/pages/tasks/list.rs:47), `tasks` (apps/causeway-server/src/routes/pages/tasks/list.rs:57), `Breadcrumb::current` (apps/causeway-server/src/routes/pages/tasks/list.rs:36) … 10 more  
importers 2: `apps/causeway-server/src/routes/pages/tasks.rs` (apps/causeway-server/src/routes/pages/tasks.rs:10), `apps/causeway-server/src/routes/pages.rs` (apps/causeway-server/src/routes/pages.rs:74)  
referenced_by 1: `routes` (apps/causeway-server/src/routes/app/govern.rs:71)  
references 2: `LIST_OPEN_TASKS` (apps/causeway-server/src/routes/pages/tasks/list.rs:37), `Section::TASKS` (apps/causeway-server/src/routes/pages/tasks/list.rs:40)  
uses_types 3: `AppState` (apps/causeway-server/src/routes/pages/tasks/list.rs:29), `BrowserCaller` (apps/causeway-server/src/routes/pages/tasks/list.rs:31), `TasksQuery` (apps/causeway-server/src/routes/pages/tasks/list.rs:30)  
apps/causeway-server/src/routes/pages/tasks/list.rs:28-62
```rust
pub async fn tasks(
    State(state): State<AppState>,
    Query(query): Query<TasksQuery>,
    Extension(caller): Extension<BrowserCaller>,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
) -> Response {
    let breadcrumbs = vec![Breadcrumb::link("Work", "#"), Breadcrumb::current("Tasks")];
    if let Err(refusal) = twin::decide(&state, &caller, twin::LIST_OPEN_TASKS) {
        return refused(
            &state,
            Section::TASKS,
            &refusal,
            (&method, &uri, &headers),
            &breadcrumbs,
        )
        .await;
    }
    let assignee = query.assignee();
    let task_context = crate::workflow_tasks::TaskPageContext::from_state(&state);
    let tasks = crate::workflow_tasks::load_task_list_page(&task_context, assignee).await;
    render(
        &state,
        "Tasks",
        "tasks",
        breadcrumbs,
        vec![],
        "",
        causeway_server_web::views::platform::tasks::tasks(&tasks, assignee, chrono::Utc::now()),
        Some(causeway_server_web::views::platform::tasks::tasks_realtime_script()),
    )
    .await
    .into_response()
}
```

### 6. Const Section::TASKS
`rust:apps/causeway-server/src/routes/pages/refused.rs::Section::TASKS`  apps/causeway-server/src/routes/pages/refused.rs:100-106  pkg:cargo/causeway-server  score 2.57  
`pub(super) const TASKS: Self = Self { title: "Tasks", active_nav: "tasks", not_found: "No open task with that id. Open the task list to pick one.", method_refused: "You may not work on tasks.", unavailable: "Tasks are unavailable; try again shortly.", };`  
The task pages, decided as their `WorkflowService` twins are.  
referenced_by 3: `task_detail` (apps/causeway-server/src/routes/pages/tasks/detail.rs:41), `task_complete` (apps/causeway-server/src/routes/pages/tasks/complete.rs:34), `tasks` (apps/causeway-server/src/routes/pages/tasks/list.rs:40)  
apps/causeway-server/src/routes/pages/refused.rs:100-106
```rust
    pub(super) const TASKS: Self = Self {
        title: "Tasks",
        active_nav: "tasks",
        not_found: "No open task with that id. Open the task list to pick one.",
        method_refused: "You may not work on tasks.",
        unavailable: "Tasks are unavailable; try again shortly.",
    };
```

### 7. Const LOAD_TASK_WITH_FORM
`rust:apps/causeway-server/src/routes/pages/twin.rs::LOAD_TASK_WITH_FORM`  apps/causeway-server/src/routes/pages/twin.rs:22-23  pkg:cargo/causeway-server  score 2.39  
`pub(super) const LOAD_TASK_WITH_FORM: &str = "causeway.workflow.v1.WorkflowService/LoadTaskWithForm";`  
The task page's twin, and the read the completion form makes first.  
referenced_by 2: `task_detail` (apps/causeway-server/src/routes/pages/tasks/detail.rs:38), `task_complete` (apps/causeway-server/src/routes/pages/tasks/complete.rs:30)  
apps/causeway-server/src/routes/pages/twin.rs:22-23
```rust
pub(super) const LOAD_TASK_WITH_FORM: &str =
    "causeway.workflow.v1.WorkflowService/LoadTaskWithForm";
```

### 8. Function load_task_list_page
`rust:apps/causeway-server/src/workflow_tasks/page.rs::load_task_list_page`  apps/causeway-server/src/workflow_tasks/page.rs:22-34  pkg:cargo/causeway-server  score 2.33  
`pub(crate) async fn load_task_list_page( context: &TaskPageContext<'_>, assignee: Option<&str>, ) -> Vec<super::TaskRow>`  
callers 1: `tasks` (apps/causeway-server/src/routes/pages/tasks/list.rs:49)  
calls 5: `WorkflowService::list_open_tasks` (apps/causeway-server/src/workflow_tasks/page.rs:28), `TasksRead::list_open_tasks` (apps/causeway-server/src/workflow_tasks/page.rs:28), `PgWorkflowReadRepo::list_open_tasks` (apps/causeway-server/src/workflow_tasks/page.rs:28) … 2 more  
importers 1: `apps/causeway-server/src/workflow_tasks.rs` (apps/causeway-server/src/workflow_tasks.rs:9-12)  
uses_types 5: `TaskPageContext` (apps/causeway-server/src/workflow_tasks/page.rs:23), `MockWorkflowReadRepo::TaskRow` (apps/causeway-server/src/workflow_tasks/page.rs:25), `PgWorkflowReadRepo::TaskRow` (apps/causeway-server/src/workflow_tasks/page.rs:25) … 2 more  
apps/causeway-server/src/workflow_tasks/page.rs:22-34
```rust
pub(crate) async fn load_task_list_page(
    context: &TaskPageContext<'_>,
    assignee: Option<&str>,
) -> Vec<super::TaskRow> {
    context
        .tasks_read
        .list_open_tasks(assignee)
        .await
        .unwrap_or_else(|error| {
            tracing::warn!(error = %error, "failed to load workflow tasks");
            Vec::new()
        })
}
```

### 9. Function render_task_detail
`rust:apps/causeway-server/src/routes/pages/tasks/detail.rs::render_task_detail`  apps/causeway-server/src/routes/pages/tasks/detail.rs:53-78  pkg:cargo/causeway-server  score 2.17  via `load_task_detail_page` <- CALLS (apps/causeway-server/src/routes/pages/tasks/detail.rs:55, 1.00)  
`async fn render_task_detail(state: &AppState, id: &str, query: &TaskDetailQuery) -> Html<String>`  
callers 1: `task_detail` (apps/causeway-server/src/routes/pages/tasks/detail.rs:48)  
calls 10: `load_task_detail_page` (apps/causeway-server/src/routes/pages/tasks/detail.rs:55), `task_wizard` (apps/causeway-server/src/routes/pages/tasks/detail.rs:74), `not_found` (apps/causeway-server/src/routes/pages/tasks/detail.rs:57) … 7 more  
references 3: `TaskDetailPageData::LoadFailed` (apps/causeway-server/src/routes/pages/tasks/detail.rs:58), `TaskDetailPageData::Found` (apps/causeway-server/src/routes/pages/tasks/detail.rs:56), `TaskDetailPageData::Missing` (apps/causeway-server/src/routes/pages/tasks/detail.rs:57)  
uses_types 2: `AppState` (apps/causeway-server/src/routes/pages/tasks/detail.rs:53), `TaskDetailQuery` (apps/causeway-server/src/routes/pages/tasks/detail.rs:53)  
apps/causeway-server/src/routes/pages/tasks/detail.rs:53-78
```rust
async fn render_task_detail(state: &AppState, id: &str, query: &TaskDetailQuery) -> Html<String> {
    let task_context = crate::workflow_tasks::TaskPageContext::from_state(state);
    let loaded = match crate::workflow_tasks::load_task_detail_page(&task_context, id).await {
        crate::workflow_tasks::TaskDetailPageData::Found(loaded) => loaded,
        crate::workflow_tasks::TaskDetailPageData::Missing => return not_found(state, id).await,
        crate::workflow_tasks::TaskDetailPageData::LoadFailed => {
            return render_load_error(state, id).await;
        }
    };
    let error = query.error();
    render(
        state,
        &loaded.task.title,
        "tasks",
        vec![
            Breadcrumb::link("Work", "#"),
            Breadcrumb::link("Tasks", "/app/tasks"),
            Breadcrumb::current(&loaded.task.title),
        ],
        vec![],
        "",
        causeway_server_web::views::platform::tasks::task_wizard(&loaded, error),
        None,
    )
    .await
}
```

### 10. Const LIST_OPEN_TASKS
`rust:apps/causeway-server/src/routes/pages/twin.rs::LIST_OPEN_TASKS`  apps/causeway-server/src/routes/pages/twin.rs:19  pkg:cargo/causeway-server  score 2.01  
`pub(super) const LIST_OPEN_TASKS: &str = "causeway.workflow.v1.WorkflowService/ListOpenTasks";`  
The task list's twin.  
referenced_by 1: `tasks` (apps/causeway-server/src/routes/pages/tasks/list.rs:37)  
apps/causeway-server/src/routes/pages/twin.rs:19-19
```rust
pub(super) const LIST_OPEN_TASKS: &str = "causeway.workflow.v1.WorkflowService/ListOpenTasks";
```

### 11. Function taskHref
`tsx:apps-web/causeway-web/src/pages/routes/Tasks.tsx::taskHref`  apps-web/causeway-web/src/pages/routes/Tasks.tsx:13-17  pkg:npm/@causeway/web  score 1.99  
`function taskHref(task: TaskRow): string`  
The review page a task opens: catalog reviews have their own.  
callers 1: `Row` (apps-web/causeway-web/src/pages/routes/Tasks.tsx:42)  
uses_types 1: `TaskRow` (apps-web/causeway-web/src/pages/routes/Tasks.tsx:13)  
apps-web/causeway-web/src/pages/routes/Tasks.tsx:13-17
```tsx
export function taskHref(task: TaskRow): string {
  return task.definition_name === "builtin.catalog-publication-review"
    ? `/app/catalog/reviews/${task.id}`
    : `/app/tasks/${task.id}`;
}
```

### 12. File apps/causeway-server/src/workflow_tasks/page.rs
`file:apps/causeway-server/src/workflow_tasks/page.rs`  apps/causeway-server/src/workflow_tasks/page.rs  pkg:cargo/causeway-server  score 1.99  via `load_task_detail_page` <- CONTAINS (apps/causeway-server/src/workflow_tasks/page.rs:42-59, 1.00)  
members 7  
imports 4: `cargo/workflows-component` (apps/causeway-server/src/workflow_tasks/page.rs:4), `TasksRead` (apps/causeway-server/src/workflow_tasks/page.rs:4), `AppState` (apps/causeway-server/src/workflow_tasks/page.rs:6) … 1 more  

### 13. Variant TaskDetailPageData::Missing
`rust:apps/causeway-server/src/workflow_tasks/page.rs::TaskDetailPageData::Missing`  apps/causeway-server/src/workflow_tasks/page.rs:38  pkg:cargo/causeway-server  score 1.98  via `load_task_detail_page` -> REFERENCES (apps/causeway-server/src/workflow_tasks/page.rs:53, 1.00)  
`Missing`  
referenced_by 3: `load_task_detail_page` (apps/causeway-server/src/workflow_tasks/page.rs:53), `load_task_detail_page` (apps/causeway-server/src/workflow_tasks/page.rs:48), `render_task_detail` (apps/causeway-server/src/routes/pages/tasks/detail.rs:57)  

### 14. File apps/causeway-server/src/routes/pages/tasks/detail.rs
`file:apps/causeway-server/src/routes/pages/tasks/detail.rs`  apps/causeway-server/src/routes/pages/tasks/detail.rs  pkg:cargo/causeway-server  score 1.77  via `task_detail` <- CONTAINS (apps/causeway-server/src/routes/pages/tasks/detail.rs:29-51, 1.00)  
members 7  
documents 1: `docs/DESIGN-LOG.md` (docs/DESIGN-LOG.md:1187)  
imports 10: `Section` (apps/causeway-server/src/routes/pages/tasks/detail.rs:13), `BrowserCaller` (apps/causeway-server/src/routes/pages/tasks/detail.rs:11), `AppState` (apps/causeway-server/src/routes/pages/tasks/detail.rs:12) … 7 more  

### 15. File apps/causeway-server/src/routes/pages/tasks.rs
`file:apps/causeway-server/src/routes/pages/tasks.rs`  apps/causeway-server/src/routes/pages/tasks.rs  pkg:cargo/causeway-server  score 1.55  via `task_detail` <- IMPORTS (apps/causeway-server/src/routes/pages/tasks.rs:9, 1.00)  
members 4  
imports 3: `tasks` (apps/causeway-server/src/routes/pages/tasks.rs:10), `task_complete` (apps/causeway-server/src/routes/pages/tasks.rs:8), `task_detail` (apps/causeway-server/src/routes/pages/tasks.rs:9)  

### 16. Struct TaskPageContext
`rust:apps/causeway-server/src/workflow_tasks/page.rs::TaskPageContext`  apps/causeway-server/src/workflow_tasks/page.rs:8-11  pkg:cargo/causeway-server  score 1.37  via `load_task_detail_page` -> USES_TYPE (apps/causeway-server/src/workflow_tasks/page.rs:43, 1.00)  
`pub(crate) struct TaskPageContext<'a>`  
members 1  
importers 1: `apps/causeway-server/src/workflow_tasks.rs` (apps/causeway-server/src/workflow_tasks.rs:9-12)  
used_by 4: `load_task_list_page` (apps/causeway-server/src/workflow_tasks/page.rs:23), `TaskPageContext#impl` (apps/causeway-server/src/workflow_tasks/page.rs:13-20), `complete_task_from_form` (apps/causeway-server/src/workflow_tasks/page.rs:67) … 1 more  
uses_types 10: `TasksCommand` (apps/causeway-server/src/workflow_tasks/page.rs:10), `TasksRead` (apps/causeway-server/src/workflow_tasks/page.rs:9), `PgWorkflowReadRepo::TaskRow` (apps/causeway-server/src/workflow_tasks/page.rs:9) … 7 more  
apps/causeway-server/src/workflow_tasks/page.rs:8-11
```rust
pub(crate) struct TaskPageContext<'a> {
    tasks_read: &'a dyn TasksRead<TaskRow = super::TaskRow, LoadedTask = super::LoadedTask>,
    tasks_command: &'a dyn TasksCommand,
}
```

### 17. File apps/causeway-server/src/routes/pages/tasks/list.rs
`file:apps/causeway-server/src/routes/pages/tasks/list.rs`  apps/causeway-server/src/routes/pages/tasks/list.rs  pkg:cargo/causeway-server  score 1.36  via `tasks` <- CONTAINS (apps/causeway-server/src/routes/pages/tasks/list.rs:28-62, 1.00)  
members 3  
documents 1: `docs/DESIGN-LOG.md` (docs/DESIGN-LOG.md:1188)  
imports 10: `AppState` (apps/causeway-server/src/routes/pages/tasks/list.rs:12), `BrowserCaller` (apps/causeway-server/src/routes/pages/tasks/list.rs:11), `render` (apps/causeway-server/src/routes/pages/tasks/list.rs:14) … 7 more  

### 18. Function task_complete
`rust:apps/causeway-server/src/routes/pages/tasks/complete.rs::task_complete`  apps/causeway-server/src/routes/pages/tasks/complete.rs:18-54  pkg:cargo/causeway-server  score 1.33  via `LOAD_TASK_WITH_FORM` <- REFERENCES (apps/causeway-server/src/routes/pages/tasks/complete.rs:30, 1.00)  
`pub async fn task_complete( State(state): State<AppState>, Path(id): Path<String>, Extension(caller): Extension<BrowserCaller>, method: Method, uri: Uri, headers: HeaderMap, axum::Form(body): axum::Form<Vec<(String, String)>>, ) -> Response`  
Completes a task from its form, decided as the two calls its twins make (doc continues, 188 bytes)  
calls 11: `Breadcrumb::link` (apps/causeway-server/src/routes/pages/tasks/complete.rs:39), `redirect_err` (apps/causeway-server/src/routes/pages/tasks/complete.rs:51), `complete_task_from_form` (apps/causeway-server/src/routes/pages/tasks/complete.rs:46) … 8 more  
importers 2: `apps/causeway-server/src/routes/pages.rs` (apps/causeway-server/src/routes/pages.rs:74), `apps/causeway-server/src/routes/pages/tasks.rs` (apps/causeway-server/src/routes/pages/tasks.rs:8)  
referenced_by 1: `routes` (apps/causeway-server/src/routes/app/govern.rs:73)  
references 5: `TaskCompletionResult::Rejected` (apps/causeway-server/src/routes/pages/tasks/complete.rs:50), `SIGNAL_TASK` (apps/causeway-server/src/routes/pages/tasks/complete.rs:30), `Section::TASKS` (apps/causeway-server/src/routes/pages/tasks/complete.rs:34) … 2 more  
uses_types 4: `AppState` (apps/causeway-server/src/routes/pages/tasks/complete.rs:19), `BrowserCaller` (apps/causeway-server/src/routes/pages/tasks/complete.rs:21), `Form` (apps/causeway-server/src/routes/pages/tasks/complete.rs:25) … 1 more  
apps/causeway-server/src/routes/pages/tasks/complete.rs:18-54
```rust
pub async fn task_complete(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Extension(caller): Extension<BrowserCaller>,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    axum::Form(body): axum::Form<Vec<(String, String)>>,
) -> Response {
    if let Err(refusal) = twin::decide_all(
        &state,
        &caller,
        &[twin::LOAD_TASK_WITH_FORM, twin::SIGNAL_TASK],
    ) {
        return refused(
            &state,
            Section::TASKS,
            &refusal,
            (&method, &uri, &headers),
            &[
                Breadcrumb::link("Work", "#"),
                Breadcrumb::link("Tasks", "/app/tasks"),
                Breadcrumb::current(&id),
            ],
        )
        .await;
    }
    let task_context = crate::workflow_tasks::TaskPageContext::from_state(&state);
    match crate::workflow_tasks::complete_task_from_form(&task_context, &id, body).await {
        crate::workflow_tasks::TaskCompletionResult::Completed => {
            Redirect::to("/app/tasks").into_response()
        }
        crate::workflow_tasks::TaskCompletionResult::Rejected(message) => {
            redirect_err(&id, &message).into_response()
        }
    }
}
```

### 19. File apps-web/causeway-web/src/pages/routes/Tasks.tsx
`file:apps-web/causeway-web/src/pages/routes/Tasks.tsx`  apps-web/causeway-web/src/pages/routes/Tasks.tsx  pkg:npm/@causeway/web  score 1.32  via `taskHref` <- CONTAINS (apps-web/causeway-web/src/pages/routes/Tasks.tsx:13-17, 1.00)  
members 5  
documents 1: `docs/DESIGN-LOG.md` (docs/DESIGN-LOG.md:1882)  
imports 8: `apps-web/causeway-web/src/pages/app-read.ts` (apps-web/causeway-web/src/pages/routes/Tasks.tsx:6), `npm/solid-js` (apps-web/causeway-web/src/pages/routes/Tasks.tsx:5), `apps-web/causeway-web/src/pages/context.ts` (apps-web/causeway-web/src/pages/routes/Tasks.tsx:8) … 5 more  

### 20. File apps/causeway-server/src/workflow_tasks.rs
`file:apps/causeway-server/src/workflow_tasks.rs`  apps/causeway-server/src/workflow_tasks.rs  pkg:cargo/causeway-server  score 1.30  via `load_task_detail_page` <- IMPORTS (apps/causeway-server/src/workflow_tasks.rs:9-12, 1.00)  
members 1  
imports 12: `TaskDetailPageData` (apps/causeway-server/src/workflow_tasks.rs:9-12), `load_task_list_page` (apps/causeway-server/src/workflow_tasks.rs:9-12), `load_task_detail_page` (apps/causeway-server/src/workflow_tasks.rs:9-12) … 9 more  

### 21. File apps/causeway-server/src/routes/pages.rs
`file:apps/causeway-server/src/routes/pages.rs`  apps/causeway-server/src/routes/pages.rs  pkg:cargo/causeway-server  score 1.25  via `task_detail` <- IMPORTS (apps/causeway-server/src/routes/pages.rs:74, 1.00)  
members 33  
documents 2: `docs/DESIGN-LOG.md` (docs/DESIGN-LOG.md:819), `docs/plans/ODRL-RIGHTS-WORKPLAN.md` (docs/plans/ODRL-RIGHTS-WORKPLAN.md:71)  
imports 103: `contract_review_submit` (apps/causeway-server/src/routes/pages.rs:30-32), `glossary_term_by_id` (apps/causeway-server/src/routes/pages.rs:36-42), `Breadcrumb` (apps/causeway-server/src/routes/pages.rs:10) … 100 more  

### 22. Method TaskPageContext::from_state
`rust:apps/causeway-server/src/workflow_tasks/page.rs::TaskPageContext::from_state`  apps/causeway-server/src/workflow_tasks/page.rs:14-19  pkg:cargo/causeway-server  score 1.24  via `tasks` -> CALLS (apps/causeway-server/src/routes/pages/tasks/list.rs:48, 1.00)  
`pub(crate) fn from_state(state: &'a AppState) -> Self`  
callers 3: `tasks` (apps/causeway-server/src/routes/pages/tasks/list.rs:48), `task_complete` (apps/causeway-server/src/routes/pages/tasks/complete.rs:45), `render_task_detail` (apps/causeway-server/src/routes/pages/tasks/detail.rs:54)  
calls 2: `AppState::tasks_command` (apps/causeway-server/src/workflow_tasks/page.rs:17), `AppState::tasks_read` (apps/causeway-server/src/workflow_tasks/page.rs:16)  
uses_types 1: `AppState` (apps/causeway-server/src/workflow_tasks/page.rs:14)  
apps/causeway-server/src/workflow_tasks/page.rs:14-19
```rust
    pub(crate) fn from_state(state: &'a AppState) -> Self {
        Self {
            tasks_read: state.tasks_read(),
            tasks_command: state.tasks_command(),
        }
    }
```

### 23. Struct AppState
`rust:apps/causeway-server/src/composition/state.rs::AppState`  apps/causeway-server/src/composition/state.rs:277-330  pkg:cargo/causeway-server  score 1.23  via `task_detail` -> USES_TYPE (apps/causeway-server/src/routes/pages/tasks/detail.rs:30, 1.00)  
`pub(crate) struct AppState`  
members 156  
importers 145: `apps/causeway-server/src/agent/ws.rs` (apps/causeway-server/src/agent/ws.rs:11), `apps/causeway-server/src/explorer/open.rs` (apps/causeway-server/src/explorer/open.rs:20), `apps/causeway-server/src/routes/pages/query.rs` (apps/causeway-server/src/routes/pages/query.rs:8) … 142 more  
references 6: `AppState::browser_auth` (apps/causeway-server/src/composition/state.rs:329), `access` (apps/causeway-server/src/composition/state.rs:307), `services` (apps/causeway-server/src/composition/state.rs:326) … 3 more  
used_by 349: `explorer_opened` (apps/causeway-server/src/transport/explorer_session_rpc.rs:266), `task_complete` (apps/causeway-server/src/routes/pages/tasks/complete.rs:19), `tasks` (apps/causeway-server/src/routes/pages/tasks/list.rs:29) … 346 more  
uses_types 21: `PlatformRuntime` (apps/causeway-server/src/composition/state.rs:287), `SessionRuntime` (apps/causeway-server/src/composition/state.rs:278), `QueryRuntime` (apps/causeway-server/src/composition/state.rs:291) … 18 more  
apps/causeway-server/src/composition/state.rs:277-330
```rust
pub(crate) struct AppState {
    session: Arc<SessionRuntime>,
    content: Arc<ContentRuntime>,
    realtime: Arc<RealtimeRuntime>,
    /// Command settlements when no Valkey is attached (`settlement_store`).
    local_settlements: Arc<settlement_store::LocalSettlements>,
    services: Arc<ServiceRuntime>,
    search: Arc<SearchRuntime>,
    catalog_serving: causeway_catalog_host::CatalogServingState,
    cache: Arc<CacheRuntime>,
    platform: Arc<PlatformRuntime>,
    object_store: Arc<ObjectStoreRuntime>,
    drive: Arc<DriveRuntime>,
    catalog: Arc<CatalogRuntime>,
    query: Arc<QueryRuntime>,
    semantic: Arc<SemanticRuntime>,
    /// How this process answers the public catalog RPC surface
    /// (RFD-q276nkj9): in process from `catalog_serving`, or
    /// forwarded to the catalog read host. Startup sets it from the typed
    /// topology; construction defaults to in process.
    catalog_rpc: RpcServing,
    /// How this process answers the catalog publication commands
    /// (RFD-q276nkj9): the command host's service in process, or
    /// forwarded to the metadata admin command host. Never the shell's own
    /// code path: the shell records no decision.
    catalog_commands: CatalogCommandServing,
    /// The metadata-admin plane's authentication, built once at startup
    /// from the environment and the topology and handed to every surface
    /// that checks it: the metadata-ops routes, the review pages and the
    /// in-process commands. No surface builds its own (RFD-j1jabh49).
    metadata_admin: Arc<causeway_metadata_admin_host::access::MetadataAdminAuth>,
    /// The Cedar half of the RPC enforcement point, built once at startup
    /// with it. The session socket asks it for each command that reaches a
    /// method the Connect-RPC surface also serves, so the command is decided
    /// as its Connect twin is (ADR-bn90xr5m, decision 5).
    rpc_authority: connect_rpc_access::RpcAuthority,
    #[cfg(test)]
    ontology_access_override: Option<bool>,
    /// Digest of the catalog-publication policy bundle in force, loaded
    /// once at startup. `None` when the bundle cannot be read — surfaces
    /// then render binding provenance without a freshness claim rather
    /// than implying currency (RFD-0115: staleness is a rendered
    /// comparison, and no comparison is not the same as fresh).
    policy_bundle_digest: Option<Arc<str>>,
    /// Display projection of the ported usage-defaults policies, parsed
    /// from the same bundle load as the digest so the two cannot skew.
    usage_defaults: Arc<causeway_server_web::services::UsageDefaults>,
    /// Display projection of the compiled contract rights (RFD-2x49z609),
    /// from the same bundle load so it cannot skew from the digest.
    contract_rights: Arc<causeway_server_web::services::ContractRights>,
    /// Browser sign-in (RFD-q276nkj9), when the topology names a
    /// browser client. `None` leaves pages anonymous.
    browser_auth: Option<Arc<crate::browser_auth::BrowserAuth>>,
}
```

### 24. File apps/causeway-server/src/routes/pages/twin.rs
`file:apps/causeway-server/src/routes/pages/twin.rs`  apps/causeway-server/src/routes/pages/twin.rs  pkg:cargo/causeway-server  score 1.22  via `LOAD_TASK_WITH_FORM` <- CONTAINS (apps/causeway-server/src/routes/pages/twin.rs:22-23, 1.00)  
members 7  
documents 1: `docs/DESIGN-LOG.md` (docs/DESIGN-LOG.md:1189)  
imports 5: `AppState` (apps/causeway-server/src/routes/pages/twin.rs:15), `PageRefusal` (apps/causeway-server/src/routes/pages/twin.rs:16), `RpcGrant` (apps/causeway-server/src/routes/pages/twin.rs:12) … 2 more  

### 25. Struct TaskDetailQuery
`rust:apps/causeway-server/src/routes/pages/tasks/detail.rs::TaskDetailQuery`  apps/causeway-server/src/routes/pages/tasks/detail.rs:17-19  pkg:cargo/causeway-server  score 1.17  via `task_detail` -> USES_TYPE (apps/causeway-server/src/routes/pages/tasks/detail.rs:32, 1.00)  
`pub(crate) struct TaskDetailQuery`  
members 1  
used_by 3: `TaskDetailQuery#impl` (apps/causeway-server/src/routes/pages/tasks/detail.rs:21-25), `task_detail` (apps/causeway-server/src/routes/pages/tasks/detail.rs:32), `render_task_detail` (apps/causeway-server/src/routes/pages/tasks/detail.rs:53)  
apps/causeway-server/src/routes/pages/tasks/detail.rs:17-19
```rust
pub(crate) struct TaskDetailQuery {
    err: Option<String>,
}
```

### 26. Variant TaskDetailPageData::Found
`rust:apps/causeway-server/src/workflow_tasks/page.rs::TaskDetailPageData::Found`  apps/causeway-server/src/workflow_tasks/page.rs:37  pkg:cargo/causeway-server  score 1.16  via `TaskDetailPageData` -> DEFINES (apps/causeway-server/src/workflow_tasks/page.rs:37, 1.00)  
`Found`  
referenced_by 1: `render_task_detail` (apps/causeway-server/src/routes/pages/tasks/detail.rs:56)  
uses_types 4: `PgWorkflowReadRepo::LoadedTask` (apps/causeway-server/src/workflow_tasks/page.rs:37), `MockWorkflowReadRepo::LoadedTask` (apps/causeway-server/src/workflow_tasks/page.rs:37), `RemoteWorkflowClient::LoadedTask` (apps/causeway-server/src/workflow_tasks/page.rs:37) … 1 more  

### 27. Test /app/tasks in Node
`typescript:apps-web/causeway-web/test/app-tasks.test.ts::/app/tasks in Node`  apps-web/causeway-web/test/app-tasks.test.ts:87-154  pkg:npm/@causeway/web  score 1.12  
`describe("/app/tasks in Node", () => { it("renders the open tasks inside the platform shell, as the signed-in person", async () => { const { tasks, asked } = fakeTasks(ROWS); const response = await (await pages(tasks)).handle(request("/app/tasks"), signedIn()); expect(response?.status).toBe(200); ex…`  
members 4  
apps-web/causeway-web/test/app-tasks.test.ts:87-154
```typescript
describe("/app/tasks in Node", () => {
  it("renders the open tasks inside the platform shell, as the signed-in person", async () => {
    const { tasks, asked } = fakeTasks(ROWS);
    const response = await (await pages(tasks)).handle(request("/app/tasks"), signedIn());
    expect(response?.status).toBe(200);
    expect(response?.headers.get("cache-control")).toBe("private, no-store");
    expect(asked).toEqual([{ bearer: "token-of-the-reader", assignee: Option.none() }]);

    const html = (await response?.text()) ?? "";
    expect(html.startsWith("<!doctype html>")).toBe(true);
    expect(html).toContain("<title>Tasks — Causeway</title>");
    expect(html).toContain("All open workflow tasks across every definition and assignee.");
    expect(html).toContain('href="/app/tasks/7d0f6a4e-2b9c-4f61-9a0e-0c1d2e3f4a5b"');
    expect(html).toContain('href="/app/catalog/reviews/1b2c3d4e-5f60-4718-8a9b-0c1d2e3f4a5c"');
    expect(html).toContain("form: contract-review@1");
    expect(html).toContain('<span class="pill pill--tide">team:governance</span>');
    expect(html).toContain('<span class="pill pill--lamp">andrew-dean</span>');
    expect(html).toContain("contract-approval@3");
    expect(html).toContain("3 hrs ago");
    expect(html).toContain("in 2 ds");
    expect(html).toContain("1 min ago");
    expect(html).toContain("—");
    // The chrome: Tasks is the highlighted entry, and the islands load with the nonce.
    expect(html).toMatch(/<a href="\/app\/tasks" title="Tasks" class="nav-item active">/);
    const nonce = /nonce-([A-Za-z0-9_-]+)/.exec(
      response?.headers.get("content-security-policy") ?? "",
    )?.[1];
    for (const tag of html.match(/<script\b[^>]*>/g) ?? []) {
      expect(tag).toContain(`nonce="${nonce}"`);
    }
    expect(html).toContain("<site-user-menu");
    expect(html).not.toContain("data-hk");
  });

  it("passes the assignee filter on and says whose list is empty", async () => {
    const { tasks, asked } = fakeTasks([]);
    const response = await (await pages(tasks)).handle(
      request("/app/tasks?assignee=team%3Adata"),
      signedIn(),
    );
    expect(asked[0]?.assignee).toEqual(Option.some("team:data"));
    const html = (await response?.text()) ?? "";
    expect(html).toContain("Open tasks assigned to team:data.");
    expect(html).toContain("No open tasks for team:data.");
    expect(html).not.toContain("<table");
  });

  it("says the read failed rather than showing an empty list", async () => {
    const { tasks } = fakeTasks("fails");
    const response = await (await pages(tasks)).handle(request("/app/tasks"), signedIn());
    expect(response?.status).toBe(200);
    const html = (await response?.text()) ?? "";
    expect(html).toContain('role="alert"');
    expect(html).toContain("The open tasks could not be loaded.");
    expect(html).not.toContain("No open tasks");
  });

  it("leaves the page to the shell when the flag is closed or nobody is signed in", async () => {
    const { tasks, asked } = fakeTasks(ROWS);
    const layer = await pages(tasks);
    expect(await layer.handle(request("/app/tasks"), signedIn([]))).toBeUndefined();
    expect(await layer.handle(request("/app/tasks"), ANONYMOUS_FACTS)).toBeUndefined();
    const open: RequestFacts = { ...ANONYMOUS_FACTS, gate: async () => true };
    expect(await layer.handle(request("/app/tasks"), open)).toBeUndefined();
    expect(await layer.handle(request("/app/tasks/7d0f6a4e"), signedIn())).toBeUndefined();
    expect(asked).toHaveLength(0);
  });
});
```

### 28. Method Breadcrumb::link
`rust:apps/causeway-server-web/src/nav.rs::Breadcrumb::link`  apps/causeway-server-web/src/nav.rs:15-20  pkg:cargo/causeway-server-web  score 1.09  via `tasks` -> CALLS (apps/causeway-server/src/routes/pages/tasks/list.rs:36, 1.00)  
`pub fn link(label: impl Into<String>, href: impl Into<String>) -> Self`  
callers 64: `query` (apps/causeway-server/src/routes/pages/query.rs:16), `create_product` (apps/causeway-server/src/routes/pages/create_product.rs:16), `task_complete` (apps/causeway-server/src/routes/pages/tasks/complete.rs:39) … 61 more  
apps/causeway-server-web/src/nav.rs:15-20
```rust
    pub fn link(label: impl Into<String>, href: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            href: Some(href.into()),
        }
    }
```

### 29. Function refused
`rust:apps/causeway-server/src/routes/pages/refused.rs::refused`  apps/causeway-server/src/routes/pages/refused.rs:131-182  pkg:cargo/causeway-server  score 1.09  via `task_detail` -> CALLS (apps/causeway-server/src/routes/pages/tasks/detail.rs:39, 1.00)  
`pub(super) async fn refused( state: &AppState, section: Section, refusal: &PageRefusal, request: (&Method, &Uri, &HeaderMap), breadcrumbs: &[Breadcrumb], ) -> Response`  
The answer to a catalog page that was not shown: the shell's sign-in (doc continues, 278 bytes)  
callers 23: `contracts` (apps/causeway-server/src/routes/pages/contracts/list.rs:33), `dataset_detail` (apps/causeway-server/src/routes/pages/catalog/detail.rs:55), `catalog` (apps/causeway-server/src/routes/pages/catalog/list.rs:33) … 20 more  
calls 12: `jsonld_error` (apps/causeway-server/src/routes/pages/refused.rs:166), `sign_in_response` (apps/causeway-server/src/routes/pages/refused.rs:141), `render` (apps/causeway-server/src/routes/pages/refused.rs:176) … 9 more  
importers 14: `apps/causeway-server/src/routes/pages/governance/workbench.rs` (apps/causeway-server/src/routes/pages/governance/workbench.rs:28), `apps/causeway-server/src/routes/pages/tasks/complete.rs` (apps/causeway-server/src/routes/pages/tasks/complete.rs:12), `apps/causeway-server/src/routes/pages/catalog/list.rs` (apps/causeway-server/src/routes/pages/catalog/list.rs:16) … 11 more  
references 3: `PageRefusal::SignInRequired` (apps/causeway-server/src/routes/pages/refused.rs:140), `PageRefusal::NotFound` (apps/causeway-server/src/routes/pages/refused.rs:143), `pages` (apps/causeway-server/src/routes/pages/refused.rs:169)  
uses_types 5: `PlatformArgs` (apps/causeway-server/src/routes/pages/refused.rs:169), `Breadcrumb` (apps/causeway-server/src/routes/pages/refused.rs:136), `Section` (apps/causeway-server/src/routes/pages/refused.rs:133) … 2 more  
apps/causeway-server/src/routes/pages/refused.rs:131-182
```rust
pub(super) async fn refused(
    state: &AppState,
    section: Section,
    refusal: &PageRefusal,
    request: (&Method, &Uri, &HeaderMap),
    breadcrumbs: &[Breadcrumb],
) -> Response {
    let (method, uri, headers) = request;
    let (status, error, message) = match refusal {
        PageRefusal::SignInRequired => {
            return crate::browser_auth::sign_in_response(method, uri, headers);
        }
        PageRefusal::NotFound => (StatusCode::NOT_FOUND, "not_found", section.not_found),
        PageRefusal::MethodRefused { .. } => (
            StatusCode::FORBIDDEN,
            "permission_denied",
            section.method_refused,
        ),
        PageRefusal::Undecided { .. } => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal",
            "Access could not be decided.",
        ),
        PageRefusal::CatalogUnavailable { .. } => (
            StatusCode::SERVICE_UNAVAILABLE,
            "unavailable",
            section.unavailable,
        ),
    };
    if status.is_server_error() {
        tracing::warn!(path = %uri.path(), ?refusal, "governed page: not decided");
    } else {
        tracing::info!(path = %uri.path(), ?refusal, "governed page: refused");
    }
    if crate::routes::pages::wants_jsonld(headers) {
        return crate::routes::pages::jsonld_error(status, error, message);
    }
    let hydration_json = crate::routes::pages::hydration(state, section.active_nav).await;
    let markup = crate::routes::pages::platform_shell(crate::routes::pages::PlatformArgs {
        title: section.title,
        active_nav: section.active_nav,
        hydration_json: &hydration_json,
        breadcrumbs,
        tabs: &[],
        active_tab: "",
        content: crate::routes::pages::placeholder::render(section.title, message),
        extra_scripts: None,
        island_scripts: None,
        preload_pdfjs: false,
    });
    (status, Html(markup.into_string())).into_response()
}
```

### 30. Struct Section
`rust:apps/causeway-server/src/routes/pages/refused.rs::Section`  apps/causeway-server/src/routes/pages/refused.rs:15-21  pkg:cargo/causeway-server  score 1.04  via `Section#impl` -> USES_TYPE (apps/causeway-server/src/routes/pages/refused.rs:23-125, 1.00)  
`pub(super) struct Section`  
The section of the shell a refused page belongs to: its title, the (doc continues, 174 bytes)  
members 12  
importers 14: `apps/causeway-server/src/routes/pages/tasks/detail.rs` (apps/causeway-server/src/routes/pages/tasks/detail.rs:13), `apps/causeway-server/src/routes/pages/metadata_ops/trace.rs` (apps/causeway-server/src/routes/pages/metadata_ops/trace.rs:36), `apps/causeway-server/src/routes/pages/catalog/list.rs` (apps/causeway-server/src/routes/pages/catalog/list.rs:16) … 11 more  
used_by 2: `Section#impl` (apps/causeway-server/src/routes/pages/refused.rs:23-125), `refused` (apps/causeway-server/src/routes/pages/refused.rs:133)  
apps/causeway-server/src/routes/pages/refused.rs:15-21
```rust
pub(super) struct Section {
    title: &'static str,
    active_nav: &'static str,
    not_found: &'static str,
    method_refused: &'static str,
    unavailable: &'static str,
}
```

### 31. Function decide
`rust:apps/causeway-server/src/routes/pages/twin.rs::decide`  apps/causeway-server/src/routes/pages/twin.rs:37-49  pkg:cargo/causeway-server  score 1.00  via `task_detail` -> CALLS (apps/causeway-server/src/routes/pages/tasks/detail.rs:38, 1.00)  
`pub(super) fn decide( state: &AppState, caller: &BrowserCaller, twin: &str, ) -> Result<RpcGrant, PageRefusal>`  
The grant `twin` answers `caller` with. (doc continues, 159 bytes)  
callers 4: `decide_all` (apps/causeway-server/src/routes/pages/twin.rs:63), `tasks` (apps/causeway-server/src/routes/pages/tasks/list.rs:37), `task_detail` (apps/causeway-server/src/routes/pages/tasks/detail.rs:38) … 1 more  
calls 3: `EntryPage::grant` (apps/causeway-server/src/routes/pages/twin.rs:47), `AppState::rpc_authority` (apps/causeway-server/src/routes/pages/twin.rs:46), `RouteAccess::rpc_authority` (apps/causeway-server/src/routes/pages/twin.rs:46)  
references 3: `PageRefusal::SignInRequired` (apps/causeway-server/src/routes/pages/twin.rs:43), `PageRefusal::method` (apps/causeway-server/src/routes/pages/twin.rs:48), `BrowserCaller::SignedIn` (apps/causeway-server/src/routes/pages/twin.rs:42)  
uses_types 4: `PageRefusal` (apps/causeway-server/src/routes/pages/twin.rs:41), `BrowserCaller` (apps/causeway-server/src/routes/pages/twin.rs:39), `AppState` (apps/causeway-server/src/routes/pages/twin.rs:38) … 1 more  
apps/causeway-server/src/routes/pages/twin.rs:37-49
```rust
pub(super) fn decide(
    state: &AppState,
    caller: &BrowserCaller,
    twin: &str,
) -> Result<RpcGrant, PageRefusal> {
    let BrowserCaller::SignedIn(access) = caller else {
        return Err(PageRefusal::SignInRequired);
    };
    state
        .rpc_authority()
        .grant(twin, Arc::clone(access))
        .map_err(PageRefusal::method)
}
```

### 32. Function pages
`typescript:apps-web/causeway-web/test/app-tasks.test.ts::pages`  apps-web/causeway-web/test/app-tasks.test.ts:67-76  pkg:npm/@causeway/web  score 1.00  via `/app/tasks in Node::leaves the page to the shell when the flag is closed or nobody is signed in` <- TESTED_BY (apps-web/causeway-web/test/app-tasks.test.ts:146, 1.00)  
`async function pages(openTasks: OpenTasks): Promise<Pages>`  
callers 5: `/app/tasks in Node::passes the assignee filter on and says whose list is empty` (apps-web/causeway-web/test/app-tasks.test.ts:123), `/app/tasks in Node::leaves the page to the shell when the flag is closed or nobody is signed in` (apps-web/causeway-web/test/app-tasks.test.ts:146), `/app/tasks in Node::says the read failed rather than showing an empty list` (apps-web/causeway-web/test/app-tasks.test.ts:136) … 2 more  
calls 2: `startPages` (apps-web/causeway-web/test/app-tasks.test.ts:72), `fsCorpus` (apps-web/causeway-web/test/app-tasks.test.ts:72)  
tests 4: `/app/tasks in Node::passes the assignee filter on and says whose list is empty` (apps-web/causeway-web/test/app-tasks.test.ts:123), `/app/tasks in Node::says the read failed rather than showing an empty list` (apps-web/causeway-web/test/app-tasks.test.ts:136), `/app/tasks in Node::renders the open tasks inside the platform shell, as the signed-in person` (apps-web/causeway-web/test/app-tasks.test.ts:90) … 1 more  
uses_types 1: `OpenTasks` (apps-web/causeway-web/test/app-tasks.test.ts:67)  
apps-web/causeway-web/test/app-tasks.test.ts:67-76
```typescript
async function pages(openTasks: OpenTasks): Promise<Pages> {
  const scope = Effect.runSync(Scope.make());
  scopes.push(scope);
  return Effect.runPromise(
    Scope.extend(
      startPages({ appDir: APP_DIR, corpus: fsCorpus(), vite: undefined, openTasks }),
      scope,
    ),
  );
}
```

### 33. Enum BrowserCaller
`rust:apps/causeway-server/src/browser_auth/middleware.rs::BrowserCaller`  apps/causeway-server/src/browser_auth/middleware.rs:26-32  pkg:cargo/causeway-server  score 0.92  via `task_detail` -> USES_TYPE (apps/causeway-server/src/routes/pages/tasks/detail.rs:33, 1.00)  
`pub(crate) enum BrowserCaller`  
Who is asking, for a request the shell is serving. (doc continues, 257 bytes)  
members 3  
importers 52: `apps/causeway-server/src/routes/pages/governance/placements/tests.rs` (apps/causeway-server/src/routes/pages/governance/placements/tests.rs:6), `apps/causeway-server/src/routes/pages/test_shell.rs` (apps/causeway-server/src/routes/pages/test_shell.rs:18), `apps/causeway-server/src/routes/pages/tasks/detail.rs` (apps/causeway-server/src/routes/pages/tasks/detail.rs:11) … 49 more  
used_by 97: `SessionConnection::browser_caller` (apps/causeway-server/src/ws/session.rs:100), `decide_emit` (apps/causeway-server/src/routes/pages/governance/workbench.rs:144), `signed_in_person` (apps/causeway-server/src/ws/tests.rs:268) … 94 more  
apps/causeway-server/src/browser_auth/middleware.rs:26-32
```rust
pub(crate) enum BrowserCaller {
    /// Nobody is signed in: no session cookie, one that no longer validates,
    /// or no browser sign-in configured.
    Anonymous,
    /// The signed-in person, whose session token the shared verifier proved.
    SignedIn(BrowserAccess),
}
```

### 34. TypeAlias TaskRow
`typescript:apps-web/causeway-web/src/pages/data/tasks.ts::TaskRow`  apps-web/causeway-web/src/pages/data/tasks.ts:26  pkg:npm/@causeway/web  score 0.89  via `taskHref` -> USES_TYPE (apps-web/causeway-web/src/pages/routes/Tasks.tsx:13, 1.00)  
`type TaskRow = typeof TaskRow.Type;`  
importers 2: `apps-web/causeway-web/src/pages/routes/Tasks.tsx` (apps-web/causeway-web/src/pages/routes/Tasks.tsx:9), `apps-web/causeway-web/test/app-tasks.test.ts` (apps-web/causeway-web/test/app-tasks.test.ts:11)  
used_by 2: `Row` (apps-web/causeway-web/src/pages/routes/Tasks.tsx:37), `taskHref` (apps-web/causeway-web/src/pages/routes/Tasks.tsx:13)  
apps-web/causeway-web/src/pages/data/tasks.ts:26-26
```typescript
export type TaskRow = typeof TaskRow.Type;
```

### 35. Test /app/tasks in Node::leaves the page to the shell when the flag is closed or nobody is signed in
`typescript:apps-web/causeway-web/test/app-tasks.test.ts::/app/tasks in Node::leaves the page to the shell when the flag is closed or nobody is signed in`  apps-web/causeway-web/test/app-tasks.test.ts:144-153  pkg:npm/@causeway/web  score 0.88  
`it("leaves the page to the shell when the flag is closed or nobody is signed in", async () => { const { tasks, asked } = fakeTasks(ROWS); const layer = await pages(tasks); expect(await layer.handle(request("/app/tasks"), signedIn([]))).toBeUndefined(); expect(await layer.handle(request("/app/tasks")…`  
calls 16: `request` (apps-web/causeway-web/test/app-tasks.test.ts:151), `request` (apps-web/causeway-web/test/app-tasks.test.ts:150), `pages` (apps-web/causeway-web/test/app-tasks.test.ts:146) … 13 more  
tests_for 6: `signedIn` (apps-web/causeway-web/test/app-tasks.test.ts:147), `request` (apps-web/causeway-web/test/app-tasks.test.ts:147), `pages` (apps-web/causeway-web/test/app-tasks.test.ts:146) … 3 more  
uses_types 1: `RequestFacts` (apps-web/causeway-web/test/app-tasks.test.ts:149)  
apps-web/causeway-web/test/app-tasks.test.ts:144-153
```typescript
  it("leaves the page to the shell when the flag is closed or nobody is signed in", async () => {
    const { tasks, asked } = fakeTasks(ROWS);
    const layer = await pages(tasks);
    expect(await layer.handle(request("/app/tasks"), signedIn([]))).toBeUndefined();
    expect(await layer.handle(request("/app/tasks"), ANONYMOUS_FACTS)).toBeUndefined();
    const open: RequestFacts = { ...ANONYMOUS_FACTS, gate: async () => true };
    expect(await layer.handle(request("/app/tasks"), open)).toBeUndefined();
    expect(await layer.handle(request("/app/tasks/7d0f6a4e"), signedIn())).toBeUndefined();
    expect(asked).toHaveLength(0);
  });
```

### 36. Test /app/tasks in Node::passes the assignee filter on and says whose list is empty
`typescript:apps-web/causeway-web/test/app-tasks.test.ts::/app/tasks in Node::passes the assignee filter on and says whose list is empty`  apps-web/causeway-web/test/app-tasks.test.ts:121-132  pkg:npm/@causeway/web  score 0.88  
`it("passes the assignee filter on and says whose list is empty", async () => { const { tasks, asked } = fakeTasks([]); const response = await (await pages(tasks)).handle( request("/app/tasks?assignee=team%3Adata"), signedIn(), ); expect(asked[0]?.assignee).toEqual(Option.some("team:data")); const ht…`  
calls 8: `signedIn` (apps-web/causeway-web/test/app-tasks.test.ts:125), `pages` (apps-web/causeway-web/test/app-tasks.test.ts:123), `fakeTasks` (apps-web/causeway-web/test/app-tasks.test.ts:122) … 5 more  
tests_for 8: `pages` (apps-web/causeway-web/test/app-tasks.test.ts:123), `signedIn` (apps-web/causeway-web/test/app-tasks.test.ts:125), `request` (apps-web/causeway-web/test/app-tasks.test.ts:124) … 5 more  
apps-web/causeway-web/test/app-tasks.test.ts:121-132
```typescript
  it("passes the assignee filter on and says whose list is empty", async () => {
    const { tasks, asked } = fakeTasks([]);
    const response = await (await pages(tasks)).handle(
      request("/app/tasks?assignee=team%3Adata"),
      signedIn(),
    );
    expect(asked[0]?.assignee).toEqual(Option.some("team:data"));
    const html = (await response?.text()) ?? "";
    expect(html).toContain("Open tasks assigned to team:data.");
    expect(html).toContain("No open tasks for team:data.");
    expect(html).not.toContain("<table");
  });
```

### 37. Test /app/tasks in Node::renders the open tasks inside the platform shell, as the signed-in person
`typescript:apps-web/causeway-web/test/app-tasks.test.ts::/app/tasks in Node::renders the open tasks inside the platform shell, as the signed-in person`  apps-web/causeway-web/test/app-tasks.test.ts:88-119  pkg:npm/@causeway/web  score 0.88  
`it("renders the open tasks inside the platform shell, as the signed-in person", async () => { const { tasks, asked } = fakeTasks(ROWS); const response = await (await pages(tasks)).handle(request("/app/tasks"), signedIn()); expect(response?.status).toBe(200); expect(response?.headers.get("cache-contr…`  
calls 8: `signedIn` (apps-web/causeway-web/test/app-tasks.test.ts:90), `request` (apps-web/causeway-web/test/app-tasks.test.ts:90), `fakeTasks` (apps-web/causeway-web/test/app-tasks.test.ts:89) … 5 more  
tests_for 8: `signedIn` (apps-web/causeway-web/test/app-tasks.test.ts:90), `request` (apps-web/causeway-web/test/app-tasks.test.ts:90), `pages` (apps-web/causeway-web/test/app-tasks.test.ts:90) … 5 more  
apps-web/causeway-web/test/app-tasks.test.ts:88-119
```typescript
  it("renders the open tasks inside the platform shell, as the signed-in person", async () => {
    const { tasks, asked } = fakeTasks(ROWS);
    const response = await (await pages(tasks)).handle(request("/app/tasks"), signedIn());
    expect(response?.status).toBe(200);
    expect(response?.headers.get("cache-control")).toBe("private, no-store");
    expect(asked).toEqual([{ bearer: "token-of-the-reader", assignee: Option.none() }]);

    const html = (await response?.text()) ?? "";
    expect(html.startsWith("<!doctype html>")).toBe(true);
    expect(html).toContain("<title>Tasks — Causeway</title>");
    expect(html).toContain("All open workflow tasks across every definition and assignee.");
    expect(html).toContain('href="/app/tasks/7d0f6a4e-2b9c-4f61-9a0e-0c1d2e3f4a5b"');
    expect(html).toContain('href="/app/catalog/reviews/1b2c3d4e-5f60-4718-8a9b-0c1d2e3f4a5c"');
    expect(html).toContain("form: contract-review@1");
    expect(html).toContain('<span class="pill pill--tide">team:governance</span>');
    expect(html).toContain('<span class="pill pill--lamp">andrew-dean</span>');
    expect(html).toContain("contract-approval@3");
    expect(html).toContain("3 hrs ago");
    expect(html).toContain("in 2 ds");
    expect(html).toContain("1 min ago");
    expect(html).toContain("—");
    // The chrome: Tasks is the highlighted entry, and the islands load with the nonce.
    expect(html).toMatch(/<a href="\/app\/tasks" title="Tasks" class="nav-item active">/);
    const nonce = /nonce-([A-Za-z0-9_-]+)/.exec(
      response?.headers.get("content-security-policy") ?? "",
    )?.[1];
    for (const tag of html.match(/<script\b[^>]*>/g) ?? []) {
      expect(tag).toContain(`nonce="${nonce}"`);
    }
    expect(html).toContain("<site-user-menu");
    expect(html).not.toContain("data-hk");
  });
```

### 38. Test /app/tasks in Node::says the read failed rather than showing an empty list
`typescript:apps-web/causeway-web/test/app-tasks.test.ts::/app/tasks in Node::says the read failed rather than showing an empty list`  apps-web/causeway-web/test/app-tasks.test.ts:134-142  pkg:npm/@causeway/web  score 0.88  
`it("says the read failed rather than showing an empty list", async () => { const { tasks } = fakeTasks("fails"); const response = await (await pages(tasks)).handle(request("/app/tasks"), signedIn()); expect(response?.status).toBe(200); const html = (await response?.text()) ?? ""; expect(html).toCont…`  
calls 8: `request` (apps-web/causeway-web/test/app-tasks.test.ts:136), `pages` (apps-web/causeway-web/test/app-tasks.test.ts:136), `signedIn` (apps-web/causeway-web/test/app-tasks.test.ts:136) … 5 more  
tests_for 8: `pages` (apps-web/causeway-web/test/app-tasks.test.ts:136), `request` (apps-web/causeway-web/test/app-tasks.test.ts:136), `signedIn` (apps-web/causeway-web/test/app-tasks.test.ts:136) … 5 more  
apps-web/causeway-web/test/app-tasks.test.ts:134-142
```typescript
  it("says the read failed rather than showing an empty list", async () => {
    const { tasks } = fakeTasks("fails");
    const response = await (await pages(tasks)).handle(request("/app/tasks"), signedIn());
    expect(response?.status).toBe(200);
    const html = (await response?.text()) ?? "";
    expect(html).toContain('role="alert"');
    expect(html).toContain("The open tasks could not be loaded.");
    expect(html).not.toContain("No open tasks");
  });
```

### 39. Function routes
`rust:apps/causeway-server/src/routes/app/govern.rs::routes`  apps/causeway-server/src/routes/app/govern.rs:10-74  pkg:cargo/causeway-server  score 0.87  via `task_detail` <- REFERENCES (apps/causeway-server/src/routes/app/govern.rs:72, 1.00)  
`pub(super) fn routes() -> Router<AppState>`  
callers 1: `routes` (apps/causeway-server/src/routes/app.rs:20)  
references 37: `task_detail` (apps/causeway-server/src/routes/app/govern.rs:72), `rights_preview` (apps/causeway-server/src/routes/app/govern.rs:50), `rights_new` (apps/causeway-server/src/routes/app/govern.rs:48) … 34 more  
uses_types 1: `AppState` (apps/causeway-server/src/routes/app/govern.rs:10)  
apps/causeway-server/src/routes/app/govern.rs:10-74
```rust
pub(super) fn routes() -> Router<AppState> {
    Router::new()
        .route("/glossary", get(pages::glossary))
        // Identity-keyed entry points: a search document or semantic link
        // carries the resource UUID; the server resolves it to the owning
        // page so a rename never orphans the link.
        .route("/glossary/id/{term_id}", get(pages::glossary_term_by_id))
        .route(
            "/glossary/concepts/id/{concept_id}",
            get(pages::glossary_concept_by_id),
        )
        .route(
            "/glossary/concepts/{key}",
            get(pages::glossary_concept_detail),
        )
        .route("/glossary/{key}", get(pages::glossary_term_detail))
        .route("/policies", get(pages::policies))
        .route("/issues", get(pages::issues))
        .route("/placements", get(pages::placements))
        .route("/placements/proposals", post(pages::placements_propose))
        .route(
            "/placements/proposals/{proposal_id}",
            post(pages::placements_decide),
        )
        .route(
            "/policies/workbench",
            get(pages::policy_workbench).post(pages::policy_workbench_analyze),
        )
        .route(
            "/policies/workbench/extract",
            axum::routing::post(pages::policy_workbench_extract),
        )
        .route(
            "/policies/workbench/emit",
            axum::routing::post(pages::policy_workbench_emit),
        )
        .route("/policies/{name}", get(pages::policy_detail))
        .route("/rights", get(pages::rights_index))
        .route("/rights/new", get(pages::rights_new))
        .route("/rights/import", get(pages::rights_import))
        .route("/rights/preview", post(pages::rights_preview))
        .route("/rights/agreements/{slug}", get(pages::rights_agreement))
        .route(
            "/rights/agreements/{slug}/export",
            get(pages::rights_export),
        )
        .route(
            "/rights/agreements/{slug}/correct/{term}",
            get(pages::rights_correct),
        )
        .route("/contracts", get(pages::contracts))
        .route(
            "/contracts/{id}/versions/{version}/review",
            axum::routing::post(pages::contract_review_ensure),
        )
        .route(
            "/contracts/reviews/{task_id}",
            get(pages::contract_review).post(pages::contract_review_submit),
        )
        .route("/contracts/{id}", get(pages::contract_detail))
        .route("/workflows", get(pages::workflows))
        .route("/tasks", get(pages::tasks))
        .route("/tasks/{id}", get(pages::task_detail))
        .route("/tasks/{id}/complete", post(pages::task_complete))
}
```

### 40. Function task_breadcrumbs
`rust:apps/causeway-server/src/routes/pages/tasks/detail.rs::task_breadcrumbs`  apps/causeway-server/src/routes/pages/tasks/detail.rs:116-122  pkg:cargo/causeway-server  score 0.87  via `task_detail` -> CALLS (apps/causeway-server/src/routes/pages/tasks/detail.rs:44, 1.00)  
`fn task_breadcrumbs(id: &str) -> Vec<Breadcrumb>`  
callers 3: `render_load_error` (apps/causeway-server/src/routes/pages/tasks/detail.rs:85), `not_found` (apps/causeway-server/src/routes/pages/tasks/detail.rs:103), `task_detail` (apps/causeway-server/src/routes/pages/tasks/detail.rs:44)  
calls 3: `Breadcrumb::link` (apps/causeway-server/src/routes/pages/tasks/detail.rs:118), `Breadcrumb::link` (apps/causeway-server/src/routes/pages/tasks/detail.rs:119), `Breadcrumb::current` (apps/causeway-server/src/routes/pages/tasks/detail.rs:120)  
uses_types 1: `Breadcrumb` (apps/causeway-server/src/routes/pages/tasks/detail.rs:116)  
apps/causeway-server/src/routes/pages/tasks/detail.rs:116-122
```rust
fn task_breadcrumbs(id: &str) -> Vec<Breadcrumb> {
    vec![
        Breadcrumb::link("Work", "#"),
        Breadcrumb::link("Tasks", "/app/tasks"),
        Breadcrumb::current(id),
    ]
}
```

### 41. Struct TasksQuery
`rust:apps/causeway-server/src/routes/pages/tasks/list.rs::TasksQuery`  apps/causeway-server/src/routes/pages/tasks/list.rs:17-19  pkg:cargo/causeway-server  score 0.84  via `tasks` -> USES_TYPE (apps/causeway-server/src/routes/pages/tasks/list.rs:30, 1.00)  
`pub(crate) struct TasksQuery`  
members 1  
used_by 2: `tasks` (apps/causeway-server/src/routes/pages/tasks/list.rs:30), `TasksQuery#impl` (apps/causeway-server/src/routes/pages/tasks/list.rs:21-25)  
apps/causeway-server/src/routes/pages/tasks/list.rs:17-19
```rust
pub(crate) struct TasksQuery {
    assignee: Option<String>,
}
```

### 42. Function Row
`tsx:apps-web/causeway-web/src/pages/routes/Tasks.tsx::Row`  apps-web/causeway-web/src/pages/routes/Tasks.tsx:37-70  pkg:npm/@causeway/web  score 0.84  via `taskHref` <- CALLS (apps-web/causeway-web/src/pages/routes/Tasks.tsx:42, 1.00)  
`function Row(props: { readonly task: TaskRow; readonly now: Date })`  
calls 3: `relativeTime` (apps-web/causeway-web/src/pages/routes/Tasks.tsx:61), `taskHref` (apps-web/causeway-web/src/pages/routes/Tasks.tsx:42), `relativeTime` (apps-web/causeway-web/src/pages/routes/Tasks.tsx:65)  
uses_types 1: `TaskRow` (apps-web/causeway-web/src/pages/routes/Tasks.tsx:37)  
apps-web/causeway-web/src/pages/routes/Tasks.tsx:37-70
```tsx
function Row(props: { readonly task: TaskRow; readonly now: Date }) {
  const task = props.task;
  return (
    <tr>
      <td>
        <a class="title" href={taskHref(task)}>
          {task.title}
        </a>
        {Option.match(task.form, {
          onNone: () => null,
          onSome: (form) => <div class="sub">form: {form}</div>,
        })}
      </td>
      <td>
        <span class={task.assignee.startsWith("team:") ? "pill pill--tide" : "pill pill--lamp"}>
          {task.assignee}
        </span>
      </td>
      <td class="muted">
        {task.definition_name}@{task.definition_version}
      </td>
      <td>
        <code class="muted">{task.step}</code>
      </td>
      <td class="muted">{relativeTime(task.created_at, props.now)}</td>
      <td class="muted">
        {Option.match(task.due_at, {
          onNone: () => "—",
          onSome: (due) => relativeTime(due, props.now),
        })}
      </td>
    </tr>
  );
}
```

### 43. Function request
`typescript:apps-web/causeway-web/test/app-tasks.test.ts::request`  apps-web/causeway-web/test/app-tasks.test.ts:85  pkg:npm/@causeway/web  score 0.83  via `/app/tasks in Node::leaves the page to the shell when the flag is closed or nobody is signed in` <- TESTED_BY (apps-web/causeway-web/test/app-tasks.test.ts:147, 1.00)  
`request = (path: string) => new Request(new URL(path, "http://edge.test"))`  
callers 10: `/app/tasks in Node::leaves the page to the shell when the flag is closed or nobody is signed in` (apps-web/causeway-web/test/app-tasks.test.ts:151), `/app/tasks in Node::leaves the page to the shell when the flag is closed or nobody is signed in` (apps-web/causeway-web/test/app-tasks.test.ts:150), `/app/tasks in Node::leaves the page to the shell when the flag is closed or nobody is signed in` (apps-web/causeway-web/test/app-tasks.test.ts:147) … 7 more  
tests 5: `/app/tasks in Node::renders the open tasks inside the platform shell, as the signed-in person` (apps-web/causeway-web/test/app-tasks.test.ts:90), `/app/tasks in Node::says the read failed rather than showing an empty list` (apps-web/causeway-web/test/app-tasks.test.ts:136), `/app/tasks in Node::passes the assignee filter on and says whose list is empty` (apps-web/causeway-web/test/app-tasks.test.ts:124) … 2 more  
apps-web/causeway-web/test/app-tasks.test.ts:85-85
```typescript
const request = (path: string) => new Request(new URL(path, "http://edge.test"));
```

### 44. Function complete_task_from_form
`rust:apps/causeway-server/src/workflow_tasks/page.rs::complete_task_from_form`  apps/causeway-server/src/workflow_tasks/page.rs:66-110  pkg:cargo/causeway-server  score 0.82  via `TaskPageContext` <- USES_TYPE (apps/causeway-server/src/workflow_tasks/page.rs:67, 1.00)  
`pub(crate) async fn complete_task_from_form( context: &TaskPageContext<'_>, id: &str, body: Vec<(String, String)>, ) -> TaskCompletionResult`  
callers 1: `task_complete` (apps/causeway-server/src/routes/pages/tasks/complete.rs:46)  
calls 6: `task_signal_payload` (apps/causeway-server/src/workflow_tasks/page.rs:93), `TasksCommand::signal_task` (apps/causeway-server/src/workflow_tasks/page.rs:104), `MockWorkflowReadRepo::signal_task` (apps/causeway-server/src/workflow_tasks/page.rs:104) … 3 more  
importers 1: `apps/causeway-server/src/workflow_tasks.rs` (apps/causeway-server/src/workflow_tasks.rs:9-12)  
references 2: `TaskCompletionResult::Completed` (apps/causeway-server/src/workflow_tasks/page.rs:109), `ports` (apps/causeway-server/src/workflow_tasks/page.rs:98)  
uses_types 3: `TaskCompletionResult` (apps/causeway-server/src/workflow_tasks/page.rs:70), `TaskPageContext` (apps/causeway-server/src/workflow_tasks/page.rs:67), `TaskSignalCommand` (apps/causeway-server/src/workflow_tasks/page.rs:98)  
apps/causeway-server/src/workflow_tasks/page.rs:66-110
```rust
pub(crate) async fn complete_task_from_form(
    context: &TaskPageContext<'_>,
    id: &str,
    body: Vec<(String, String)>,
) -> TaskCompletionResult {
    let task_uuid = match uuid::Uuid::parse_str(id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return TaskCompletionResult::Rejected(
                "Malformed task id - expected a UUID.".to_owned(),
            );
        }
    };

    let loaded = match context.tasks_read.load_task_with_form(task_uuid).await {
        Ok(Some(loaded)) => loaded,
        Ok(None) => {
            return TaskCompletionResult::Rejected(
                "Task not found - it may have already been completed.".to_owned(),
            );
        }
        Err(error) => {
            tracing::warn!(error = %error, task_id = %id, "complete: failed to load task");
            return TaskCompletionResult::Rejected("Failed to load task from database.".to_owned());
        }
    };

    let task_payload = causeway_server_web::views::platform::tasks::task_signal_payload(
        loaded.form.as_ref(),
        body,
    );

    let command = workflows_component::ports::TaskSignalCommand {
        task_id: loaded.task.id,
        workflow_instance_id: loaded.instance_id,
        signal: task_payload.signal,
        payload: task_payload.payload,
    };
    if let Err(error) = context.tasks_command.signal_task(command).await {
        tracing::warn!(error = %error, task_id = %id, "complete: workflow signal failed");
        return TaskCompletionResult::Rejected(format!("Workflow signal failed: {error}"));
    }

    TaskCompletionResult::Completed
}
```

### 45. Function fakeTasks
`typescript:apps-web/causeway-web/test/app-tasks.test.ts::fakeTasks`  apps-web/causeway-web/test/app-tasks.test.ts:52-65  pkg:npm/@causeway/web  score 0.82  via `/app/tasks in Node::leaves the page to the shell when the flag is closed or nobody is signed in` <- TESTED_BY (apps-web/causeway-web/test/app-tasks.test.ts:145, 1.00)  
`function fakeTasks(answer: readonly TaskRow[] | "fails")`  
A `WorkflowService` that answers `answer` and records what it was asked.  
callers 4: `/app/tasks in Node::leaves the page to the shell when the flag is closed or nobody is signed in` (apps-web/causeway-web/test/app-tasks.test.ts:145), `/app/tasks in Node::passes the assignee filter on and says whose list is empty` (apps-web/causeway-web/test/app-tasks.test.ts:122), `/app/tasks in Node::renders the open tasks inside the platform shell, as the signed-in person` (apps-web/causeway-web/test/app-tasks.test.ts:89) … 1 more  
tests 4: `/app/tasks in Node::renders the open tasks inside the platform shell, as the signed-in person` (apps-web/causeway-web/test/app-tasks.test.ts:89), `/app/tasks in Node::passes the assignee filter on and says whose list is empty` (apps-web/causeway-web/test/app-tasks.test.ts:122), `/app/tasks in Node::says the read failed rather than showing an empty list` (apps-web/causeway-web/test/app-tasks.test.ts:135) … 1 more  
uses_types 2: `OpenTasks` (apps-web/causeway-web/test/app-tasks.test.ts:54), `ShellRpcError` (apps-web/causeway-web/test/app-tasks.test.ts:59)  
apps-web/causeway-web/test/app-tasks.test.ts:52-65
```typescript
function fakeTasks(answer: readonly TaskRow[] | "fails") {
  const asked: { bearer: string; assignee: Option.Option<string> }[] = [];
  const tasks: OpenTasks = {
    list: (bearer, assignee) => {
      asked.push({ bearer, assignee });
      return answer === "fails"
        ? Effect.fail(
            new ShellRpcError({ method: "ListOpenTasks", code: "unavailable", reason: "down" }),
          )
        : Effect.succeed(answer);
    },
  };
  return { tasks, asked };
}
```

### 46. Function signedIn
`typescript:apps-web/causeway-web/test/app-tasks.test.ts::signedIn`  apps-web/causeway-web/test/app-tasks.test.ts:78-83  pkg:npm/@causeway/web  score 0.78  via `/app/tasks in Node::leaves the page to the shell when the flag is closed or nobody is signed in` <- TESTED_BY (apps-web/causeway-web/test/app-tasks.test.ts:147, 1.00)  
`signedIn = (flags: readonly string[] = [NODE_APP_TASKS]): RequestFacts => ({ caller: { kind: "signed-in", subject: "fixture-tasks-reader" }, gate: async (key) => flags.includes(key), flag: async (key, fallback) => (flags.includes(key) ? true : fallback), bearer: Option.some(AccessToken("token-of-the…`  
callers 9: `/app/tasks in Node::renders the open tasks inside the platform shell, as the signed-in person` (apps-web/causeway-web/test/app-tasks.test.ts:90), `/app/tasks in Node::passes the assignee filter on and says whose list is empty` (apps-web/causeway-web/test/app-tasks.test.ts:125), `/app/tasks in Node::leaves the page to the shell when the flag is closed or nobody is signed in` (apps-web/causeway-web/test/app-tasks.test.ts:147) … 6 more  
calls 1: `AccessToken` (apps-web/causeway-web/test/app-tasks.test.ts:82)  
tests 4: `/app/tasks in Node::renders the open tasks inside the platform shell, as the signed-in person` (apps-web/causeway-web/test/app-tasks.test.ts:90), `/app/tasks in Node::passes the assignee filter on and says whose list is empty` (apps-web/causeway-web/test/app-tasks.test.ts:125), `/app/tasks in Node::leaves the page to the shell when the flag is closed or nobody is signed in` (apps-web/causeway-web/test/app-tasks.test.ts:147) … 1 more  
uses_types 1: `RequestFacts` (apps-web/causeway-web/test/app-tasks.test.ts:78)  
apps-web/causeway-web/test/app-tasks.test.ts:78-83
```typescript
const signedIn = (flags: readonly string[] = [NODE_APP_TASKS]): RequestFacts => ({
  caller: { kind: "signed-in", subject: "fixture-tasks-reader" },
  gate: async (key) => flags.includes(key),
  flag: async (key, fallback) => (flags.includes(key) ? true : fallback),
  bearer: Option.some(AccessToken("token-of-the-reader")),
});
```

### 47. Method Breadcrumb::current
`rust:apps/causeway-server-web/src/nav.rs::Breadcrumb::current`  apps/causeway-server-web/src/nav.rs:22-27  pkg:cargo/causeway-server-web  score 0.78  via `tasks` -> CALLS (apps/causeway-server/src/routes/pages/tasks/list.rs:36, 1.00)  
`pub fn current(label: impl Into<String>) -> Self`  
callers 53: `render_catalog_review_page` (apps/causeway-server/src/routes/pages/catalog/review.rs:57), `glossary_term_detail` (apps/causeway-server/src/routes/pages/governance/glossary/term.rs:38), `policy_detail` (apps/causeway-server/src/routes/pages/governance/policies.rs:66) … 50 more  
apps/causeway-server-web/src/nav.rs:22-27
```rust
    pub fn current(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            href: None,
        }
    }
```

### 48. File apps/causeway-server/src/routes/pages/refused.rs
`file:apps/causeway-server/src/routes/pages/refused.rs`  apps/causeway-server/src/routes/pages/refused.rs  pkg:cargo/causeway-server  score 0.75  via `Section#impl` <- CONTAINS (apps/causeway-server/src/routes/pages/refused.rs:23-125, 1.00)  
members 3  
documents 1: `docs/DESIGN-LOG.md` (docs/DESIGN-LOG.md:106)  
imports 5: `cargo/axum` (apps/causeway-server/src/routes/pages/refused.rs:4), `Breadcrumb` (apps/causeway-server/src/routes/pages/refused.rs:6), `AppState` (apps/causeway-server/src/routes/pages/refused.rs:8) … 2 more  

### 49. Method TasksQuery::assignee
`rust:apps/causeway-server/src/routes/pages/tasks/list.rs::TasksQuery::assignee`  apps/causeway-server/src/routes/pages/tasks/list.rs:22-24  pkg:cargo/causeway-server  score 0.74  via `tasks` -> CALLS (apps/causeway-server/src/routes/pages/tasks/list.rs:47, 1.00)  
`fn assignee(&self) -> Option<&str>`  
callers 1: `tasks` (apps/causeway-server/src/routes/pages/tasks/list.rs:47)  
apps/causeway-server/src/routes/pages/tasks/list.rs:22-24
```rust
    fn assignee(&self) -> Option<&str> {
        self.assignee.as_deref().filter(|value| !value.is_empty())
    }
```

### 50. File apps/causeway-server/src/routes/pages/tasks/complete.rs
`file:apps/causeway-server/src/routes/pages/tasks/complete.rs`  apps/causeway-server/src/routes/pages/tasks/complete.rs  pkg:cargo/causeway-server  score 0.74  via `task_complete` <- CONTAINS (apps/causeway-server/src/routes/pages/tasks/complete.rs:18-54, 1.00)  
members 2  
documents 1: `docs/DESIGN-LOG.md` (docs/DESIGN-LOG.md:1186)  
imports 8: `refused` (apps/causeway-server/src/routes/pages/tasks/complete.rs:12), `cargo/axum` (apps/causeway-server/src/routes/pages/tasks/complete.rs:4), `cargo/causeway-server-web` (apps/causeway-server/src/routes/pages/tasks/complete.rs:8) … 5 more  

### 51. Impl Section#impl
`rust:apps/causeway-server/src/routes/pages/refused.rs::Section#impl`  apps/causeway-server/src/routes/pages/refused.rs:23-125  pkg:cargo/causeway-server  score 0.72  via `Section::TASKS` <- DEFINES (apps/causeway-server/src/routes/pages/refused.rs:100-106, 1.00)  
`impl Section`  
members 12  
uses_types 1: `Section` (apps/causeway-server/src/routes/pages/refused.rs:23-125)  

### 52. Function tasks
`rust:apps/causeway-server-web/src/views/platform/tasks/list.rs::tasks`  apps/causeway-server-web/src/views/platform/tasks/list.rs:15-52  pkg:cargo/causeway-server-web  score 0.69  via `tasks` -> CALLS (apps/causeway-server/src/routes/pages/tasks/list.rs:57, 1.00)  
`pub fn tasks(rows: &[TaskRow], assignee_filter: Option<&str>, now: DateTime<Utc>) -> Markup`  
callers 1: `tasks` (apps/causeway-server/src/routes/pages/tasks/list.rs:57)  
calls 1: `task_row` (apps/causeway-server-web/src/views/platform/tasks/list.rs:45)  
importers 1: `apps/causeway-server-web/src/views/platform/tasks.rs` (apps/causeway-server-web/src/views/platform/tasks.rs:10)  
uses_types 1: `TaskRow` (apps/causeway-server-web/src/views/platform/tasks/list.rs:15)  
apps/causeway-server-web/src/views/platform/tasks/list.rs:15-52
```rust
pub fn tasks(rows: &[TaskRow], assignee_filter: Option<&str>, now: DateTime<Utc>) -> Markup {
    let lead = match assignee_filter {
        Some(a) => format!("Open tasks assigned to {a}."),
        None => "All open workflow tasks across every definition and assignee.".to_owned(),
    };
    html! {
        div style=(PAGE_WRAPPER) {
            (page_head("Tasks", Some(&lead)))

            @if rows.is_empty() {
                div style=(CARD) {
                    p style="margin:0;font-size:13px;color:var(--fg-3);" {
                        @match assignee_filter {
                            Some(a) => (format!("No open tasks for {a}. New tasks arrive when a human_task node opens on a running workflow.")),
                            None    => ("No open tasks. New tasks arrive when a human_task node opens on a running workflow."),
                        }
                    }
                }
            } @else {
                div style=(CARD) {
                    table style=(TABLE) {
                        thead { tr {
                            th style=(TH) { "Task" }
                            th style=(TH) { "Assignee" }
                            th style=(TH) { "Workflow" }
                            th style=(TH) { "Step" }
                            th style=(TH) { "Created" }
                            th style=(TH) { "Due" }
                        }}
                        tbody {
                            @for r in rows { (task_row(r, now)) }
                        }
                    }
                }
            }
        }
    }
}
```

### 53. Function tasks_realtime_script
`rust:apps/causeway-server-web/src/views/platform/tasks.rs::tasks_realtime_script`  apps/causeway-server-web/src/views/platform/tasks.rs:65-87  pkg:cargo/causeway-server-web  score 0.68  via `tasks` -> CALLS (apps/causeway-server/src/routes/pages/tasks/list.rs:58, 1.00)  
`pub fn tasks_realtime_script() -> Markup`  
callers 2: `tests::tasks_realtime_script_uses_task_event_model_signal` (apps/causeway-server-web/src/views/platform/tasks.rs:95), `tasks` (apps/causeway-server/src/routes/pages/tasks/list.rs:58)  
calls 3: `inline_script` (apps/causeway-server-web/src/views/platform/tasks.rs:86), `TaskEvent::signal_name` (apps/causeway-server-web/src/views/platform/tasks.rs:84), `TaskCommand::signal_name` (apps/causeway-server-web/src/views/platform/tasks.rs:84)  
imports 1: `cargo/maud` (apps/causeway-server-web/src/views/platform/tasks.rs:66)  
references 1: `TaskEvent::OpenTasksChanged` (apps/causeway-server-web/src/views/platform/tasks.rs:68)  
tests 1: `tests::tasks_realtime_script_uses_task_event_model_signal` (apps/causeway-server-web/src/views/platform/tasks.rs:95)  
apps/causeway-server-web/src/views/platform/tasks.rs:65-87
```rust
pub fn tasks_realtime_script() -> Markup {
    use maud::PreEscaped;

    let event = model::TaskEvent::OpenTasksChanged;
    let script = format!(
        r#"
            (function() {{
              function init() {{
                var bus = window.__bus;
                if (!bus) {{ setTimeout(init, 50); return; }}
                var skipNext = bus.get('{tasks_changed_signal}') !== undefined;
                bus.subscribe('{tasks_changed_signal}', function() {{
                  if (skipNext) {{ skipNext = false; return; }}
                  location.reload();
                }});
              }}
              init();
            }})();
            "#,
        tasks_changed_signal = event.signal_name(),
    );
    crate::csp::inline_script(PreEscaped(script))
}
```

### 54. File apps-web/causeway-web/src/pages/data/tasks.ts
`file:apps-web/causeway-web/src/pages/data/tasks.ts`  apps-web/causeway-web/src/pages/data/tasks.ts  pkg:npm/@causeway/web  score 0.65  via `TaskRow` <- CONTAINS (apps-web/causeway-web/src/pages/data/tasks.ts:26, 1.00)  
members 3  
documents 1: `docs/DESIGN-LOG.md` (docs/DESIGN-LOG.md:1878)  
importers 5: `apps-web/causeway-web/test/app-tasks.test.ts` (apps-web/causeway-web/test/app-tasks.test.ts:11), `apps-web/causeway-web/src/pages/routes/Tasks.tsx` (apps-web/causeway-web/src/pages/routes/Tasks.tsx:9), `apps-web/causeway-web/test/app-tasks-routing.test.ts` (apps-web/causeway-web/test/app-tasks-routing.test.ts:10) … 2 more  
imports 5: `apps-web/causeway-web/src/pages/shell-rpc.ts` (apps-web/causeway-web/src/pages/data/tasks.ts:10), `ShellRpc` (apps-web/causeway-web/src/pages/data/tasks.ts:10), `AccessToken` (apps-web/causeway-web/src/pages/data/tasks.ts:10) … 2 more  

### 55. Function render
`rust:apps/causeway-server/src/routes/pages.rs::render`  apps/causeway-server/src/routes/pages.rs:275-299  pkg:cargo/causeway-server  score 0.63  via `tasks` -> CALLS (apps/causeway-server/src/routes/pages/tasks/list.rs:50, 1.00)  
`async fn render( state: &AppState, title: &str, active_nav: &'static str, breadcrumbs: Vec<Breadcrumb>, tabs: Vec<TabDef>, active_tab: &str, content: Markup, extra_scripts: Option<Markup>, ) -> Html<String>`  
callers 44: `rights_agreement` (apps/causeway-server/src/routes/pages/governance/rights.rs:63), `stuck_work_record` (apps/causeway-server/src/routes/pages/metadata_ops/stuck_work.rs:244), `marketplace` (apps/causeway-server/src/routes/pages/marketplace/list.rs:27) … 41 more  
calls 4: `platform_shell` (apps/causeway-server/src/routes/pages.rs:286), `hydration` (apps/causeway-server/src/routes/pages.rs:285), `FeedSlug::into_string` (apps/causeway-server/src/routes/pages.rs:298) … 1 more  
importers 26: `apps/causeway-server/src/routes/pages/explorer/feed.rs` (apps/causeway-server/src/routes/pages/explorer/feed.rs:9), `apps/causeway-server/src/routes/pages/contracts/list.rs` (apps/causeway-server/src/routes/pages/contracts/list.rs:18), `apps/causeway-server/src/routes/pages/catalog/review.rs` (apps/causeway-server/src/routes/pages/catalog/review.rs:18) … 23 more  
uses_types 4: `Breadcrumb` (apps/causeway-server/src/routes/pages.rs:279), `PlatformArgs` (apps/causeway-server/src/routes/pages.rs:286), `AppState` (apps/causeway-server/src/routes/pages.rs:276) … 1 more  
apps/causeway-server/src/routes/pages.rs:275-299
```rust
async fn render(
    state: &AppState,
    title: &str,
    active_nav: &'static str,
    breadcrumbs: Vec<Breadcrumb>,
    tabs: Vec<TabDef>,
    active_tab: &str,
    content: Markup,
    extra_scripts: Option<Markup>,
) -> Html<String> {
    let hydration_json = hydration(state, active_nav).await;
    let markup = platform_shell(PlatformArgs {
        title,
        active_nav,
        hydration_json: &hydration_json,
        breadcrumbs: &breadcrumbs,
        tabs: &tabs,
        active_tab,
        content,
        extra_scripts,
        island_scripts: None,
        preload_pdfjs: false,
    });
    Html(markup.into_string())
}
```

### 56. Function render_load_error
`rust:apps/causeway-server/src/routes/pages/tasks/detail.rs::render_load_error`  apps/causeway-server/src/routes/pages/tasks/detail.rs:80-96  pkg:cargo/causeway-server  score 0.62  via `render_task_detail` -> CALLS (apps/causeway-server/src/routes/pages/tasks/detail.rs:59, 1.00)  
`async fn render_load_error(state: &AppState, id: &str) -> Html<String>`  
callers 1: `render_task_detail` (apps/causeway-server/src/routes/pages/tasks/detail.rs:59)  
calls 3: `task_breadcrumbs` (apps/causeway-server/src/routes/pages/tasks/detail.rs:85), `render` (apps/causeway-server/src/routes/pages/tasks/detail.rs:88), `render` (apps/causeway-server/src/routes/pages/tasks/detail.rs:81)  
uses_types 1: `AppState` (apps/causeway-server/src/routes/pages/tasks/detail.rs:80)  
apps/causeway-server/src/routes/pages/tasks/detail.rs:80-96
```rust
async fn render_load_error(state: &AppState, id: &str) -> Html<String> {
    render(
        state,
        "Task",
        "tasks",
        task_breadcrumbs(id),
        vec![],
        "",
        placeholder::render(
            "Task failed to load",
            "The task row or its form could not be read from Postgres. \
             Check server logs and database state.",
        ),
        None,
    )
    .await
}
```

### 57. Function not_found
`rust:apps/causeway-server/src/routes/pages/tasks/detail.rs::not_found`  apps/causeway-server/src/routes/pages/tasks/detail.rs:98-114  pkg:cargo/causeway-server  score 0.58  via `render_task_detail` -> CALLS (apps/causeway-server/src/routes/pages/tasks/detail.rs:57, 1.00)  
`async fn not_found(state: &AppState, id: &str) -> Html<String>`  
callers 8: `render_task_detail` (apps/causeway-server/src/routes/pages/tasks/detail.rs:57), `MetadataAdmin::get_observation` (apps/causeway-server/src/metadata_admin/handlers.rs:92), `MetadataAdmin::get_claim` (apps/causeway-server/src/metadata_admin/handlers.rs:122) … 5 more  
calls 3: `task_breadcrumbs` (apps/causeway-server/src/routes/pages/tasks/detail.rs:103), `render` (apps/causeway-server/src/routes/pages/tasks/detail.rs:99), `render` (apps/causeway-server/src/routes/pages/tasks/detail.rs:106)  
uses_types 1: `AppState` (apps/causeway-server/src/routes/pages/tasks/detail.rs:98)  
apps/causeway-server/src/routes/pages/tasks/detail.rs:98-114
```rust
async fn not_found(state: &AppState, id: &str) -> Html<String> {
    render(
        state,
        "Task not found",
        "tasks",
        task_breadcrumbs(id),
        vec![],
        "",
        placeholder::render(
            "Task not found",
            "No open task with that id exists. It may have been completed \
             or cancelled.",
        ),
        None,
    )
    .await
}
```

### 58. Function decide_all
`rust:apps/causeway-server/src/routes/pages/twin.rs::decide_all`  apps/causeway-server/src/routes/pages/twin.rs:56-65  pkg:cargo/causeway-server  score 0.56  via `task_complete` -> CALLS (apps/causeway-server/src/routes/pages/tasks/complete.rs:27, 1.00)  
`pub(super) fn decide_all( state: &AppState, caller: &BrowserCaller, twins: &[&str], ) -> Result<Vec<RpcGrant>, PageRefusal>`  
Every one of `twins`, for a page that makes each call its twins do. (doc continues, 116 bytes)  
callers 1: `task_complete` (apps/causeway-server/src/routes/pages/tasks/complete.rs:27)  
calls 1: `decide` (apps/causeway-server/src/routes/pages/twin.rs:63)  
uses_types 4: `PageRefusal` (apps/causeway-server/src/routes/pages/twin.rs:60), `RpcGrant` (apps/causeway-server/src/routes/pages/twin.rs:60), `BrowserCaller` (apps/causeway-server/src/routes/pages/twin.rs:58) … 1 more  
apps/causeway-server/src/routes/pages/twin.rs:56-65
```rust
pub(super) fn decide_all(
    state: &AppState,
    caller: &BrowserCaller,
    twins: &[&str],
) -> Result<Vec<RpcGrant>, PageRefusal> {
    twins
        .iter()
        .map(|twin| decide(state, caller, twin))
        .collect()
}
```

### 59. Method TaskDetailQuery::error
`rust:apps/causeway-server/src/routes/pages/tasks/detail.rs::TaskDetailQuery::error`  apps/causeway-server/src/routes/pages/tasks/detail.rs:22-24  pkg:cargo/causeway-server  score 0.55  via `render_task_detail` -> CALLS (apps/causeway-server/src/routes/pages/tasks/detail.rs:62, 1.00)  
`fn error(&self) -> Option<&str>`  
callers 1: `render_task_detail` (apps/causeway-server/src/routes/pages/tasks/detail.rs:62)  
apps/causeway-server/src/routes/pages/tasks/detail.rs:22-24
```rust
    fn error(&self) -> Option<&str> {
        self.err.as_deref().filter(|value| !value.is_empty())
    }
```

### 60. File apps-web/causeway-web/test/app-tasks.test.ts
`file:apps-web/causeway-web/test/app-tasks.test.ts`  apps-web/causeway-web/test/app-tasks.test.ts  pkg:npm/@causeway/web  score 0.52  via `/app/tasks in Node` <- CONTAINS (apps-web/causeway-web/test/app-tasks.test.ts:87-154, 1.00)  
members 6  
documents 1: `docs/DESIGN-LOG.md` (docs/DESIGN-LOG.md:1888)  
imports 14: `npm/@causeway/content-core` (apps-web/causeway-web/test/app-tasks.test.ts:8), `apps-web/causeway-web/src/pages/data/tasks.ts` (apps-web/causeway-web/test/app-tasks.test.ts:11), `npm/vitest` (apps-web/causeway-web/test/app-tasks.test.ts:10) … 11 more  

### 61. Const SIGNAL_TASK
`rust:apps/causeway-server/src/routes/pages/twin.rs::SIGNAL_TASK`  apps/causeway-server/src/routes/pages/twin.rs:26  pkg:cargo/causeway-server  score 0.52  via `task_complete` -> REFERENCES (apps/causeway-server/src/routes/pages/tasks/complete.rs:30, 1.00)  
`pub(super) const SIGNAL_TASK: &str = "causeway.workflow.v1.WorkflowService/SignalTask";`  
The completion form's command twin.  
referenced_by 1: `task_complete` (apps/causeway-server/src/routes/pages/tasks/complete.rs:30)  
apps/causeway-server/src/routes/pages/twin.rs:26-26
```rust
pub(super) const SIGNAL_TASK: &str = "causeway.workflow.v1.WorkflowService/SignalTask";
```

### 62. Function task_wizard
`rust:apps/causeway-server-web/src/views/platform/tasks/wizard.rs::task_wizard`  apps/causeway-server-web/src/views/platform/tasks/wizard.rs:11-50  pkg:cargo/causeway-server-web  score 0.50  via `render_task_detail` -> CALLS (apps/causeway-server/src/routes/pages/tasks/detail.rs:74, 1.00)  
`pub fn task_wizard(loaded: &LoadedTask, error: Option<&str>) -> Markup`  
callers 1: `render_task_detail` (apps/causeway-server/src/routes/pages/tasks/detail.rs:74)  
calls 7: `missing_card` (apps/causeway-server-web/src/views/platform/tasks/wizard.rs:44), `render_form` (apps/causeway-server-web/src/views/platform/tasks/wizard.rs:43), `submit_row` (apps/causeway-server-web/src/views/platform/tasks/wizard.rs:46) … 4 more  
importers 1: `apps/causeway-server-web/src/views/platform/tasks.rs` (apps/causeway-server-web/src/views/platform/tasks.rs:14)  
uses_types 1: `LoadedTask` (apps/causeway-server-web/src/views/platform/tasks/wizard.rs:11)  

### 63. Trait TasksRead
`rust:component/orchestration/workflows/src/ports.rs::TasksRead`  component/orchestration/workflows/src/ports.rs:222-235  pkg:cargo/workflows-component  score 0.48  via `TaskPageContext` -> USES_TYPE (apps/causeway-server/src/workflow_tasks/page.rs:9, 1.00)  
`pub trait TasksRead: Send + Sync + 'static`  
Human workflow task read surface.  
members 2  
implementations 3: `PgWorkflowReadRepo#impl(TasksRead)` (component/orchestration/workflows/src/impls/read_repo.rs:98), `RemoteWorkflowClient#impl(TasksRead)` (component/orchestration/workflows/src/remote.rs:74), `MockWorkflowReadRepo#impl(TasksRead)` (component/orchestration/workflows/src/impls/read_repo.rs:641)  
importers 8: `component/orchestration/workflows/src/remote.rs` (component/orchestration/workflows/src/remote.rs:18-22), `apps/causeway-server/src/composition/local_services/ports.rs` (apps/causeway-server/src/composition/local_services/ports.rs:19-21), `apps/causeway-server/src/composition/remote_services/tests.rs` (apps/causeway-server/src/composition/remote_services/tests.rs:75) … 5 more  
used_by 11: `ServicePorts` (apps/causeway-server/src/composition/local_services/ports.rs:70), `WorkflowServicePorts` (component/orchestration/workflows/src/rpc.rs:34), `MockWorkflowReadRepo#impl(TasksRead)` (component/orchestration/workflows/src/impls/read_repo.rs:641) … 8 more  
uses_types 2: `LoadedTask` (component/orchestration/workflows/src/ports.rs:224), `TaskRow` (component/orchestration/workflows/src/ports.rs:223)  
component/orchestration/workflows/src/ports.rs:222-235
```rust
pub trait TasksRead: Send + Sync + 'static {
    type TaskRow: Send + 'static;
    type LoadedTask: Send + 'static;

    fn list_open_tasks(
        &self,
        assignee: Option<&str>,
    ) -> BoxFuture<'_, Result<Vec<Self::TaskRow>, WorkflowPortError>>;

    fn load_task_with_form(
        &self,
        task_id: Uuid,
    ) -> BoxFuture<'_, Result<Option<Self::LoadedTask>, WorkflowPortError>>;
}
```

### 64. Trait TasksCommand
`rust:component/orchestration/workflows/src/ports.rs::TasksCommand`  component/orchestration/workflows/src/ports.rs:238-243  pkg:cargo/workflows-component  score 0.48  via `TaskPageContext` -> USES_TYPE (apps/causeway-server/src/workflow_tasks/page.rs:10, 1.00)  
`pub trait TasksCommand: Send + Sync + 'static`  
Human workflow task command surface.  
members 1  
implementations 3: `PgWorkflowReadRepo#impl(TasksCommand)` (component/orchestration/workflows/src/impls/read_repo.rs:126), `MockWorkflowReadRepo#impl(TasksCommand)` (component/orchestration/workflows/src/impls/read_repo.rs:680), `RemoteWorkflowClient#impl(TasksCommand)` (component/orchestration/workflows/src/remote.rs:124)  
importers 8: `component/orchestration/workflows/src/impls/read_repo.rs` (component/orchestration/workflows/src/impls/read_repo.rs:28-31), `apps/causeway-server/src/composition/state/service_ports.rs` (apps/causeway-server/src/composition/state/service_ports.rs:19-21), `component/orchestration/workflows/src/rpc.rs` (component/orchestration/workflows/src/rpc.rs:21-23) … 5 more  
used_by 11: `RemoteWorkflowClient#impl(TasksCommand)` (component/orchestration/workflows/src/remote.rs:124), `TaskPageContext` (apps/causeway-server/src/workflow_tasks/page.rs:10), `AppState::tasks_command` (apps/causeway-server/src/composition/state/service_ports.rs:150) … 8 more  

### 65. Function relativeTime
`tsx:apps-web/causeway-web/src/pages/routes/Tasks.tsx::relativeTime`  apps-web/causeway-web/src/pages/routes/Tasks.tsx:20-33  pkg:npm/@causeway/web  score 0.47  via `Row` -> CALLS (apps-web/causeway-web/src/pages/routes/Tasks.tsx:65, 1.00)  
`function relativeTime(at: Date, now: Date): string`  
`list/time.rs`: "3 mins ago", "in 2 d".  
callers 8: `relative time::reads as the shell's does` (apps-web/causeway-web/test/app-tasks.test.ts:164), `relative time::reads as the shell's does` (apps-web/causeway-web/test/app-tasks.test.ts:160), `Row` (apps-web/causeway-web/src/pages/routes/Tasks.tsx:61) … 5 more  
tests 1: `relative time::reads as the shell's does` (apps-web/causeway-web/test/app-tasks.test.ts:160)  

### 66. Variant TaskCompletionResult::Completed
`rust:apps/causeway-server/src/workflow_tasks/page.rs::TaskCompletionResult::Completed`  apps/causeway-server/src/workflow_tasks/page.rs:62  pkg:cargo/causeway-server  score 0.43  via `task_complete` -> REFERENCES (apps/causeway-server/src/routes/pages/tasks/complete.rs:47, 1.00)  
`Completed`  
referenced_by 2: `task_complete` (apps/causeway-server/src/routes/pages/tasks/complete.rs:47), `complete_task_from_form` (apps/causeway-server/src/workflow_tasks/page.rs:109)  

### 67. Variant TaskCompletionResult::Rejected
`rust:apps/causeway-server/src/workflow_tasks/page.rs::TaskCompletionResult::Rejected`  apps/causeway-server/src/workflow_tasks/page.rs:63  pkg:cargo/causeway-server  score 0.43  via `task_complete` -> REFERENCES (apps/causeway-server/src/routes/pages/tasks/complete.rs:50, 1.00)  
`Rejected`  
referenced_by 1: `task_complete` (apps/causeway-server/src/routes/pages/tasks/complete.rs:50)  

### 68. Enum PageRefusal
`rust:apps/causeway-server/src/explorer/page.rs::PageRefusal`  apps/causeway-server/src/explorer/page.rs:180-203  pkg:cargo/causeway-server  score 0.42  via `refused` -> USES_TYPE (apps/causeway-server/src/routes/pages/refused.rs:134, 1.00)  
`pub(crate) enum PageRefusal`  
Why an explorer page was not shown.  
members 8  
importers 12: `apps/causeway-server/src/routes/pages/metadata_ops/plane.rs` (apps/causeway-server/src/routes/pages/metadata_ops/plane.rs:19), `apps/causeway-server/src/routes/pages/metadata_ops/trace.rs` (apps/causeway-server/src/routes/pages/metadata_ops/trace.rs:31), `apps/causeway-server/src/explorer.rs` (apps/causeway-server/src/explorer.rs:37) … 9 more  
used_by 17: `decide` (apps/causeway-server/src/routes/pages/twin.rs:41), `decide_all` (apps/causeway-server/src/routes/pages/twin.rs:60), `decide_contract_pages` (apps/causeway-server/src/contracts/page.rs:35) … 14 more  

### 69. Enum TaskCompletionResult
`rust:apps/causeway-server/src/workflow_tasks/page.rs::TaskCompletionResult`  apps/causeway-server/src/workflow_tasks/page.rs:61-64  pkg:cargo/causeway-server  score 0.42  via `apps/causeway-server/src/workflow_tasks.rs` -> IMPORTS (apps/causeway-server/src/workflow_tasks.rs:9-12, 1.00)  
`pub(crate) enum TaskCompletionResult`  
members 2  
importers 1: `apps/causeway-server/src/workflow_tasks.rs` (apps/causeway-server/src/workflow_tasks.rs:9-12)  
used_by 1: `complete_task_from_form` (apps/causeway-server/src/workflow_tasks/page.rs:70)  

### 70. Function redirect_err
`rust:apps/causeway-server/src/routes/pages/tasks/complete.rs::redirect_err`  apps/causeway-server/src/routes/pages/tasks/complete.rs:56-59  pkg:cargo/causeway-server  score 0.41  via `task_complete` -> CALLS (apps/causeway-server/src/routes/pages/tasks/complete.rs:51, 1.00)  
`fn redirect_err(task_id: &str, msg: &str) -> Redirect`  
callers 1: `task_complete` (apps/causeway-server/src/routes/pages/tasks/complete.rs:51)  

### 71. File apps/causeway-server/src/browser_auth/middleware.rs
`file:apps/causeway-server/src/browser_auth/middleware.rs`  apps/causeway-server/src/browser_auth/middleware.rs  pkg:cargo/causeway-server  score 0.38  via `BrowserCaller` <- CONTAINS (apps/causeway-server/src/browser_auth/middleware.rs:26-32, 1.00)  
members 17  
documents 1: `docs/DESIGN-LOG.md` (docs/DESIGN-LOG.md:187)  
imports 11: `AppState` (apps/causeway-server/src/browser_auth/middleware.rs:14), `BearerCredential` (apps/causeway-server/src/browser_auth/middleware.rs:10), `Authenticated` (apps/causeway-server/src/browser_auth/middleware.rs:10) … 8 more  

### 72. File apps/causeway-server/src/composition/state.rs
`file:apps/causeway-server/src/composition/state.rs`  apps/causeway-server/src/composition/state.rs  pkg:cargo/causeway-server  score 0.37  via `AppState` <- CONTAINS (apps/causeway-server/src/composition/state.rs:277-330, 1.00)  
members 32  
designs 1: `docs/rfd/RFD-2cqt6wf1-web-session-state-and-recovery.md` (docs/rfd/RFD-2cqt6wf1-web-session-state-and-recovery.md:83)  
documents 1: `docs/DESIGN-LOG.md` (docs/DESIGN-LOG.md:572)  
imports 32: `cargo/drive-runtime` (apps/causeway-server/src/composition/state.rs:30), `SessionRuntime` (apps/causeway-server/src/composition/state.rs:40), `PlatformRuntime` (apps/causeway-server/src/composition/state.rs:31) … 29 more  

### 73. Struct LoadedTask
`rust:crates/workflow-read-model/src/lib.rs::LoadedTask`  crates/workflow-read-model/src/lib.rs:141-150  pkg:cargo/workflow-read-model  score 0.37  via `apps/causeway-server/src/workflow_tasks.rs` -> IMPORTS (apps/causeway-server/src/workflow_tasks.rs:5, 1.00)  
`pub struct LoadedTask`  
importers 3: `apps/causeway-server-web/src/tasks.rs` (apps/causeway-server-web/src/tasks.rs:6-9), `component/orchestration/workflows/src/ports.rs` (component/orchestration/workflows/src/ports.rs:14), `apps/causeway-server/src/workflow_tasks.rs` (apps/causeway-server/src/workflow_tasks.rs:5)  
used_by 10: `TasksRead` (component/orchestration/workflows/src/ports.rs:224), `TasksRead::load_task_with_form` (component/orchestration/workflows/src/ports.rs:234), `task_wizard` (apps/causeway-server-web/src/views/platform/tasks/wizard.rs:11) … 7 more  
uses_types 5: `TaskRow` (crates/workflow-read-model/src/lib.rs:142), `Form` (crates/workflow-read-model/src/lib.rs:144), `MockWorkflowReadRepo::LoadedTask` (crates/workflow-read-model/src/lib.rs:141) … 2 more  

### 74. Struct TaskRow
`rust:crates/workflow-read-model/src/lib.rs::TaskRow`  crates/workflow-read-model/src/lib.rs:128-138  pkg:cargo/workflow-read-model  score 0.36  via `apps/causeway-server/src/workflow_tasks.rs` -> IMPORTS (apps/causeway-server/src/workflow_tasks.rs:5, 1.00)  
`pub struct TaskRow`  
importers 3: `apps/causeway-server/src/workflow_tasks.rs` (apps/causeway-server/src/workflow_tasks.rs:5), `component/orchestration/workflows/src/ports.rs` (component/orchestration/workflows/src/ports.rs:14), `apps/causeway-server-web/src/tasks.rs` (apps/causeway-server-web/src/tasks.rs:6-9)  
used_by 12: `TasksRead::list_open_tasks` (component/orchestration/workflows/src/ports.rs:229), `tasks` (apps/causeway-server-web/src/views/platform/tasks/list.rs:15), `task_row` (apps/causeway-server-web/src/views/platform/tasks/list/row.rs:12) … 9 more  
uses_types 3: `PgWorkflowReadRepo::TaskRow` (crates/workflow-read-model/src/lib.rs:128), `MockWorkflowReadRepo::TaskRow` (crates/workflow-read-model/src/lib.rs:128), `RemoteWorkflowClient::TaskRow` (crates/workflow-read-model/src/lib.rs:128)  

### 75. Function TasksPage
`tsx:apps-web/causeway-web/src/pages/routes/Tasks.tsx::TasksPage`  apps-web/causeway-web/src/pages/routes/Tasks.tsx:93-127  pkg:npm/@causeway/web  score 0.35  via `apps-web/causeway-web/src/pages/routes/Tasks.tsx` -> CONTAINS (apps-web/causeway-web/src/pages/routes/Tasks.tsx:93-127, 1.00)  
`function TasksPage(props: { readonly ctx: PageContext; readonly tasks: Read<readonly TaskRow[]>; readonly assignee: Option.Option<string>; readonly now: Date; })`  
calls 1: `listing` (apps-web/causeway-web/src/pages/routes/Tasks.tsx:120)  
uses_types 2: `PageContext` (apps-web/causeway-web/src/pages/routes/Tasks.tsx:94), `Read` (apps-web/causeway-web/src/pages/routes/Tasks.tsx:95)  

### 76. Function issues
`rust:apps/causeway-server/src/routes/pages/governance/issues.rs::issues`  apps/causeway-server/src/routes/pages/governance/issues.rs:15-56  pkg:cargo/causeway-server  score 0.34  via `decide` <- CALLS (apps/causeway-server/src/routes/pages/governance/issues.rs:27, 1.00)  
`pub async fn issues( State(state): State<AppState>, Query(query): Query<TabQuery>, Extension(caller): Extension<BrowserCaller>, method: Method, uri: Uri, headers: HeaderMap, ) -> Response`  
The open issues, decided as their twin `PolicyService/ListViolations` is.  
calls 17: `TabDef::new` (apps/causeway-server/src/routes/pages/governance/issues.rs:45), `Breadcrumb::current` (apps/causeway-server/src/routes/pages/governance/issues.rs:25), `render` (apps/causeway-server/src/routes/pages/governance/issues.rs:39) … 14 more  
importers 2: `apps/causeway-server/src/routes/pages/governance.rs` (apps/causeway-server/src/routes/pages/governance.rs:16), `apps/causeway-server/src/routes/pages.rs` (apps/causeway-server/src/routes/pages.rs:36-42)  
referenced_by 1: `routes` (apps/causeway-server/src/routes/app/govern.rs:27)  
references 2: `Section::ISSUES` (apps/causeway-server/src/routes/pages/governance/issues.rs:30), `LIST_VIOLATIONS` (apps/causeway-server/src/routes/pages/governance/issues.rs:27)  
uses_types 3: `AppState` (apps/causeway-server/src/routes/pages/governance/issues.rs:16), `TabQuery` (apps/causeway-server/src/routes/pages/governance/issues.rs:17), `BrowserCaller` (apps/causeway-server/src/routes/pages/governance/issues.rs:18)  

### 77. Variant PageRefusal::SignInRequired
`rust:apps/causeway-server/src/explorer/page.rs::PageRefusal::SignInRequired`  apps/causeway-server/src/explorer/page.rs:182  pkg:cargo/causeway-server  score 0.33  via `refused` -> REFERENCES (apps/causeway-server/src/routes/pages/refused.rs:140, 1.00)  
`SignInRequired`  
Nobody is signed in.  
referenced_by 15: `DecidedTrace::decide` (apps/causeway-server/src/routes/pages/metadata_ops/trace.rs:98), `api_metadata_trace` (apps/causeway-server/src/routes/pages/metadata_ops/trace.rs:226), `DelegatedCatalog::rows` (apps/causeway-server/src/agent/catalog.rs:180) … 12 more  

### 78. Module page
`rust:apps/causeway-server/src/workflow_tasks.rs::page`  apps/causeway-server/src/workflow_tasks.rs:7  pkg:cargo/causeway-server  score 0.33  via `apps/causeway-server/src/workflow_tasks.rs` -> CONTAINS (apps/causeway-server/src/workflow_tasks.rs:7, 1.00)  
`mod page;`  

### 79. File apps/causeway-server/src/routes/app/govern.rs
`file:apps/causeway-server/src/routes/app/govern.rs`  apps/causeway-server/src/routes/app/govern.rs  pkg:cargo/causeway-server  score 0.33  via `routes` <- CONTAINS (apps/causeway-server/src/routes/app/govern.rs:10-74, 1.00)  
members 1  
imports 3: `cargo/axum` (apps/causeway-server/src/routes/app/govern.rs:4), `pages` (apps/causeway-server/src/routes/app/govern.rs:8), `AppState` (apps/causeway-server/src/routes/app/govern.rs:7)  

### 80. Function listing
`tsx:apps-web/causeway-web/src/pages/routes/Tasks.tsx::listing`  apps-web/causeway-web/src/pages/routes/Tasks.tsx:72-91  pkg:npm/@causeway/web  score 0.31  via `apps-web/causeway-web/src/pages/routes/Tasks.tsx` -> CONTAINS (apps-web/causeway-web/src/pages/routes/Tasks.tsx:72-91, 1.00)  
`function listing(rows: readonly TaskRow[], empty: string, now: Date): JSX.Element`  
callers 1: `TasksPage` (apps-web/causeway-web/src/pages/routes/Tasks.tsx:120)  

### 81. Variant BrowserCaller::SignedIn
`rust:apps/causeway-server/src/browser_auth/middleware.rs::BrowserCaller::SignedIn`  apps/causeway-server/src/browser_auth/middleware.rs:31  pkg:cargo/causeway-server  score 0.30  via `BrowserCaller` -> DEFINES (apps/causeway-server/src/browser_auth/middleware.rs:31, 1.00)  
`SignedIn`  
The signed-in person, whose session token the shared verifier proved.  
referenced_by 19: `decide_contract_pages` (apps/causeway-server/src/contracts/page.rs:36), `DecidedAtlasCall::decide` (apps/causeway-server/src/atlas.rs:141), `CommandCaller::decide` (apps/causeway-server/src/composition/state.rs:234) … 16 more  
uses_types 1: `BrowserAccess` (apps/causeway-server/src/browser_auth/middleware.rs:31)  

### 82. Variant PageRefusal::NotFound
`rust:apps/causeway-server/src/explorer/page.rs::PageRefusal::NotFound`  apps/causeway-server/src/explorer/page.rs:186  pkg:cargo/causeway-server  score 0.30  via `refused` -> REFERENCES (apps/causeway-server/src/routes/pages/refused.rs:143, 1.00)  
`NotFound`  
No entry by that name that the caller may view: the catalog has none, (doc continues, 165 bytes)  
referenced_by 10: `DecidedTrace::decide` (apps/causeway-server/src/routes/pages/metadata_ops/trace.rs:116), `PageRefusal::entry` (apps/causeway-server/src/explorer/page.rs:231), `DelegatedCatalog::entry` (apps/causeway-server/src/agent/catalog.rs:88) … 7 more  

### 83. Const TASKS_CHANGED_SIGNAL
`rust:apps/causeway-server-web/src/tasks.rs::TASKS_CHANGED_SIGNAL`  apps/causeway-server-web/src/tasks.rs:13  pkg:cargo/causeway-server-web  score 0.30  via `apps/causeway-server/src/workflow_tasks.rs` -> IMPORTS (apps/causeway-server/src/workflow_tasks.rs:4, 1.00)  
`pub const TASKS_CHANGED_SIGNAL: SignalName = SignalName::new("workflow-tasks-changed");`  
calls 1: `SignalName::new` (apps/causeway-server-web/src/tasks.rs:13)  
importers 1: `apps/causeway-server/src/workflow_tasks.rs` (apps/causeway-server/src/workflow_tasks.rs:4)  
referenced_by 2: `TaskEvent::signal_name` (apps/causeway-server-web/src/views/platform/tasks/model.rs:46), `tests::tasks_realtime_script_uses_task_event_model_signal` (apps/causeway-server-web/src/views/platform/tasks.rs:97)  
uses_types 1: `SignalName` (apps/causeway-server-web/src/tasks.rs:13)  

## Left out

12 included nodes have no source window for want of budget.  
- `pkg:cargo/causeway-server` Package 0.30 (over budget, 460 tokens)
- `rust:apps/causeway-server/src/workflow_tasks/page.rs::TaskPageContext#impl` Impl 0.30 (over budget, 140 tokens)
- `typescript:apps-web/causeway-web/src/pages/app-read.ts::Read` TypeAlias 0.29 (over budget, 326 tokens)
- and 615 more

## Unresolved references in the seeds' files

- `getTime` at apps-web/causeway-web/src/pages/routes/Tasks.tsx:21 (no-declaration)
- `trunc` at apps-web/causeway-web/src/pages/routes/Tasks.tsx:21 (no-declaration)
- `abs` at apps-web/causeway-web/src/pages/routes/Tasks.tsx:22 (no-declaration)
- `trunc` at apps-web/causeway-web/src/pages/routes/Tasks.tsx:27 (no-declaration)
- `trunc` at apps-web/causeway-web/src/pages/routes/Tasks.tsx:29 (no-declaration)
- `trunc` at apps-web/causeway-web/src/pages/routes/Tasks.tsx:30 (no-declaration)
- `toContain` at apps-web/causeway-web/test/app-tasks.test.ts:100 (no-declaration)
- `toContain` at apps-web/causeway-web/test/app-tasks.test.ts:101 (no-declaration)
- `toContain` at apps-web/causeway-web/test/app-tasks.test.ts:102 (no-declaration)
- `toContain` at apps-web/causeway-web/test/app-tasks.test.ts:103 (no-declaration)

