# Context pack

Task: A page on a sibling origin can make the browser attach the session cookie to a Connect GET or a text/plain POST without a preflight, and SameSite=Lax does not stop it. Serve a cookie-authenticated RPC only when the request carries Connect's protocol header or content type, or its Origin is one of ours; refuse everything else before the handler.

## Seeds

- `rust:apps/causeway-server/src/browser_auth/mod.rs::BrowserAuth::secure_cookies` Method 0.90 (text)
- `rust:apps/causeway-server/src/browser_auth/session.rs::SessionCookies::jar_from` Method 0.81 (text)
- `rust:apps/causeway-server/src/browser_auth/handlers.rs::LoginOrigin` Enum 0.81 (text)
- `rust:apps/causeway-server/src/browser_auth/middleware.rs::attach_browser_caller` Function 0.78 (text)
- `rust:apps/causeway-server/src/browser_auth/middleware.rs::present_session_token` Function 0.75 (text)
- `rust:apps/causeway-server/src/routes/cors.rs::ALLOWED_HEADERS` Const 0.74 (text)
- `rhai:.mise/rhai/causeway/personas.rhai::session_headers` Function 0.74 (text)
- `rust:apps/causeway-server/src/browser_auth/session.rs::SessionCookies::clear` Method 0.72 (text)
- `rust:apps/causeway-server/src/browser_auth/session.rs::BrowserSession` Struct 0.71 (text)
- `rust:bootstrap/cw-host/src/modules/http.rs::set_cookies` Function 0.69 (text)

Budget 30000 tokens, used 29949. Seeds 10, candidates 594, included 88, left out 801.

## Context

### 1. Function attach_browser_caller
`rust:apps/causeway-server/src/browser_auth/middleware.rs::attach_browser_caller`  apps/causeway-server/src/browser_auth/middleware.rs:169-214  pkg:cargo/causeway-server  score 2.39  
`pub(crate) async fn attach_browser_caller( State(state): State<AppState>, mut request: Request, next: Next, ) -> Response`  
Reads the session cookie, re-validates its access token against the (doc continues, 309 bytes)  
calls 25: `caller_from_token` (apps/causeway-server/src/browser_auth/middleware.rs:189), `anonymous` (apps/causeway-server/src/browser_auth/middleware.rs:204), `sign_in_required` (apps/causeway-server/src/browser_auth/middleware.rs:202) … 22 more  
importers 2: `apps/causeway-server/src/browser_auth/tests.rs` (apps/causeway-server/src/browser_auth/tests.rs:18-20), `apps/causeway-server/src/browser_auth/mod.rs` (apps/causeway-server/src/browser_auth/mod.rs:28-31)  
referenced_by 1: `build_router` (apps/causeway-server/src/routes/mod.rs:114)  
uses_types 1: `AppState` (apps/causeway-server/src/browser_auth/middleware.rs:170)  
apps/causeway-server/src/browser_auth/middleware.rs:169-214
```rust
pub(crate) async fn attach_browser_caller(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Response {
    let Some(auth) = state.browser_auth().cloned() else {
        return anonymous(request, next).await;
    };
    let cookies = SessionCookies::new(auth.key(), auth.secure_cookies());
    let gated = requires_sign_in(request.uri().path());
    let Some(session) = cookies.session(request.headers()) else {
        if gated {
            return sign_in_required(&request);
        }
        return anonymous(request, next).await;
    };
    let now = cookie::time::OffsetDateTime::now_utc().unix_timestamp();
    let access = if session.is_expired(now) {
        None
    } else {
        caller_from_token(&auth, &session.access_token).await
    };
    match access {
        Some(access) => {
            tracing::debug!(subject = %session.subject, "browser caller attached");
            let caller = BrowserCaller::SignedIn(Arc::new(access));
            request.extensions_mut().insert(caller.clone());
            present_session_token(request.headers_mut(), &session.access_token);
            CURRENT_CALLER.scope(caller, next.run(request)).await
        }
        None => {
            tracing::info!(subject = %session.subject, "browser session no longer validates; clearing");
            let mut response = if gated {
                sign_in_required(&request)
            } else {
                anonymous(request, next).await
            };
            for value in cookies.clear(SESSION_COOKIE) {
                if let Ok(value) = HeaderValue::from_str(&value) {
                    response.headers_mut().append(header::SET_COOKIE, value);
                }
            }
            response
        }
    }
}
```

### 2. Method BrowserAuth::secure_cookies
`rust:apps/causeway-server/src/browser_auth/mod.rs::BrowserAuth::secure_cookies`  apps/causeway-server/src/browser_auth/mod.rs:185-187  pkg:cargo/causeway-server  score 2.24  
`pub(crate) fn secure_cookies(&self) -> bool`  
Whether cookies are marked `Secure`: yes when the public origin is (doc continues, 142 bytes)  
callers 7: `whoami` (apps/causeway-server/src/browser_auth/handlers.rs:345), `callback` (apps/causeway-server/src/browser_auth/handlers.rs:198), `attach_browser_caller` (apps/causeway-server/src/browser_auth/middleware.rs:177) … 4 more  
apps/causeway-server/src/browser_auth/mod.rs:185-187
```rust
    pub(crate) fn secure_cookies(&self) -> bool {
        self.topology.redirect_base.starts_with("https://")
    }
```

### 3. Struct BrowserSession
`rust:apps/causeway-server/src/browser_auth/session.rs::BrowserSession`  apps/causeway-server/src/browser_auth/session.rs:19-31  pkg:cargo/causeway-server  score 1.93  
`pub(crate) struct BrowserSession`  
What the session cookie holds. Stateless: the cookie is the session.  
members 1  
importers 2: `apps/causeway-server/src/browser_auth/mod.rs` (apps/causeway-server/src/browser_auth/mod.rs:34), `apps/causeway-server/src/browser_auth/handlers.rs` (apps/causeway-server/src/browser_auth/handlers.rs:17)  
used_by 5: `SessionCookies::session` (apps/causeway-server/src/browser_auth/session.rs:82), `tests::a_session_round_trips_through_the_private_cookie` (apps/causeway-server/src/browser_auth/session.rs:161), `complete_login` (apps/causeway-server/src/browser_auth/handlers.rs:239) … 2 more  
apps/causeway-server/src/browser_auth/session.rs:19-31
```rust
pub(crate) struct BrowserSession {
    /// The subject the issuer asserted; what the directory joins on.
    pub(crate) subject: String,
    /// A display handle when the issuer offered one.
    pub(crate) handle: Option<String>,
    /// Groups asserted at sign-in.
    pub(crate) groups: Vec<String>,
    /// The access token minted for this process's audience; the subject
    /// token for downstream delegation.
    pub(crate) access_token: String,
    /// Unix seconds after which the session is not honored.
    pub(crate) expires_at: i64,
}
```

### 4. Enum LoginOrigin
`rust:apps/causeway-server/src/browser_auth/handlers.rs::LoginOrigin`  apps/causeway-server/src/browser_auth/handlers.rs:51-56  pkg:cargo/causeway-server  score 1.90  
`enum LoginOrigin`  
Where a sign-in starts. (doc continues, 315 bytes)  
members 3  
used_by 1: `LoginOrigin#impl` (apps/causeway-server/src/browser_auth/handlers.rs:58-84)  
apps/causeway-server/src/browser_auth/handlers.rs:51-56
```rust
enum LoginOrigin {
    /// The browser is on the callback's origin, or nothing says otherwise.
    Here,
    /// The browser is elsewhere: send it to this `/auth/login` URL.
    Move(String),
}
```

### 5. Method SessionCookies::jar_from
`rust:apps/causeway-server/src/browser_auth/session.rs::SessionCookies::jar_from`  apps/causeway-server/src/browser_auth/session.rs:61-74  pkg:cargo/causeway-server  score 1.74  
`fn jar_from(&self, headers: &HeaderMap) -> CookieJar`  
The jar the request carried, decrypted with this key.  
callers 1: `SessionCookies::read` (apps/causeway-server/src/browser_auth/session.rs:77)  
apps/causeway-server/src/browser_auth/session.rs:61-74
```rust
    fn jar_from(&self, headers: &HeaderMap) -> CookieJar {
        let mut jar = CookieJar::new();
        for header in headers.get_all(axum::http::header::COOKIE) {
            let Ok(raw) = header.to_str() else { continue };
            // `encoded()` percent-encodes values on the way out and the
            // browser returns them as sent, so decode on the way in.
            for pair in raw.split(';') {
                if let Ok(cookie) = Cookie::parse_encoded(pair.trim().to_owned()) {
                    jar.add_original(cookie);
                }
            }
        }
        jar
    }
```

### 6. Function session_headers
`rhai:.mise/rhai/causeway/personas.rhai::session_headers`  .mise/rhai/causeway/personas.rhai:393-403  pkg:npm/causeway-monorepo  score 1.69  
`fn session_headers(session, extra)`  
The headers a signed-in browser's request carries: EXTRA (or ()) plus the session's cookie. A session that is not signed in adds nothing.  
callers 4: `test_a_session_adds_its_cookie_and_an_unsigned_one_adds_nothing` (.mise/rhai/causeway/personas_test.rhai:364), `.mise/scripts/tasks/vsa/runtime-smoke.rhai` (.mise/scripts/tasks/vsa/runtime-smoke.rhai:168), `test_a_session_adds_its_cookie_and_an_unsigned_one_adds_nothing` (.mise/rhai/causeway/personas_test.rhai:365) … 1 more  
calls 1: `is_blank` (.mise/rhai/causeway/personas.rhai:399)  
.mise/rhai/causeway/personas.rhai:393-403
```rhai
fn session_headers(session, extra) {
    let out = #{};
    let more = extra ?? #{};
    for name in more.keys() {
        out[name] = more[name];
    }
    if !is_blank(session.cookie) {
        out.cookie = session.cookie;
    }
    out
}
```

### 7. Function present_session_token
`rust:apps/causeway-server/src/browser_auth/middleware.rs::present_session_token`  apps/causeway-server/src/browser_auth/middleware.rs:237-244  pkg:cargo/causeway-server  score 1.67  
`pub(crate) fn present_session_token(headers: &mut axum::http::HeaderMap, access_token: &str)`  
Presents the session's access token as the request's bearer credential, (doc continues, 700 bytes)  
callers 1: `attach_browser_caller` (apps/causeway-server/src/browser_auth/middleware.rs:196)  
apps/causeway-server/src/browser_auth/middleware.rs:237-244
```rust
pub(crate) fn present_session_token(headers: &mut axum::http::HeaderMap, access_token: &str) {
    if headers.contains_key(header::AUTHORIZATION) {
        return;
    }
    if let Ok(value) = HeaderValue::from_str(&format!("Bearer {access_token}")) {
        headers.insert(header::AUTHORIZATION, value);
    }
}
```

### 8. Struct BrowserAuth
`rust:apps/causeway-server/src/browser_auth/mod.rs::BrowserAuth`  apps/causeway-server/src/browser_auth/mod.rs:37-51  pkg:cargo/causeway-server  score 1.64  via `BrowserAuth#impl` -> USES_TYPE (apps/causeway-server/src/browser_auth/mod.rs:101-207, 1.00)  
`pub(crate) struct BrowserAuth`  
Everything the sign-in handlers and the page middleware share.  
members 12  
importers 3: `apps/causeway-server/src/browser_auth/tests.rs` (apps/causeway-server/src/browser_auth/tests.rs:18-20), `apps/causeway-server/src/browser_auth/handlers.rs` (apps/causeway-server/src/browser_auth/handlers.rs:18), `apps/causeway-server/src/ws/tests/relay.rs` (apps/causeway-server/src/ws/tests/relay.rs:27)  
used_by 15: `complete_login` (apps/causeway-server/src/browser_auth/handlers.rs:236), `caller_from_token` (apps/causeway-server/src/browser_auth/middleware.rs:255), `test_browser_auth` (apps/causeway-server/src/composition/remote_services/tests.rs:126) … 12 more  
uses_types 4: `Scopes` (apps/causeway-server/src/browser_auth/mod.rs:40), `AuthenticationPoint` (apps/causeway-server/src/browser_auth/mod.rs:48), `BrowserAuthTopology` (apps/causeway-server/src/browser_auth/mod.rs:38) … 1 more  
apps/causeway-server/src/browser_auth/mod.rs:37-51
```rust
pub(crate) struct BrowserAuth {
    topology: BrowserAuthTopology,
    /// The topology's sign-in scopes, parsed once at startup.
    scopes: causeway_principal::Scopes,
    client_secret: String,
    /// The shared verifier (`Authenticator::from_topology`), built from
    /// `[auth.browser]`'s issuer and audience: the same checks, keys cache
    /// and discovery as every other plane.
    authenticator: Authenticator,
    /// The same verifier behind the access-control typestate, so a page's
    /// caller is an `Access<Authenticated>` proof, as an RPC caller is.
    authentication: AuthenticationPoint,
    key: Key,
    http: reqwest::Client,
}
```

### 9. Struct SessionCookies
`rust:apps/causeway-server/src/browser_auth/session.rs::SessionCookies`  apps/causeway-server/src/browser_auth/session.rs:50-53  pkg:cargo/causeway-server  score 1.55  via `SessionCookies#impl` -> USES_TYPE (apps/causeway-server/src/browser_auth/session.rs:55-137, 1.00)  
`pub(crate) struct SessionCookies<'a>`  
Reads and writes the two private cookies.  
members 9  
importers 2: `apps/causeway-server/src/browser_auth/middleware.rs` (apps/causeway-server/src/browser_auth/middleware.rs:13), `apps/causeway-server/src/browser_auth/handlers.rs` (apps/causeway-server/src/browser_auth/handlers.rs:17)  
used_by 1: `SessionCookies#impl` (apps/causeway-server/src/browser_auth/session.rs:55-137)  
apps/causeway-server/src/browser_auth/session.rs:50-53
```rust
pub(crate) struct SessionCookies<'a> {
    key: &'a Key,
    secure: bool,
}
```

### 10. Method SessionCookies::clear
`rust:apps/causeway-server/src/browser_auth/session.rs::SessionCookies::clear`  apps/causeway-server/src/browser_auth/session.rs:129-136  pkg:cargo/causeway-server  score 1.53  
`pub(crate) fn clear(&self, name: &'static str) -> Vec<String>`  
`Set-Cookie` values that clear one cookie.  
callers 1: `tests::another_key_cannot_read_the_cookie_and_clearing_expires_it` (apps/causeway-server/src/browser_auth/session.rs:200)  
tests 1: `tests::another_key_cannot_read_the_cookie_and_clearing_expires_it` (apps/causeway-server/src/browser_auth/session.rs:200)  
apps/causeway-server/src/browser_auth/session.rs:129-136
```rust
    pub(crate) fn clear(&self, name: &'static str) -> Vec<String> {
        let mut jar = CookieJar::new();
        jar.add_original(Cookie::build((name, "")).path("/").build());
        jar.remove(Cookie::build((name, "")).path("/").build());
        jar.delta()
            .map(|cookie| cookie.encoded().to_string())
            .collect()
    }
```

### 11. Function set_cookies
`rust:bootstrap/cw-host/src/modules/http.rs::set_cookies`  bootstrap/cw-host/src/modules/http.rs:115-122  pkg:cargo/cw-host  score 1.45  
`fn set_cookies(headers: &reqwest::header::HeaderMap) -> rhai::Array`  
Every `Set-Cookie` value, in the order the response carried them. (doc continues, 135 bytes)  
callers 2: `open` (bootstrap/cw-host/src/modules/http.rs:172), `request` (bootstrap/cw-host/src/modules/http.rs:130)  
bootstrap/cw-host/src/modules/http.rs:115-122
```rust
fn set_cookies(headers: &reqwest::header::HeaderMap) -> rhai::Array {
    headers
        .get_all(reqwest::header::SET_COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .map(|v| Dynamic::from(v.to_string()))
        .collect()
}
```

### 12. Const ALLOWED_HEADERS
`rust:apps/causeway-server/src/routes/cors.rs::ALLOWED_HEADERS`  apps/causeway-server/src/routes/cors.rs:24-32  pkg:cargo/causeway-server  score 1.42  
`const ALLOWED_HEADERS: [HeaderName; 7] = [ header::CONTENT_TYPE, header::ACCEPT, header::AUTHORIZATION, HeaderName::from_static("connect-protocol-version"), HeaderName::from_static("connect-timeout-ms"), HeaderName::from_static("x-grpc-web"), HeaderName::from_static("x-user-agent"), ];`  
The request headers a cross-origin browser client of the shell sends: (doc continues, 117 bytes)  
apps/causeway-server/src/routes/cors.rs:24-32
```rust
const ALLOWED_HEADERS: [HeaderName; 7] = [
    header::CONTENT_TYPE,
    header::ACCEPT,
    header::AUTHORIZATION,
    HeaderName::from_static("connect-protocol-version"),
    HeaderName::from_static("connect-timeout-ms"),
    HeaderName::from_static("x-grpc-web"),
    HeaderName::from_static("x-user-agent"),
];
```

### 13. File apps/causeway-server/src/browser_auth/handlers.rs
`file:apps/causeway-server/src/browser_auth/handlers.rs`  apps/causeway-server/src/browser_auth/handlers.rs  pkg:cargo/causeway-server  score 1.28  via `LoginOrigin` <- CONTAINS (apps/causeway-server/src/browser_auth/handlers.rs:51-56, 1.00)  
members 18  
documents 1: `docs/DESIGN-LOG.md` (docs/DESIGN-LOG.md:90)  
imports 21: `LOGIN_COOKIE` (apps/causeway-server/src/browser_auth/handlers.rs:17), `AppState` (apps/causeway-server/src/browser_auth/handlers.rs:19), `cargo/causeway-oidc` (apps/causeway-server/src/browser_auth/handlers.rs:9-12) … 18 more  

### 14. Impl SessionCookies#impl
`rust:apps/causeway-server/src/browser_auth/session.rs::SessionCookies#impl`  apps/causeway-server/src/browser_auth/session.rs:55-137  pkg:cargo/causeway-server  score 1.17  via `SessionCookies::jar_from` <- DEFINES (apps/causeway-server/src/browser_auth/session.rs:61-74, 1.00)  
`impl<'a> SessionCookies<'a>`  
members 9  
uses_types 1: `SessionCookies` (apps/causeway-server/src/browser_auth/session.rs:55-137)  

### 15. Method SessionCookies::session
`rust:apps/causeway-server/src/browser_auth/session.rs::SessionCookies::session`  apps/causeway-server/src/browser_auth/session.rs:82-84  pkg:cargo/causeway-server  score 1.07  via `BrowserSession` <- USES_TYPE (apps/causeway-server/src/browser_auth/session.rs:82, 1.00)  
`pub(crate) fn session(&self, headers: &HeaderMap) -> Option<BrowserSession>`  
callers 3: `tests::a_session_round_trips_through_the_private_cookie` (apps/causeway-server/src/browser_auth/session.rs:177), `whoami` (apps/causeway-server/src/browser_auth/handlers.rs:348), `attach_browser_caller` (apps/causeway-server/src/browser_auth/middleware.rs:179)  
calls 1: `SessionCookies::read` (apps/causeway-server/src/browser_auth/session.rs:83)  
tests 1: `tests::a_session_round_trips_through_the_private_cookie` (apps/causeway-server/src/browser_auth/session.rs:177)  
uses_types 1: `BrowserSession` (apps/causeway-server/src/browser_auth/session.rs:82)  
apps/causeway-server/src/browser_auth/session.rs:82-84
```rust
    pub(crate) fn session(&self, headers: &HeaderMap) -> Option<BrowserSession> {
        self.read(headers, SESSION_COOKIE)
    }
```

### 16. Method SessionCookies::new
`rust:apps/causeway-server/src/browser_auth/session.rs::SessionCookies::new`  apps/causeway-server/src/browser_auth/session.rs:56-58  pkg:cargo/causeway-server  score 0.98  via `attach_browser_caller` -> CALLS (apps/causeway-server/src/browser_auth/middleware.rs:177, 1.00)  
`pub(crate) fn new(key: &'a Key, secure: bool) -> Self`  
callers 10: `callback` (apps/causeway-server/src/browser_auth/handlers.rs:190), `logout` (apps/causeway-server/src/browser_auth/handlers.rs:334), `login` (apps/causeway-server/src/browser_auth/handlers.rs:176) … 7 more  
tests 2: `tests::a_session_round_trips_through_the_private_cookie` (apps/causeway-server/src/browser_auth/session.rs:160), `tests::another_key_cannot_read_the_cookie_and_clearing_expires_it` (apps/causeway-server/src/browser_auth/session.rs:184)  
apps/causeway-server/src/browser_auth/session.rs:56-58
```rust
    pub(crate) fn new(key: &'a Key, secure: bool) -> Self {
        Self { key, secure }
    }
```

### 17. File apps/causeway-server/src/browser_auth/mod.rs
`file:apps/causeway-server/src/browser_auth/mod.rs`  apps/causeway-server/src/browser_auth/mod.rs  pkg:cargo/causeway-server  score 0.95  via `BrowserAuth#impl` <- CONTAINS (apps/causeway-server/src/browser_auth/mod.rs:101-207, 1.00)  
members 12  
documents 1: `docs/DESIGN-LOG.md` (docs/DESIGN-LOG.md:170)  
imports 20: `cargo/causeway-oidc` (apps/causeway-server/src/browser_auth/mod.rs:23), `current_browser_caller` (apps/causeway-server/src/browser_auth/mod.rs:28-31), `BrowserAccess` (apps/causeway-server/src/browser_auth/mod.rs:28-31) … 17 more  

### 18. Function test_a_session_adds_its_cookie_and_an_unsigned_one_adds_nothing
`rhai:.mise/rhai/causeway/personas_test.rhai::test_a_session_adds_its_cookie_and_an_unsigned_one_adds_nothing`  .mise/rhai/causeway/personas_test.rhai:362-367  pkg:npm/causeway-monorepo  score 0.90  via `session_headers` <- CALLS (.mise/rhai/causeway/personas_test.rhai:366, 1.00)  
`fn test_a_session_adds_its_cookie_and_an_unsigned_one_adds_nothing()`  
calls 3: `session_headers` (.mise/rhai/causeway/personas_test.rhai:364), `session_headers` (.mise/rhai/causeway/personas_test.rhai:365), `session_headers` (.mise/rhai/causeway/personas_test.rhai:366)  
.mise/rhai/causeway/personas_test.rhai:362-367
```rhai
fn test_a_session_adds_its_cookie_and_an_unsigned_one_adds_nothing() {
    let signed = #{ signed_in: true, cookie: "causeway_session=s" };
    assert_eq(personas::session_headers(signed, #{ accept: "text/html" }), #{ accept: "text/html", cookie: "causeway_session=s" });
    assert_eq(personas::session_headers(signed, ()), #{ cookie: "causeway_session=s" });
    assert_eq(personas::session_headers(#{ signed_in: false, cookie: "" }, #{ accept: "text/html" }), #{ accept: "text/html" });
}
```

### 19. File apps/causeway-server/src/browser_auth/session.rs
`file:apps/causeway-server/src/browser_auth/session.rs`  apps/causeway-server/src/browser_auth/session.rs  pkg:cargo/causeway-server  score 0.88  via `BrowserSession` <- CONTAINS (apps/causeway-server/src/browser_auth/session.rs:19-31, 1.00)  
members 9  
documents 1: `docs/DESIGN-LOG.md` (docs/DESIGN-LOG.md:2983)  
imports 3: `cargo/serde` (apps/causeway-server/src/browser_auth/session.rs:8), `cargo/cookie` (apps/causeway-server/src/browser_auth/session.rs:6), `cargo/axum` (apps/causeway-server/src/browser_auth/session.rs:5)  

### 20. Struct AppState
`rust:apps/causeway-server/src/composition/state.rs::AppState`  apps/causeway-server/src/composition/state.rs:203-254  pkg:cargo/causeway-server  score 0.86  via `attach_browser_caller` -> USES_TYPE (apps/causeway-server/src/browser_auth/middleware.rs:170, 1.00)  
`pub(crate) struct AppState`  
members 150  
importers 134: `apps/causeway-server/src/agent/ws.rs` (apps/causeway-server/src/agent/ws.rs:11), `apps/causeway-server/src/routes/pages/query.rs` (apps/causeway-server/src/routes/pages/query.rs:8), `apps/causeway-server/src/routes/pages/semantic.rs` (apps/causeway-server/src/routes/pages/semantic.rs:10) … 131 more  
references 6: `access` (apps/causeway-server/src/composition/state.rs:231), `AppState::browser_auth` (apps/causeway-server/src/composition/state.rs:253), `services` (apps/causeway-server/src/composition/state.rs:250) … 3 more  
used_by 317: `metadata_quarantine_dismiss_batch` (apps/causeway-server/src/routes/pages/metadata_ops/quarantine.rs:177), `execute_catalog_query` (apps/causeway-server/src/catalog/ws.rs:77), `contract_review_ensure` (apps/causeway-server/src/routes/pages/contracts/review.rs:37) … 314 more  
uses_types 20: `SessionRuntime` (apps/causeway-server/src/composition/state.rs:204), `RpcAuthority` (apps/causeway-server/src/composition/state.rs:236), `QueryRuntime` (apps/causeway-server/src/composition/state.rs:215) … 17 more  
apps/causeway-server/src/composition/state.rs:203-254
```rust
pub(crate) struct AppState {
    session: Arc<SessionRuntime>,
    content: Arc<ContentRuntime>,
    realtime: Arc<RealtimeRuntime>,
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

### 21. Method SessionCookies::set_session
`rust:apps/causeway-server/src/browser_auth/session.rs::SessionCookies::set_session`  apps/causeway-server/src/browser_auth/session.rs:101-112  pkg:cargo/causeway-server  score 0.85  via `BrowserSession` <- USES_TYPE (apps/causeway-server/src/browser_auth/session.rs:101, 1.00)  
`pub(crate) fn set_session(&self, session: &BrowserSession) -> Vec<String>`  
`Set-Cookie` values that store a session.  
callers 2: `tests::a_session_round_trips_through_the_private_cookie` (apps/causeway-server/src/browser_auth/session.rs:170), `callback` (apps/causeway-server/src/browser_auth/handlers.rs:191)  
calls 1: `SessionCookies::build` (apps/causeway-server/src/browser_auth/session.rs:108)  
tests 1: `tests::a_session_round_trips_through_the_private_cookie` (apps/causeway-server/src/browser_auth/session.rs:170)  
uses_types 1: `BrowserSession` (apps/causeway-server/src/browser_auth/session.rs:101)  
apps/causeway-server/src/browser_auth/session.rs:101-112
```rust
    pub(crate) fn set_session(&self, session: &BrowserSession) -> Vec<String> {
        let value = serde_json::to_string(session).expect("session serializes");
        let ttl = Duration::seconds(
            (session.expires_at - OffsetDateTime::now_utc().unix_timestamp()).max(1),
        );
        let mut jar = CookieJar::new();
        jar.private_mut(self.key)
            .add(self.build(SESSION_COOKIE, value, ttl));
        jar.delta()
            .map(|cookie| cookie.encoded().to_string())
            .collect()
    }
```

### 22. Method SessionCookies::read
`rust:apps/causeway-server/src/browser_auth/session.rs::SessionCookies::read`  apps/causeway-server/src/browser_auth/session.rs:76-80  pkg:cargo/causeway-server  score 0.84  via `SessionCookies::jar_from` <- CALLS (apps/causeway-server/src/browser_auth/session.rs:77, 1.00)  
`fn read<T: for<'de> Deserialize<'de>>(&self, headers: &HeaderMap, name: &str) -> Option<T>`  
callers 2: `SessionCookies::session` (apps/causeway-server/src/browser_auth/session.rs:83), `SessionCookies::pending_login` (apps/causeway-server/src/browser_auth/session.rs:87)  
calls 1: `SessionCookies::jar_from` (apps/causeway-server/src/browser_auth/session.rs:77)  
apps/causeway-server/src/browser_auth/session.rs:76-80
```rust
    fn read<T: for<'de> Deserialize<'de>>(&self, headers: &HeaderMap, name: &str) -> Option<T> {
        let jar = self.jar_from(headers);
        let cookie = jar.private(self.key).get(name)?;
        serde_json::from_str(cookie.value()).ok()
    }
```

### 23. File apps/causeway-server/src/browser_auth/middleware.rs
`file:apps/causeway-server/src/browser_auth/middleware.rs`  apps/causeway-server/src/browser_auth/middleware.rs  pkg:cargo/causeway-server  score 0.83  via `attach_browser_caller` <- CONTAINS (apps/causeway-server/src/browser_auth/middleware.rs:169-214, 1.00)  
members 16  
documents 1: `docs/DESIGN-LOG.md` (docs/DESIGN-LOG.md:91)  
imports 11: `AppState` (apps/causeway-server/src/browser_auth/middleware.rs:14), `BearerCredential` (apps/causeway-server/src/browser_auth/middleware.rs:10), `Authenticated` (apps/causeway-server/src/browser_auth/middleware.rs:10) … 8 more  

### 24. Function callback
`rust:apps/causeway-server/src/browser_auth/handlers.rs::callback`  apps/causeway-server/src/browser_auth/handlers.rs:180-208  pkg:cargo/causeway-server  score 0.79  via `BrowserAuth::secure_cookies` <- CALLS (apps/causeway-server/src/browser_auth/handlers.rs:198, 0.75)  
`async fn callback( State(state): State<AppState>, headers: HeaderMap, Query(query): Query<CallbackQuery>, ) -> Response`  
calls 21: `redirect_with_cookies` (apps/causeway-server/src/browser_auth/handlers.rs:194), `SessionCookies::new` (apps/causeway-server/src/browser_auth/handlers.rs:190), `complete_login` (apps/causeway-server/src/browser_auth/handlers.rs:188) … 18 more  
uses_types 2: `CallbackQuery` (apps/causeway-server/src/browser_auth/handlers.rs:183), `AppState` (apps/causeway-server/src/browser_auth/handlers.rs:181)  
apps/causeway-server/src/browser_auth/handlers.rs:180-208
```rust
async fn callback(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<CallbackQuery>,
) -> Response {
    let Some(auth) = state.browser_auth() else {
        return not_configured();
    };
    match complete_login(auth, &headers, &query).await {
        Ok((session, return_to)) => {
            let cookies = SessionCookies::new(auth.key(), auth.secure_cookies());
            let mut set = cookies.set_session(&session);
            set.extend(cookies.clear(LOGIN_COOKIE));
            tracing::info!(subject = %session.subject, "browser signed in");
            redirect_with_cookies(&return_to, set)
        }
        Err(refusal) => {
            tracing::warn!(reason = %refusal.reason, "browser sign-in refused");
            let cookies = SessionCookies::new(auth.key(), auth.secure_cookies());
            let mut response = (refusal.status, refusal.reason).into_response();
            for value in cookies.clear(LOGIN_COOKIE) {
                if let Ok(value) = HeaderValue::from_str(&value) {
                    response.headers_mut().append(header::SET_COOKIE, value);
                }
            }
            response
        }
    }
}
```

### 25. Impl BrowserAuth#impl
`rust:apps/causeway-server/src/browser_auth/mod.rs::BrowserAuth#impl`  apps/causeway-server/src/browser_auth/mod.rs:101-207  pkg:cargo/causeway-server  score 0.77  via `BrowserAuth::secure_cookies` <- DEFINES (apps/causeway-server/src/browser_auth/mod.rs:185-187, 1.00)  
`impl BrowserAuth`  
members 11  
uses_types 1: `BrowserAuth` (apps/causeway-server/src/browser_auth/mod.rs:101-207)  

### 26. Function complete_login
`rust:apps/causeway-server/src/browser_auth/handlers.rs::complete_login`  apps/causeway-server/src/browser_auth/handlers.rs:235-328  pkg:cargo/causeway-server  score 0.73  via `BrowserSession` <- USES_TYPE (apps/causeway-server/src/browser_auth/handlers.rs:239, 1.00)  
`async fn complete_login( auth: &BrowserAuth, headers: &HeaderMap, query: &CallbackQuery, ) -> Result<(BrowserSession, String), Refusal>`  
The callback's checks, in order: a login in flight, the state it sent, (doc continues, 271 bytes)  
callers 1: `callback` (apps/causeway-server/src/browser_auth/handlers.rs:188)  
calls 40: `Refusal::upstream` (apps/causeway-server/src/browser_auth/handlers.rs:279), `Refusal::bad_request` (apps/causeway-server/src/browser_auth/handlers.rs:312), `claims_to_identity` (apps/causeway-server/src/browser_auth/handlers.rs:309) … 37 more  
uses_types 4: `BrowserAuth` (apps/causeway-server/src/browser_auth/handlers.rs:236), `Refusal` (apps/causeway-server/src/browser_auth/handlers.rs:239), `BrowserSession` (apps/causeway-server/src/browser_auth/handlers.rs:239) … 1 more  
apps/causeway-server/src/browser_auth/handlers.rs:235-314, cut
```rust
async fn complete_login(
    auth: &BrowserAuth,
    headers: &HeaderMap,
    query: &CallbackQuery,
) -> Result<(BrowserSession, String), Refusal> {
    let cookies = SessionCookies::new(auth.key(), auth.secure_cookies());
    let pending = cookies
        .pending_login(headers)
        .ok_or_else(|| Refusal::bad_request("no sign-in is in progress in this browser"))?;
    if let Some(error) = &query.error {
        return Err(Refusal::bad_request(format!(
            "the issuer refused the sign-in: {error}{}",
            query
                .error_description
                .as_deref()
                .map(|d| format!(" ({d})"))
                .unwrap_or_default()
        )));
    }
    if query.state.as_deref() != Some(pending.state.as_str()) {
        return Err(Refusal::bad_request(
            "the sign-in state does not match the login that started it",
        ));
    }
    let code = query
        .code
        .as_deref()
        .filter(|code| !code.trim().is_empty())
        .ok_or_else(|| Refusal::bad_request("the callback carried no code"))?;
    let topology = auth.topology();
    let redirect_uri = topology.redirect_uri();
    let token_endpoint = topology.token_endpoint();
    let grant = redeem_code(
        auth.http(),
        &CodeRedemption::new(
            &token_endpoint,
            &topology.client_id,
            auth.client_secret(),
            &redirect_uri,
            &pending.verifier,
        ),
        code,
    )
    .await
    .map_err(|error| Refusal::upstream(format!("could not redeem the sign-in code: {error}")))?;
    let id_token = grant
        .id_token
        .as_deref()
        .ok_or_else(|| Refusal::upstream("the issuer returned no id_token"))?;
    let kid = causeway_oidc::code_flow::key_id(id_token);
    let jwks = auth
        .jwks(kid.as_deref())
        .await
        .map_err(|error| Refusal::upstream(format!("issuer keys unavailable: {error}")))?;
    let identity = validate_id_token(
        id_token,
        &jwks,
        topology.issuer.as_str(),
        &topology.client_id,
        &pending.nonce,
    )
    .map_err(|error| Refusal::bad_request(error.to_string()))?;
    let claims = auth
        .authenticator()
        .verify(&grant.access_token)
        .await
        .map_err(|error| Refusal::bad_request(format!("access token rejected: {error}")))?;
    if claims.subject() != identity.sub {
        return Err(Refusal::bad_request(
            "the access token and the id_token name different subjects",
        ));
    }
    // The checks every request's caller passes, made once more at sign-in so
    // a token no page could act on never becomes a session.
    let caller = claims_to_identity(&claims, None, None, None)
        .map_err(|error| Refusal::bad_request(format!("access token refused: {error}")))?;
    let Subject::User { groups, .. } = caller.subject() else {
        return Err(Refusal::bad_request(
            "a browser signs in a person; the access token is a workload's",
        ));
```

### 27. Method SessionCookies::pending_login
`rust:apps/causeway-server/src/browser_auth/session.rs::SessionCookies::pending_login`  apps/causeway-server/src/browser_auth/session.rs:86-88  pkg:cargo/causeway-server  score 0.72  via `SessionCookies::read` <- CALLS (apps/causeway-server/src/browser_auth/session.rs:87, 1.00)  
`pub(crate) fn pending_login(&self, headers: &HeaderMap) -> Option<PendingLogin>`  
callers 3: `tests::another_key_cannot_read_the_cookie_and_clearing_expires_it` (apps/causeway-server/src/browser_auth/session.rs:196), `tests::another_key_cannot_read_the_cookie_and_clearing_expires_it` (apps/causeway-server/src/browser_auth/session.rs:199), `complete_login` (apps/causeway-server/src/browser_auth/handlers.rs:242)  
calls 1: `SessionCookies::read` (apps/causeway-server/src/browser_auth/session.rs:87)  
tests 1: `tests::another_key_cannot_read_the_cookie_and_clearing_expires_it` (apps/causeway-server/src/browser_auth/session.rs:196)  
uses_types 1: `PendingLogin` (apps/causeway-server/src/browser_auth/session.rs:86)  
apps/causeway-server/src/browser_auth/session.rs:86-88
```rust
    pub(crate) fn pending_login(&self, headers: &HeaderMap) -> Option<PendingLogin> {
        self.read(headers, LOGIN_COOKIE)
    }
```

### 28. Variant LoginOrigin::Move
`rust:apps/causeway-server/src/browser_auth/handlers.rs::LoginOrigin::Move`  apps/causeway-server/src/browser_auth/handlers.rs:55  pkg:cargo/causeway-server  score 0.68  via `LoginOrigin` -> DEFINES (apps/causeway-server/src/browser_auth/handlers.rs:55, 1.00)  
`Move`  
The browser is elsewhere: send it to this `/auth/login` URL.  
referenced_by 1: `login` (apps/causeway-server/src/browser_auth/handlers.rs:143)  

### 29. Test tests::a_session_round_trips_through_the_private_cookie
`rust:apps/causeway-server/src/browser_auth/session.rs::tests::a_session_round_trips_through_the_private_cookie`  apps/causeway-server/src/browser_auth/session.rs:158-179  pkg:cargo/causeway-server  score 0.65  via `BrowserSession` <- USES_TYPE (apps/causeway-server/src/browser_auth/session.rs:161, 1.00)  
`fn a_session_round_trips_through_the_private_cookie()`  
calls 9: `Directory::committed` (apps/causeway-server/src/browser_auth/session.rs:162), `SessionCookies::set_session` (apps/causeway-server/src/browser_auth/session.rs:170), `tests::headers_with` (apps/causeway-server/src/browser_auth/session.rs:177) … 6 more  
tests_for 9: `Directory::committed` (apps/causeway-server/src/browser_auth/session.rs:162), `SessionCookies::new` (apps/causeway-server/src/browser_auth/session.rs:160), `SessionCookies::session` (apps/causeway-server/src/browser_auth/session.rs:177) … 6 more  
uses_types 1: `BrowserSession` (apps/causeway-server/src/browser_auth/session.rs:161)  
apps/causeway-server/src/browser_auth/session.rs:158-179
```rust
    fn a_session_round_trips_through_the_private_cookie() {
        let key = Key::generate();
        let cookies = SessionCookies::new(&key, false);
        let session = BrowserSession {
            subject: causeway_principals::Directory::committed()
                .subject_of("amelia-rhodes")
                .to_owned(),
            handle: Some("amelia-rhodes".to_owned()),
            groups: vec!["catalog-data-owners".to_owned()],
            access_token: "tok".to_owned(),
            expires_at: OffsetDateTime::now_utc().unix_timestamp() + 600,
        };
        let set = cookies.set_session(&session);
        assert_eq!(set.len(), 1);
        assert!(set[0].contains("HttpOnly") && set[0].contains("SameSite=Lax"));
        assert!(
            !set[0].contains("Secure"),
            "plain http keeps the cookie usable"
        );
        let read = cookies.session(&headers_with(&set)).expect("decrypts");
        assert_eq!(read, session);
    }
```

### 30. Variant LoginOrigin::Here
`rust:apps/causeway-server/src/browser_auth/handlers.rs::LoginOrigin::Here`  apps/causeway-server/src/browser_auth/handlers.rs:53  pkg:cargo/causeway-server  score 0.64  via `LoginOrigin` -> DEFINES (apps/causeway-server/src/browser_auth/handlers.rs:53, 1.00)  
`Here`  
The browser is on the callback's origin, or nothing says otherwise.  
referenced_by 5: `tests::a_login_on_the_callbacks_origin_or_with_no_host_stays` (apps/causeway-server/src/browser_auth/handlers.rs:434), `tests::a_login_on_the_callbacks_origin_or_with_no_host_stays` (apps/causeway-server/src/browser_auth/handlers.rs:438), `tests::a_login_on_another_host_moves_to_the_callbacks_origin_once` (apps/causeway-server/src/browser_auth/handlers.rs:421) … 2 more  

### 31. File bootstrap/cw-host/src/modules/http.rs
`file:bootstrap/cw-host/src/modules/http.rs`  bootstrap/cw-host/src/modules/http.rs  pkg:cargo/cw-host  score 0.63  via `set_cookies` <- CONTAINS (bootstrap/cw-host/src/modules/http.rs:115-122, 1.00)  
members 9  
imports 14: `opt_map` (bootstrap/cw-host/src/modules/http.rs:48-50), `opt_int` (bootstrap/cw-host/src/modules/http.rs:48-50), `dynamic_to_json` (bootstrap/cw-host/src/modules/http.rs:48-50) … 11 more  

### 32. File apps/causeway-server/src/routes/cors.rs
`file:apps/causeway-server/src/routes/cors.rs`  apps/causeway-server/src/routes/cors.rs  pkg:cargo/causeway-server  score 0.59  via `ALLOWED_HEADERS` <- CONTAINS (apps/causeway-server/src/routes/cors.rs:24-32, 1.00)  
members 11  
documents 1: `docs/DESIGN-LOG.md` (docs/DESIGN-LOG.md:720)  
imports 4: `cargo/axum` (apps/causeway-server/src/routes/cors.rs:16), `CorsTopology` (apps/causeway-server/src/routes/cors.rs:20), `cargo/causeway-platform` (apps/causeway-server/src/routes/cors.rs:20) … 1 more  

### 33. File .mise/rhai/causeway/personas.rhai
`file:.mise/rhai/causeway/personas.rhai`  .mise/rhai/causeway/personas.rhai  pkg:npm/causeway-monorepo  score 0.59  via `session_headers` <- CONTAINS (.mise/rhai/causeway/personas.rhai:393-403, 1.00)  
members 65  
designs 1: `docs/rfd/RFD-krvga4f3-one-directory-of-principals-for-non-production-environments.md` (docs/rfd/RFD-krvga4f3-one-directory-of-principals-for-non-production-environments.md:844)  
documents 1: `docs/DESIGN-LOG.md` (docs/DESIGN-LOG.md:2458)  

### 34. Function whoami
`rust:apps/causeway-server/src/browser_auth/handlers.rs::whoami`  apps/causeway-server/src/browser_auth/handlers.rs:340-372  pkg:cargo/causeway-server  score 0.57  via `BrowserAuth::secure_cookies` <- CALLS (apps/causeway-server/src/browser_auth/handlers.rs:345, 0.75)  
`async fn whoami(State(state): State<AppState>, headers: HeaderMap) -> Response`  
calls 24: `SessionCookies::new` (apps/causeway-server/src/browser_auth/handlers.rs:345), `caller_from_token` (apps/causeway-server/src/browser_auth/handlers.rs:356), `BrowserAuth::secure_cookies` (apps/causeway-server/src/browser_auth/handlers.rs:345) … 21 more  
uses_types 1: `AppState` (apps/causeway-server/src/browser_auth/handlers.rs:340)  
apps/causeway-server/src/browser_auth/handlers.rs:340-372
```rust
async fn whoami(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let Some(auth) = state.browser_auth() else {
        return Json(serde_json::json!({ "configured": false, "authenticated": false }))
            .into_response();
    };
    let cookies = SessionCookies::new(auth.key(), auth.secure_cookies());
    let now = cookie::time::OffsetDateTime::now_utc().unix_timestamp();
    let session = cookies
        .session(&headers)
        .filter(|session| !session.is_expired(now));
    let Some(session) = session else {
        return Json(serde_json::json!({ "configured": true, "authenticated": false }))
            .into_response();
    };
    // Re-validate rather than trust the cookie alone: a rotated key or a
    // revoked issuer must read as signed out here too.
    if caller_from_token(auth, &session.access_token)
        .await
        .is_none()
    {
        return Json(serde_json::json!({ "configured": true, "authenticated": false }))
            .into_response();
    }
    Json(serde_json::json!({
        "configured": true,
        "authenticated": true,
        "subject": session.subject,
        "handle": session.handle,
        "groups": session.groups,
        "expires_at": session.expires_at,
    }))
    .into_response()
}
```

### 35. Function build_router
`rust:apps/causeway-server/src/routes/mod.rs::build_router`  apps/causeway-server/src/routes/mod.rs:40-120  pkg:cargo/causeway-server  score 0.57  via `attach_browser_caller` <- REFERENCES (apps/causeway-server/src/routes/mod.rs:114, 1.00)  
`pub(crate) fn build_router(state: AppState, access: RouteAccess) -> Router`  
Builds the Axum router with all routes and static asset serving.  
callers 11: `shell` (apps/causeway-server/src/routes/security_headers/tests.rs:67), `shell` (apps/causeway-server/src/routes/cors/tests.rs:74), `Shell::anonymous_router` (apps/causeway-server/src/routes/api/tests.rs:196) … 8 more  
calls 28: `routes` (apps/causeway-server/src/routes/mod.rs:55), `routes` (apps/causeway-server/src/routes/mod.rs:56), `routes` (apps/causeway-server/src/routes/mod.rs:53) … 25 more  
references 3: `handler` (apps/causeway-server/src/routes/mod.rs:66), `attach_browser_caller` (apps/causeway-server/src/routes/mod.rs:114), `guard` (apps/causeway-server/src/routes/mod.rs:107)  
tests 5: `remote_service_ports_smoke_against_rpc_router` (apps/causeway-server/src/composition/remote_services/tests.rs:359), `mounted_semantic_neighborhood_enforces_authorization_and_limits` (apps/causeway-server/src/composition/remote_services/tests.rs:813), `mounted_rights_service_saves_drafts_with_optimistic_concurrency` (apps/causeway-server/src/composition/remote_services/tests.rs:502) … 2 more  
uses_types 2: `RouteAccess` (apps/causeway-server/src/routes/mod.rs:40), `AppState` (apps/causeway-server/src/routes/mod.rs:40)  
apps/causeway-server/src/routes/mod.rs:40-119, cut
```rust
pub(crate) fn build_router(state: AppState, access: RouteAccess) -> Router {
    // The metadata-admin plane's authentication is the state's: startup
    // injected `access`'s there, and every surface that checks the plane
    // reads that one copy.
    let RouteAccess {
        rpc: rpc_access,
        cors,
        security_headers,
        ..
    } = access;
    let metadata_admin = state.metadata_admin_auth().clone();
    let router = Router::new()
        .merge(rpc::routes(&state, &rpc_access, metadata_admin))
        .merge(public_web::routes())
        .merge(crate::browser_auth::routes())
        .merge(system_routes::routes())
        .merge(semantic_artifacts::routes())
        .merge(semantic_catalog::routes())
        .merge(semantic_policies::routes())
        .nest("/app", app::routes())
        .merge(platform_api::routes())
        .merge(agui_api::routes())
        .merge(static_assets::routes(static_assets::static_dir()))
        // Generic relay proxy (ADR 0018): forwards unmatched
        // /<service>/<method> to a configured relay worker; else 404. Lets
        // any relay service route with no per-service Rust.
        .fallback(crate::transport::connect_proxy::handler)
        .layer(axum::Extension(rpc_access));

    let router = router
        // Default Cache-Control. `no-cache` lets browsers store the
        // response but every navigation revalidates against the
        // server (304 on no change). Routes that benefit from
        // long-lived caching (the /static asset tree) override this with
        // their own header; a catalog entry's object is `private, no-store`,
        // since it was decided for one caller.
        // The previous value (`public, max-age=3600`) silently
        // pinned stale HTML in the browser through a deploy and
        // showed up as "wrong breadcrumb links" until the cache
        // expired — never useful for HTML, never default again.
        .layer(SetResponseHeaderLayer::if_not_present(
            header::CACHE_CONTROL,
            HeaderValue::from_static("no-cache, must-revalidate"),
        ))
        // gzip / brotli / deflate / zstd based on Accept-Encoding.
        // Applied at the outermost layer so every handler and
        // ServeDir response gets compressed when the client supports
        // it. Biggest win on HTML + CSS + JS; skipped automatically
        // for images, fonts, and other already-compressed bytes.
        .layer(CompressionLayer::new());
    // Same origin unless the topology's `[cors]` names another; an origin it
    // names is answered with exactly itself and credentials, and no other
    // origin is ever echoed (see `cors`).
    let router = cors.apply(router)
        // Wraps every HTTP request in an INFO-level root span (method,
        // uri, version, matched_path). Defaulting to INFO (not DEBUG)
        // keeps spans visible under RUST_LOG=info — the normal prod
        // and dev log filter — so they reach the OTel export layer.
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO)),
        )
        // Site-profile access guard (ADR 0017): denies `/app/*`, `/status`,
        // and `/roadworthy*` with a `302 → /` when the deployment profile
        // disables them. Authoritative — runs ahead of the handlers.
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            site_profile_guard::guard,
        ))
        // Browser sign-in (RFD-q276nkj9): the session cookie
        // becomes the caller identity pages see. A no-op without a cookie
        // or without `[auth.browser]` in the topology.
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            crate::browser_auth::attach_browser_caller,
        ));
    // Outermost, so every answer carries the security headers, a refusal
    // from the layers above included, and every handler renders inside the
    // response's nonce scope (see `security_headers`).
    security_headers.apply(router).with_state(state)
```

### 36. Function login
`rust:apps/causeway-server/src/browser_auth/handlers.rs::login`  apps/causeway-server/src/browser_auth/handlers.rs:134-178  pkg:cargo/causeway-server  score 0.56  via `BrowserAuth::secure_cookies` <- CALLS (apps/causeway-server/src/browser_auth/handlers.rs:176, 0.75)  
`async fn login( State(state): State<AppState>, headers: HeaderMap, Query(query): Query<LoginQuery>, ) -> Response`  
calls 26: `not_configured` (apps/causeway-server/src/browser_auth/handlers.rs:140), `requested_host` (apps/causeway-server/src/browser_auth/handlers.rs:142), `AuthorizationRequest::new` (apps/causeway-server/src/browser_auth/handlers.rs:163) … 23 more  
references 1: `LoginOrigin::Move` (apps/causeway-server/src/browser_auth/handlers.rs:143)  
uses_types 3: `LoginQuery` (apps/causeway-server/src/browser_auth/handlers.rs:137), `PendingLogin` (apps/causeway-server/src/browser_auth/handlers.rs:154), `AppState` (apps/causeway-server/src/browser_auth/handlers.rs:135)  
apps/causeway-server/src/browser_auth/handlers.rs:134-178
```rust
async fn login(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<LoginQuery>,
) -> Response {
    let Some(auth) = state.browser_auth() else {
        return not_configured();
    };
    let host = requested_host(&headers);
    if let LoginOrigin::Move(location) =
        LoginOrigin::of(&auth.topology().redirect_base, host, &query)
    {
        tracing::info!(
            host = host.unwrap_or_default(),
            redirect_base = %auth.topology().redirect_base,
            "browser sign-in moved to the callback's origin"
        );
        return redirect_with_cookies(&location, []);
    }
    let pkce = Pkce::generate();
    let pending = PendingLogin {
        state: random_token(),
        nonce: random_token(),
        verifier: pkce.verifier().to_owned(),
        return_to: safe_return_to(query.return_to.as_deref()),
    };
    let topology = auth.topology();
    let endpoint = topology.authorization_endpoint();
    let redirect_uri = topology.redirect_uri();
    let url = AuthorizationRequest::new(
        ClientParams::new(
            &endpoint,
            &topology.client_id,
            &redirect_uri,
            auth.scopes(),
            topology.audience.as_str(),
        ),
        &pending.state,
        &pending.nonce,
        pkce.challenge(),
    )
    .url();
    let cookies = SessionCookies::new(auth.key(), auth.secure_cookies());
    redirect_with_cookies(&url, cookies.set_pending_login(&pending))
}
```

### 37. Const SESSION_COOKIE
`rust:apps/causeway-server/src/browser_auth/session.rs::SESSION_COOKIE`  apps/causeway-server/src/browser_auth/session.rs:11  pkg:cargo/causeway-server  score 0.54  via `apps/causeway-server/src/browser_auth/middleware.rs` -> IMPORTS (apps/causeway-server/src/browser_auth/middleware.rs:13, 1.00)  
`pub(crate) const SESSION_COOKIE: &str = "causeway_session";`  
Name of the session cookie.  
importers 2: `apps/causeway-server/src/browser_auth/middleware.rs` (apps/causeway-server/src/browser_auth/middleware.rs:13), `apps/causeway-server/src/browser_auth/handlers.rs` (apps/causeway-server/src/browser_auth/handlers.rs:17)  
apps/causeway-server/src/browser_auth/session.rs:11-11
```rust
pub(crate) const SESSION_COOKIE: &str = "causeway_session";
```

### 38. Function caller_from_token
`rust:apps/causeway-server/src/browser_auth/middleware.rs::caller_from_token`  apps/causeway-server/src/browser_auth/middleware.rs:254-269  pkg:cargo/causeway-server  score 0.54  via `attach_browser_caller` -> CALLS (apps/causeway-server/src/browser_auth/middleware.rs:189, 1.00)  
`pub(crate) async fn caller_from_token( auth: &super::BrowserAuth, access_token: &str, ) -> Option<Access<Authenticated>>`  
The access proof an access token earns, or `None` when it no longer (doc continues, 412 bytes)  
callers 5: `attach_browser_caller` (apps/causeway-server/src/browser_auth/middleware.rs:189), `the_browser_and_the_rpc_point_refuse_the_same_claims` (apps/causeway-server/src/browser_auth/tests.rs:587), `the_browser_and_the_rpc_point_refuse_the_same_claims` (apps/causeway-server/src/browser_auth/tests.rs:549) … 2 more  
calls 10: `BearerCredential::parse` (apps/causeway-server/src/browser_auth/middleware.rs:263), `AuthenticationRequest::bearer` (apps/causeway-server/src/browser_auth/middleware.rs:266), `key_id` (apps/causeway-server/src/browser_auth/middleware.rs:258) … 7 more  
importers 2: `apps/causeway-server/src/browser_auth/handlers.rs` (apps/causeway-server/src/browser_auth/handlers.rs:18), `apps/causeway-server/src/browser_auth/mod.rs` (apps/causeway-server/src/browser_auth/mod.rs:33)  
tests 1: `the_browser_and_the_rpc_point_refuse_the_same_claims` (apps/causeway-server/src/browser_auth/tests.rs:549)  
uses_types 3: `BrowserAuth` (apps/causeway-server/src/browser_auth/middleware.rs:255), `Authenticated` (apps/causeway-server/src/browser_auth/middleware.rs:257), `Access` (apps/causeway-server/src/browser_auth/middleware.rs:257)  
apps/causeway-server/src/browser_auth/middleware.rs:254-269
```rust
pub(crate) async fn caller_from_token(
    auth: &super::BrowserAuth,
    access_token: &str,
) -> Option<Access<Authenticated>> {
    let kid = causeway_oidc::code_flow::key_id(access_token);
    if let Err(error) = auth.jwks(kid.as_deref()).await {
        tracing::warn!(%error, "browser sign-in: issuer keys unavailable");
        return None;
    }
    let credential = BearerCredential::parse(access_token).ok()?;
    let authentication = auth.authentication();
    authentication
        .authenticate(authentication.begin(AuthenticationRequest::bearer(Some(credential))))
        .inspect_err(|error| tracing::debug!(%error, "browser session token refused"))
        .ok()
}
```

### 39. Test tests::another_key_cannot_read_the_cookie_and_clearing_expires_it
`rust:apps/causeway-server/src/browser_auth/session.rs::tests::another_key_cannot_read_the_cookie_and_clearing_expires_it`  apps/causeway-server/src/browser_auth/session.rs:182-203  pkg:cargo/causeway-server  score 0.54  via `SessionCookies::clear` -> TESTED_BY (apps/causeway-server/src/browser_auth/session.rs:200, 1.00)  
`fn another_key_cannot_read_the_cookie_and_clearing_expires_it()`  
calls 8: `SessionCookies::pending_login` (apps/causeway-server/src/browser_auth/session.rs:196), `SessionCookies::clear` (apps/causeway-server/src/browser_auth/session.rs:200), `SessionCookies::pending_login` (apps/causeway-server/src/browser_auth/session.rs:199) … 5 more  
tests_for 5: `SessionCookies::new` (apps/causeway-server/src/browser_auth/session.rs:184), `SessionCookies::pending_login` (apps/causeway-server/src/browser_auth/session.rs:196), `SessionCookies::set_pending_login` (apps/causeway-server/src/browser_auth/session.rs:191) … 2 more  
uses_types 1: `PendingLogin` (apps/causeway-server/src/browser_auth/session.rs:185)  
apps/causeway-server/src/browser_auth/session.rs:182-203
```rust
    fn another_key_cannot_read_the_cookie_and_clearing_expires_it() {
        let key = Key::generate();
        let cookies = SessionCookies::new(&key, true);
        let login = PendingLogin {
            state: "s".into(),
            nonce: "n".into(),
            verifier: "v".into(),
            return_to: "/app".into(),
        };
        let set = cookies.set_pending_login(&login);
        assert!(set[0].contains("Secure"));
        let other = Key::generate();
        assert!(
            SessionCookies::new(&other, true)
                .pending_login(&headers_with(&set))
                .is_none()
        );
        assert_eq!(cookies.pending_login(&headers_with(&set)), Some(login));
        let cleared = cookies.clear(LOGIN_COOKIE);
        assert!(cleared[0].starts_with("causeway_login=;"), "{}", cleared[0]);
        assert!(cleared[0].contains("Max-Age=0"), "{}", cleared[0]);
    }
```

### 40. Function sign_in_required
`rust:apps/causeway-server/src/browser_auth/middleware.rs::sign_in_required`  apps/causeway-server/src/browser_auth/middleware.rs:136-138  pkg:cargo/causeway-server  score 0.53  via `attach_browser_caller` -> CALLS (apps/causeway-server/src/browser_auth/middleware.rs:202, 1.00)  
`fn sign_in_required(request: &Request) -> Response`  
The refusal for an anonymous request to a gated path: a navigation is (doc continues, 134 bytes)  
callers 2: `attach_browser_caller` (apps/causeway-server/src/browser_auth/middleware.rs:202), `attach_browser_caller` (apps/causeway-server/src/browser_auth/middleware.rs:181)  
calls 6: `sign_in_response` (apps/causeway-server/src/browser_auth/middleware.rs:137), `TestCaller::headers` (apps/causeway-server/src/browser_auth/middleware.rs:137), `DecidedAtlasCall::method` (apps/causeway-server/src/browser_auth/middleware.rs:137) … 3 more  
apps/causeway-server/src/browser_auth/middleware.rs:136-138
```rust
fn sign_in_required(request: &Request) -> Response {
    sign_in_response(request.method(), request.uri(), request.headers())
}
```

### 41. Package cargo/causeway-server
`pkg:cargo/causeway-server`  apps/causeway-server/Cargo.toml:2  pkg:cargo/causeway-server  score 0.52  via `apps/causeway-server/src/browser_auth/handlers.rs` <- CONTAINS (apps/causeway-server/Cargo.toml:2, 1.00)  
`cargo/causeway-server`  
members 270  
dependencies 122: `cargo/mokta` (apps/causeway-server/Cargo.toml:182), `cargo/reqwest` (apps/causeway-server/Cargo.toml:165), `cargo/workflow-read-model` (apps/causeway-server/Cargo.toml:169) … 119 more  
designs 4: `docs/adrs/ADR-nbmt040e-cross-slice-read-boundaries.md` (docs/adrs/ADR-nbmt040e-cross-slice-read-boundaries.md:19), `docs/rfd/RFD-fpg4tnq2-ephemeral-infrastructure-for-ci-integration-tests.md` (docs/rfd/RFD-fpg4tnq2-ephemeral-infrastructure-for-ci-integration-tests.md:322), `docs/rfd/RFD-c4jbh7qa-typescript-and-node-web-tier-over-the-rust-core.md` (docs/rfd/RFD-c4jbh7qa-typescript-and-node-web-tier-over-the-rust-core.md:14) … 1 more  
documents 7: `docs/archive/requirements/REQ-011-rust-binary-classification-and-target-workspace-layout.md` (docs/archive/requirements/REQ-011-rust-binary-classification-and-target-workspace-layout.md:78), `docs/plans/SERVICE-BOUNDARIES-WORKPLAN.md` (docs/plans/SERVICE-BOUNDARIES-WORKPLAN.md:43), `docs/plans/RESTATE-REMEDIATION-WORKPLAN.md` (docs/plans/RESTATE-REMEDIATION-WORKPLAN.md:553) … 4 more  
importers 1: `apps/causeway-server/src/main.rs` (apps/causeway-server/src/main.rs:4)  

### 42. File apps/causeway-server/src/browser_auth/tests.rs
`file:apps/causeway-server/src/browser_auth/tests.rs`  apps/causeway-server/src/browser_auth/tests.rs  pkg:cargo/causeway-server  score 0.51  via `attach_browser_caller` <- IMPORTS (apps/causeway-server/src/browser_auth/tests.rs:18-20, 1.00)  
members 16  
documents 1: `docs/DESIGN-LOG.md` (docs/DESIGN-LOG.md:92)  
imports 16: `AppState` (apps/causeway-server/src/browser_auth/tests.rs:21), `cargo/causeway-platform` (apps/causeway-server/src/browser_auth/tests.rs:13), `cargo/causeway-principal` (apps/causeway-server/src/browser_auth/tests.rs:14) … 13 more  

### 43. Document Design log
`doc:docs/DESIGN-LOG.md`  docs/DESIGN-LOG.md:1-3800  pkg:npm/causeway-monorepo  score 0.51  via `apps/causeway-server/src/browser_auth/handlers.rs` -> DOCUMENTED_BY (docs/DESIGN-LOG.md:90, 1.00)  
`Design log`  
<!-- Generated by `mise run governance:design-log` from `Design:` commit trailers (RFD-vjzs0qzv). Do not edit; regenerate. -->  
headings: ADR 25frqh42: ADR-0023: Immutable composite gateway policy bundles
RFD 2n2tcfe2: Operational resilience: stuck work, poison records, and the operator's surface
ADR bn90xr5m: Authentication types make illegal callers unrepresentable
RFD d4v9pg0h: Identity, authorization, and enforcement
RFD fpg4tnq2: Ephemeral infrastructure for CI integration tests
RFD j1jabh49: Environment, scope and secret confi …  
documents_for 1186: `torrentfx/packages-node/causeway-platform/src/secret.ts` (docs/DESIGN-LOG.md:1726), `crates/session-domain/src/session.rs` (docs/DESIGN-LOG.md:621), `apps/causeway-server/src/routes/pages/catalog/list/tests.rs` (docs/DESIGN-LOG.md:729) … 1183 more  
docs/DESIGN-LOG.md:1-80, cut
```markdown
# Design log

<!-- Generated by `mise run governance:design-log` from `Design:` commit
trailers (RFD-vjzs0qzv). Do not edit; regenerate. -->

Every commit that carries a `Design: <uid>` trailer, grouped by the
design document it implements. A commit may implement more than one
document and then appears under each.

## ADR 25frqh42: ADR-0023: Immutable composite gateway policy bundles

- `c7484d85fea1` 2026-10-08 chore(lint): Flight pin matches its source, relay tests ignore the shell, repo-wide lint is clean (#973)
  - `.github/workflows/ci.yml`
  - `.mise/scripts/drive-ffi-py-admin-notebook-smoke.py`
  - `.mise/scripts/drive-ffi-py-notebook-smoke.py`
  - `biome.json`
  - `crates/policy-gateway/tests/flight_query_pin.rs`
  - `docs/research/license-policy/odrl/validate.py`
  - `docs/rfd/RFD-j1jabh49-environment-scope-and-secret-configuration-contract.md`
  - `docs/runbooks/cedar-policy-bundles.md`
  - `ffi/node-drive/index.d.ts`
  - `ffi/node-drive/index.js`
  - `ffi/node-drive/scripts/pack-platform.cjs`
  - `ffi/node-drive/scripts/platform-targets.cjs`
  - `ffi/node-drive/scripts/prepare-native.cjs`
  - `ffi/node-drive/tests/index.test.cjs`
  - `ffi/node-drive/tests/live-admin-smoke.cjs`
  - `ffi/node-drive/tests/live-smoke.cjs`
  - `ffi/node-drive/tests/native-smoke.cjs`
  - `ffi/node-drive/tests/package-smoke.cjs`
  - `ffi/py-drive/examples/admin_metadata_walkthrough.ipynb`
  - `ffi/py-drive/examples/analyst_catalog_walkthrough.ipynb`
  - `ffi/py-drive/tests/live_admin_smoke.py`
  - `ffi/py-drive/tests/live_smoke.py`
  - `ffi/py-drive/tests/native_smoke.py`
  - `ffi/py-drive/tests/test_drive.py`
  - `jobs-py/relay-worker/src/handlers.py`
  - `jobs-py/relay-worker/src/main.py`
  - `jobs-py/relay-worker/tests/test_scoping.py`
  - `policy/rights/validate.py`
  - `services-node/roadmap-relay/data/roadmap.json`
  - `services-py/relay-grpc/src/analytics.py`
  - `services-py/relay-grpc/src/bridge_service.py`
  - `services-py/relay-grpc/src/flight_client.py`
  - `services-py/relay-restate/src/workflow.py`
  - `services-py/snowflake-relay/src/entitlements.py`
  - `torrentfx/conformance/configuration-contract/topology-decode.json`
  - `torrentfx/packages-py/causeway-platform/src/causeway_platform/shutdown.py`
  - `torrentfx/packages-py/causeway-platform/tests/conftest.py`
  - `torrentfx/packages-py/causeway-relay/tests/conftest.py`
  - `torrentfx/packages-py/causeway-relay/tests/test_app.py`

## RFD 2n2tcfe2: Operational resilience: stuck work, poison records, and the operator's surface

- `c6f0b162c838` 2026-09-28 docs(governance): the resilience RFD's evidence names what implements it (#807)
  - `docs/rfd/RFD-2n2tcfe2-operational-resilience-and-the-stuck-work-surface.md`
- `dbb9b0266d96` 2026-09-28 chore(component): the component workspace is clippy-clean (#809)
  - `component/catalog/core/src/bindings.rs`
  - `component/catalog/core/src/placements.rs`
  - `component/catalog/core/src/semantic_read.rs`
  - `component/governance/policy/src/corrective.rs`
  - `component/governance/policy/src/repo.rs`
  - `component/governance/policy/src/rpc.rs`
  - `component/governance/policy/tests/phase3.rs`
  - `component/governance/policy/tests/phase4.rs`
  - `component/governance/terms/src/lib.rs`
  - `component/metadata/core/src/claim_query.rs`
  - `component/metadata/core/src/iceberg_rest.rs`
  - `component/metadata/core/src/postgres.rs`
  - `component/people/core/src/lib.rs`
  - `component/people/core/src/records.rs`
  - `component/platform/event-operations/src/stuck_work.rs`
  - `component/platform/supervision/src/pass.rs`
  - `component/semantic/graph/src/lib.rs`
  - `component/semantic/graph/src/presentation.rs`
  - `component/semantic/graph/src/projection_source.rs`
- `88f9a055c001` 2026-09-29 fix(policy): a completed scan says so, and compiles once per lease (#818)
  - `apps/causeway-policy-scan/src/main.rs`
  - `component/governance/policy/src/detective.rs`
  - `component/governance/policy/src/lib.rs`
```

### 44. Function redirect_with_cookies
`rust:apps/causeway-server/src/browser_auth/handlers.rs::redirect_with_cookies`  apps/causeway-server/src/browser_auth/handlers.rs:113-122  pkg:cargo/causeway-server  score 0.48  via `logout` -> CALLS (apps/causeway-server/src/browser_auth/handlers.rs:337, 1.00)  
`fn redirect_with_cookies(location: &str, cookies: impl IntoIterator<Item = String>) -> Response`  
callers 4: `callback` (apps/causeway-server/src/browser_auth/handlers.rs:194), `login` (apps/causeway-server/src/browser_auth/handlers.rs:151), `login` (apps/causeway-server/src/browser_auth/handlers.rs:177) … 1 more  
calls 4: `ArrowChunkHeaderV1::append` (apps/causeway-server/src/browser_auth/handlers.rs:118), `ContentSourceHttpError::into_response` (apps/causeway-server/src/browser_auth/handlers.rs:115), `OAuthError::into_response` (apps/causeway-server/src/browser_auth/handlers.rs:115) … 1 more  
uses_types 5: `'a Groups::Item` (apps/causeway-server/src/browser_auth/handlers.rs:113), `sidenav::Item` (apps/causeway-server/src/browser_auth/handlers.rs:113), `EmptyStream::Item` (apps/causeway-server/src/browser_auth/handlers.rs:113) … 2 more  
apps/causeway-server/src/browser_auth/handlers.rs:113-122
```rust
fn redirect_with_cookies(location: &str, cookies: impl IntoIterator<Item = String>) -> Response {
    let mut response =
        (StatusCode::FOUND, [(header::LOCATION, location.to_owned())]).into_response();
    for value in cookies {
        if let Ok(value) = HeaderValue::from_str(&value) {
            response.headers_mut().append(header::SET_COOKIE, value);
        }
    }
    response
}
```

### 45. Function anonymous
`rust:apps/causeway-server/src/browser_auth/middleware.rs::anonymous`  apps/causeway-server/src/browser_auth/middleware.rs:218-223  pkg:cargo/causeway-server  score 0.47  via `attach_browser_caller` -> CALLS (apps/causeway-server/src/browser_auth/middleware.rs:204, 1.00)  
`async fn anonymous(mut request: Request, next: Next) -> Response`  
Runs `request` with nobody signed in, saying so on the request and in (doc continues, 81 bytes)  
callers 3: `attach_browser_caller` (apps/causeway-server/src/browser_auth/middleware.rs:204), `attach_browser_caller` (apps/causeway-server/src/browser_auth/middleware.rs:183), `attach_browser_caller` (apps/causeway-server/src/browser_auth/middleware.rs:175)  
calls 3: `SemanticIdentityReconciliation::scope` (apps/causeway-server/src/browser_auth/middleware.rs:221), `Server::run` (apps/causeway-server/src/browser_auth/middleware.rs:221), `run` (apps/causeway-server/src/browser_auth/middleware.rs:221)  
references 2: `BrowserCaller::Anonymous` (apps/causeway-server/src/browser_auth/middleware.rs:219), `BrowserCaller::Anonymous` (apps/causeway-server/src/browser_auth/middleware.rs:221)  
apps/causeway-server/src/browser_auth/middleware.rs:218-223
```rust
async fn anonymous(mut request: Request, next: Next) -> Response {
    request.extensions_mut().insert(BrowserCaller::Anonymous);
    CURRENT_CALLER
        .scope(BrowserCaller::Anonymous, next.run(request))
        .await
}
```

### 46. Function build
`rust:bootstrap/cw-host/src/modules/http.rs::build`  bootstrap/cw-host/src/modules/http.rs:61-99  pkg:cargo/cw-host  score 0.44  via `request` <- CALLS (bootstrap/cw-host/src/modules/http.rs:87, 1.00)  
`fn build(opts: &Map, whole_body: bool) -> Res<(String, Duration, reqwest::RequestBuilder)>`  
The request `opts` describe, and its timeout. A streamed request (doc continues, 171 bytes)  
callers 4: `open` (bootstrap/cw-host/src/modules/http.rs:155), `request` (bootstrap/cw-host/src/modules/http.rs:125), `runtime` (bootstrap/cw-host/src/runtime.rs:20) … 1 more  
calls 14: `dynamic_to_json` (bootstrap/cw-host/src/modules/http.rs:94), `request` (bootstrap/cw-host/src/modules/http.rs:87), `opt_bool` (bootstrap/cw-host/src/modules/http.rs:73) … 11 more  
uses_types 1: `Res` (bootstrap/cw-host/src/modules/http.rs:61)  

### 47. Function err
`rust:bootstrap/cw-host/src/convert.rs::err`  bootstrap/cw-host/src/convert.rs:13-15  pkg:cargo/cw-host  score 0.43  via `open` -> CALLS (bootstrap/cw-host/src/modules/http.rs:160, 1.00)  
`pub fn err<T>(msg: impl Into<String>) -> Res<T>`  
A runtime error carrying a plain message.  
callers 108: `param` (bootstrap/cw-host/src/modules/pg.rs:136), `send` (bootstrap/cw-host/src/modules/proc.rs:530), `param` (bootstrap/cw-host/src/modules/pg.rs:142) … 105 more  
importers 12: `bootstrap/cw-host/src/modules/re.rs` (bootstrap/cw-host/src/modules/re.rs:25), `bootstrap/cw-host/src/modules/k8s.rs` (bootstrap/cw-host/src/modules/k8s.rs:70-72), `bootstrap/cw-host/src/modules/hash.rs` (bootstrap/cw-host/src/modules/hash.rs:14) … 9 more  
tests 10: `tests::a_reservation_of_nothing_is_refused` (crates/ingest-store/src/budget.rs:351), `outside_local_an_issuer_variable_that_differs_from_the_topology_stops_startup` (apps/causeway-metadata-admin-host/tests/auth_requirement.rs:256), `the_removed_selector_cannot_make_a_prod_process_local` (apps/causeway-metadata-admin-host/tests/auth_requirement.rs:338) … 7 more  
uses_types 1: `Res` (bootstrap/cw-host/src/convert.rs:13)  
bootstrap/cw-host/src/convert.rs:13-15
```rust
pub fn err<T>(msg: impl Into<String>) -> Res<T> {
    Err(msg.into().into())
}
```

### 48. Function tests::headers_with
`rust:apps/causeway-server/src/browser_auth/session.rs::tests::headers_with`  apps/causeway-server/src/browser_auth/session.rs:144-155  pkg:cargo/causeway-server  score 0.43  via `tests::another_key_cannot_read_the_cookie_and_clearing_expires_it` -> CALLS (apps/causeway-server/src/browser_auth/session.rs:199, 1.00)  
`fn headers_with(set_cookies: &[String]) -> HeaderMap`  
callers 3: `tests::a_session_round_trips_through_the_private_cookie` (apps/causeway-server/src/browser_auth/session.rs:177), `tests::another_key_cannot_read_the_cookie_and_clearing_expires_it` (apps/causeway-server/src/browser_auth/session.rs:199), `tests::another_key_cannot_read_the_cookie_and_clearing_expires_it` (apps/causeway-server/src/browser_auth/session.rs:196)  
tests 2: `tests::a_session_round_trips_through_the_private_cookie` (apps/causeway-server/src/browser_auth/session.rs:177), `tests::another_key_cannot_read_the_cookie_and_clearing_expires_it` (apps/causeway-server/src/browser_auth/session.rs:196)  

### 49. Method SessionCookies::set_pending_login
`rust:apps/causeway-server/src/browser_auth/session.rs::SessionCookies::set_pending_login`  apps/causeway-server/src/browser_auth/session.rs:115-126  pkg:cargo/causeway-server  score 0.42  via `tests::another_key_cannot_read_the_cookie_and_clearing_expires_it` -> CALLS (apps/causeway-server/src/browser_auth/session.rs:191, 1.00)  
`pub(crate) fn set_pending_login(&self, login: &PendingLogin) -> Vec<String>`  
`Set-Cookie` values that store an in-flight login.  
callers 2: `tests::another_key_cannot_read_the_cookie_and_clearing_expires_it` (apps/causeway-server/src/browser_auth/session.rs:191), `login` (apps/causeway-server/src/browser_auth/handlers.rs:177)  
calls 1: `SessionCookies::build` (apps/causeway-server/src/browser_auth/session.rs:118)  
tests 1: `tests::another_key_cannot_read_the_cookie_and_clearing_expires_it` (apps/causeway-server/src/browser_auth/session.rs:191)  
uses_types 1: `PendingLogin` (apps/causeway-server/src/browser_auth/session.rs:115)  

### 50. Function requires_sign_in
`rust:apps/causeway-server/src/browser_auth/middleware.rs::requires_sign_in`  apps/causeway-server/src/browser_auth/middleware.rs:130-132  pkg:cargo/causeway-server  score 0.41  via `attach_browser_caller` -> CALLS (apps/causeway-server/src/browser_auth/middleware.rs:178, 1.00)  
`fn requires_sign_in(path: &str) -> bool`  
Whether an unauthenticated request to this path is refused when a (doc continues, 244 bytes)  
callers 1: `attach_browser_caller` (apps/causeway-server/src/browser_auth/middleware.rs:178)  

### 51. Function logout
`rust:apps/causeway-server/src/browser_auth/handlers.rs::logout`  apps/causeway-server/src/browser_auth/handlers.rs:330-338  pkg:cargo/causeway-server  score 0.40  via `BrowserAuth::secure_cookies` <- CALLS (apps/causeway-server/src/browser_auth/handlers.rs:334, 0.75)  
`async fn logout(State(state): State<AppState>) -> Response`  
calls 9: `SessionCookies::new` (apps/causeway-server/src/browser_auth/handlers.rs:334), `not_configured` (apps/causeway-server/src/browser_auth/handlers.rs:332), `redirect_with_cookies` (apps/causeway-server/src/browser_auth/handlers.rs:337) … 6 more  
uses_types 1: `AppState` (apps/causeway-server/src/browser_auth/handlers.rs:330)  

### 52. Method LoginOrigin::of
`rust:apps/causeway-server/src/browser_auth/handlers.rs::LoginOrigin::of`  apps/causeway-server/src/browser_auth/handlers.rs:65-83  pkg:cargo/causeway-server  score 0.39  via `LoginOrigin::Here` <- REFERENCES (apps/causeway-server/src/browser_auth/handlers.rs:81, 1.00)  
`fn of(redirect_base: &str, host: Option<&str>, query: &LoginQuery) -> Self`  
Decides from the redirect base, the host the browser asked for, and (doc continues, 229 bytes)  
callers 7: `tests::a_login_on_the_callbacks_origin_or_with_no_host_stays` (apps/causeway-server/src/browser_auth/handlers.rs:429), `tests::a_login_on_another_host_moves_to_the_callbacks_origin_once` (apps/causeway-server/src/browser_auth/handlers.rs:397), `tests::a_login_on_another_host_moves_to_the_callbacks_origin_once` (apps/causeway-server/src/browser_auth/handlers.rs:420) … 4 more  
calls 4: `safe_return_to` (apps/causeway-server/src/browser_auth/handlers.rs:75), `IdentityScopeParseError::authority` (apps/causeway-server/src/browser_auth/handlers.rs:70), `IdentityScope::authority` (apps/causeway-server/src/browser_auth/handlers.rs:70) … 1 more  
references 1: `LoginOrigin::Here` (apps/causeway-server/src/browser_auth/handlers.rs:81)  
tests 2: `tests::a_login_on_the_callbacks_origin_or_with_no_host_stays` (apps/causeway-server/src/browser_auth/handlers.rs:429), `tests::a_login_on_another_host_moves_to_the_callbacks_origin_once` (apps/causeway-server/src/browser_auth/handlers.rs:397)  
uses_types 1: `LoginQuery` (apps/causeway-server/src/browser_auth/handlers.rs:65)  

### 53. Function open
`rust:bootstrap/cw-host/src/modules/http.rs::open`  bootstrap/cw-host/src/modules/http.rs:154-188  pkg:cargo/cw-host  score 0.39  via `set_cookies` <- CALLS (bootstrap/cw-host/src/modules/http.rs:172, 1.00)  
`fn open(opts: &Map) -> Res<Map>`  
Send the request and return once the headers arrive, leaving the body (doc continues, 99 bytes)  
callers 2: `open` (bootstrap/cw-host/src/modules/http.rs:208), `open` (bootstrap/cw-host/src/modules/http.rs:212)  
calls 12: `set_cookies` (bootstrap/cw-host/src/modules/http.rs:172), `build` (bootstrap/cw-host/src/modules/http.rs:155), `ByteStream::channel` (bootstrap/cw-host/src/modules/http.rs:173) … 9 more  
uses_types 1: `Res` (bootstrap/cw-host/src/modules/http.rs:154)  

### 54. Function request
`rust:bootstrap/cw-host/src/modules/http.rs::request`  bootstrap/cw-host/src/modules/http.rs:124-150  pkg:cargo/cw-host  score 0.39  via `set_cookies` <- CALLS (bootstrap/cw-host/src/modules/http.rs:130, 1.00)  
`fn request(opts: &Map) -> Res<Map>`  
callers 4: `build` (bootstrap/cw-host/src/modules/http.rs:87), `request` (bootstrap/cw-host/src/modules/http.rs:221), `with` (bootstrap/cw-host/src/modules/http.rs:202) … 1 more  
calls 8: `block_on` (bootstrap/cw-host/src/modules/http.rs:126), `err` (bootstrap/cw-host/src/modules/http.rs:139), `set_cookies` (bootstrap/cw-host/src/modules/http.rs:130) … 5 more  
uses_types 3: `Res` (bootstrap/cw-host/src/modules/http.rs:124), `Json` (bootstrap/cw-host/src/modules/http.rs:144), `Json` (bootstrap/cw-host/src/modules/http.rs:144)  

### 55. Function browser_caller
`rust:apps/causeway-server/src/browser_auth/middleware.rs::browser_caller`  apps/causeway-server/src/browser_auth/middleware.rs:55-59  pkg:cargo/causeway-server  score 0.38  via `apps/causeway-server/src/browser_auth/middleware.rs` -> CONTAINS (apps/causeway-server/src/browser_auth/middleware.rs:55-59, 1.00)  
`pub(crate) fn browser_caller( extensions: &axum::http::Extensions, ) -> Result<&BrowserCaller, OutsideRequest>`  
The caller a page request carries, as the middleware attached it. (doc continues, 141 bytes)  
callers 3: `page` (apps/causeway-server/src/browser_auth/tests.rs:106), `handle_frame` (apps/causeway-server/src/ws/router.rs:85), `handle_frame` (apps/causeway-server/src/ws/router.rs:61)  
importers 2: `apps/causeway-server/src/browser_auth/mod.rs` (apps/causeway-server/src/browser_auth/mod.rs:28-31), `apps/causeway-server/src/browser_auth/tests.rs` (apps/causeway-server/src/browser_auth/tests.rs:18-20)  
uses_types 2: `BrowserCaller` (apps/causeway-server/src/browser_auth/middleware.rs:57), `OutsideRequest` (apps/causeway-server/src/browser_auth/middleware.rs:57)  

### 56. Method Refusal::bad_request
`rust:apps/causeway-server/src/browser_auth/handlers.rs::Refusal::bad_request`  apps/causeway-server/src/browser_auth/handlers.rs:216-221  pkg:cargo/causeway-server  score 0.38  via `complete_login` -> CALLS (apps/causeway-server/src/browser_auth/handlers.rs:312, 1.00)  
`fn bad_request(reason: impl Into<String>) -> Self`  
callers 9: `complete_login` (apps/causeway-server/src/browser_auth/handlers.rs:312), `complete_login` (apps/causeway-server/src/browser_auth/handlers.rs:255), `complete_login` (apps/causeway-server/src/browser_auth/handlers.rs:245) … 6 more  

### 57. Package cargo/axum
`pkg:cargo/axum`  pkg:cargo/axum  score 0.38  via `apps/causeway-server/src/browser_auth/middleware.rs` -> IMPORTS (apps/causeway-server/src/browser_auth/middleware.rs:6, 1.00)  
dependents 14: `cargo/causeway-contract` (tools/causeway-contract/Cargo.toml:1-56), `cargo/causeway-controller` (jobs/causeway-controller/Cargo.toml:1-29), `cargo/causeway-metadata-admin-host` (apps/causeway-metadata-admin-host/Cargo.toml:1-59) … 11 more  
importers 129: `apps/causeway-server/src/routes/pages/refused.rs` (apps/causeway-server/src/routes/pages/refused.rs:4), `feed_redirect` (apps/causeway-server/src/routes/public/articles.rs:13), `apps/causeway-server/src/agui/workflow/events.rs` (apps/causeway-server/src/agui/workflow/events.rs:3) … 126 more  

### 58. Method SessionCookies::build
`rust:apps/causeway-server/src/browser_auth/session.rs::SessionCookies::build`  apps/causeway-server/src/browser_auth/session.rs:90-98  pkg:cargo/causeway-server  score 0.38  via `SessionCookies::set_session` -> CALLS (apps/causeway-server/src/browser_auth/session.rs:108, 1.00)  
`fn build<'c>(&self, name: &'c str, value: String, max_age: Duration) -> Cookie<'c>`  
callers 37: `SessionCookies::set_session` (apps/causeway-server/src/browser_auth/session.rs:108), `SessionCookies::set_pending_login` (apps/causeway-server/src/browser_auth/session.rs:118), `an_agent_without_rpc_delegate_reads_no_catalog` (apps/causeway-server/src/agent/tests.rs:298) … 34 more  
tests 11: `delegated_catalog_reads_carry_the_signed_in_reader_and_refuse_anonymous` (apps/causeway-server/src/composition/remote_services/tests.rs:2646), `tests::a_persons_agent_is_not_the_person` (apps/causeway-server/src/ws/session/subscription.rs:232), `tests::arrow_frame_decodes_into_typed_chunk` (apps/causeway-server/src/transport/session_rpc.rs:417) … 8 more  

### 59. Impl LoginOrigin#impl
`rust:apps/causeway-server/src/browser_auth/handlers.rs::LoginOrigin#impl`  apps/causeway-server/src/browser_auth/handlers.rs:58-84  pkg:cargo/causeway-server  score 0.37  via `LoginOrigin` <- USES_TYPE (apps/causeway-server/src/browser_auth/handlers.rs:58-84, 1.00)  
`impl LoginOrigin`  
members 1  
uses_types 1: `LoginOrigin` (apps/causeway-server/src/browser_auth/handlers.rs:58-84)  

### 60. Function is_blank
`rhai:bootstrap/cw-host/src/globals.rs::is_blank`  bootstrap/cw-host/src/globals.rs:110-112  pkg:cargo/cw-host  score 0.37  via `session_headers` -> CALLS (.mise/rhai/causeway/personas.rhai:399, 1.00)  
`engine.register_fn("is_blank", |s: Dynamic| -> bool { s.is_unit() || (s.is_string() && s.to_string().trim().is_empty()) })`  
callers 158: `.mise/scripts/tasks/db/migrate.rhai` (.mise/scripts/tasks/db/migrate.rhai:114), `preset_database` (.mise/rhai/causeway/cluster.rhai:291), `preset_database` (.mise/rhai/causeway/cluster.rhai:281) … 155 more  
calls 1: `is_string` (bootstrap/cw-host/src/globals.rs:111)  

### 61. Struct Refusal
`rust:apps/causeway-server/src/browser_auth/handlers.rs::Refusal`  apps/causeway-server/src/browser_auth/handlers.rs:210-213  pkg:cargo/causeway-server  score 0.37  via `complete_login` -> USES_TYPE (apps/causeway-server/src/browser_auth/handlers.rs:239, 1.00)  
`struct Refusal`  
members 2  
used_by 2: `complete_login` (apps/causeway-server/src/browser_auth/handlers.rs:239), `Refusal#impl` (apps/causeway-server/src/browser_auth/handlers.rs:215-229)  

### 62. Struct PendingLogin
`rust:apps/causeway-server/src/browser_auth/session.rs::PendingLogin`  apps/causeway-server/src/browser_auth/session.rs:41-47  pkg:cargo/causeway-server  score 0.36  via `login` -> USES_TYPE (apps/causeway-server/src/browser_auth/handlers.rs:154, 1.00)  
`pub(crate) struct PendingLogin`  
The login the browser was sent on, kept until it comes back.  
importers 1: `apps/causeway-server/src/browser_auth/handlers.rs` (apps/causeway-server/src/browser_auth/handlers.rs:17)  
used_by 4: `login` (apps/causeway-server/src/browser_auth/handlers.rs:154), `tests::another_key_cannot_read_the_cookie_and_clearing_expires_it` (apps/causeway-server/src/browser_auth/session.rs:185), `SessionCookies::pending_login` (apps/causeway-server/src/browser_auth/session.rs:86) … 1 more  

### 63. Const LOGIN_COOKIE
`rust:apps/causeway-server/src/browser_auth/session.rs::LOGIN_COOKIE`  apps/causeway-server/src/browser_auth/session.rs:13  pkg:cargo/causeway-server  score 0.35  via `apps/causeway-server/src/browser_auth/session.rs` -> CONTAINS (apps/causeway-server/src/browser_auth/session.rs:13, 1.00)  
`pub(crate) const LOGIN_COOKIE: &str = "causeway_login";`  
Name of the in-flight login cookie.  
importers 1: `apps/causeway-server/src/browser_auth/handlers.rs` (apps/causeway-server/src/browser_auth/handlers.rs:17)  

### 64. Test tests::a_login_on_the_callbacks_origin_or_with_no_host_stays
`rust:apps/causeway-server/src/browser_auth/handlers.rs::tests::a_login_on_the_callbacks_origin_or_with_no_host_stays`  apps/causeway-server/src/browser_auth/handlers.rs:426-440  pkg:cargo/causeway-server  score 0.35  via `LoginOrigin::Here` <- REFERENCES (apps/causeway-server/src/browser_auth/handlers.rs:438, 1.00)  
`fn a_login_on_the_callbacks_origin_or_with_no_host_stays()`  
calls 6: `LoginOrigin::of` (apps/causeway-server/src/browser_auth/handlers.rs:429), `LoginOrigin::of` (apps/causeway-server/src/browser_auth/handlers.rs:437), `tests::query` (apps/causeway-server/src/browser_auth/handlers.rs:433) … 3 more  
references 3: `LoginOrigin::Here` (apps/causeway-server/src/browser_auth/handlers.rs:434), `LoginOrigin::Here` (apps/causeway-server/src/browser_auth/handlers.rs:438), `LoginOrigin::Here` (apps/causeway-server/src/browser_auth/handlers.rs:430)  
tests_for 2: `LoginOrigin::of` (apps/causeway-server/src/browser_auth/handlers.rs:429), `tests::query` (apps/causeway-server/src/browser_auth/handlers.rs:429)  

### 65. Enum BrowserCaller
`rust:apps/causeway-server/src/browser_auth/middleware.rs::BrowserCaller`  apps/causeway-server/src/browser_auth/middleware.rs:26-32  pkg:cargo/causeway-server  score 0.33  via `apps/causeway-server/src/browser_auth/middleware.rs` -> CONTAINS (apps/causeway-server/src/browser_auth/middleware.rs:26-32, 1.00)  
`pub(crate) enum BrowserCaller`  
Who is asking, for a request the shell is serving. (doc continues, 257 bytes)  
members 3  
importers 35: `apps/causeway-server/src/routes/pages/governance/placements/tests.rs` (apps/causeway-server/src/routes/pages/governance/placements/tests.rs:6), `apps/causeway-server/src/routes/pages/test_shell.rs` (apps/causeway-server/src/routes/pages/test_shell.rs:18), `apps/causeway-server/src/ws/tests/explorer.rs` (apps/causeway-server/src/ws/tests/explorer.rs:26) … 32 more  
used_by 67: `handle_explorer_action` (apps/causeway-server/src/ws/router.rs:199), `decide_contract_pages` (apps/causeway-server/src/contracts/page.rs:34), `current_browser_caller` (apps/causeway-server/src/browser_auth/middleware.rs:76) … 64 more  

### 66. Struct OutsideRequest
`rust:apps/causeway-server/src/browser_auth/middleware.rs::OutsideRequest`  apps/causeway-server/src/browser_auth/middleware.rs:48  pkg:cargo/causeway-server  score 0.33  via `apps/causeway-server/src/browser_auth/middleware.rs` -> CONTAINS (apps/causeway-server/src/browser_auth/middleware.rs:48, 1.00)  
`pub(crate) struct OutsideRequest;`  
A caller was read where no request scoped one: a task spawned without (doc continues, 132 bytes)  
importers 2: `apps/causeway-server/src/browser_auth/mod.rs` (apps/causeway-server/src/browser_auth/mod.rs:28-31), `apps/causeway-server/src/browser_auth/tests.rs` (apps/causeway-server/src/browser_auth/tests.rs:18-20)  
referenced_by 2: `request_caller` (apps/causeway-server/src/composition/state/feature_flags.rs:25), `RemoteCatalogClient::client_for_caller` (apps/causeway-server/src/composition/remote_services/catalog.rs:126)  
used_by 2: `current_browser_caller` (apps/causeway-server/src/browser_auth/middleware.rs:76), `browser_caller` (apps/causeway-server/src/browser_auth/middleware.rs:57)  

### 67. File .mise/scripts/tasks/vsa/runtime-smoke.rhai
`file:.mise/scripts/tasks/vsa/runtime-smoke.rhai`  .mise/scripts/tasks/vsa/runtime-smoke.rhai  pkg:npm/causeway-monorepo  score 0.33  via `session_headers` <- CALLS (.mise/scripts/tasks/vsa/runtime-smoke.rhai:168, 1.00)  
members 1  
calls 11: `get` (.mise/scripts/tasks/vsa/runtime-smoke.rhai:47), `session_headers` (.mise/scripts/tasks/vsa/runtime-smoke.rhai:168), `browser_session` (.mise/scripts/tasks/vsa/runtime-smoke.rhai:56) … 8 more  
documents 1: `docs/DESIGN-LOG.md` (docs/DESIGN-LOG.md:2110)  
run_by 1: `mise/vsa:runtime:smoke` (.mise/toml/vsa.toml:55)  

### 68. Struct CallbackQuery
`rust:apps/causeway-server/src/browser_auth/handlers.rs::CallbackQuery`  apps/causeway-server/src/browser_auth/handlers.rs:98-103  pkg:cargo/causeway-server  score 0.32  via `complete_login` -> USES_TYPE (apps/causeway-server/src/browser_auth/handlers.rs:238, 1.00)  
`struct CallbackQuery`  
used_by 2: `callback` (apps/causeway-server/src/browser_auth/handlers.rs:183), `complete_login` (apps/causeway-server/src/browser_auth/handlers.rs:238)  

### 69. Function session_key
`rust:apps/causeway-server/src/browser_auth/mod.rs::session_key`  apps/causeway-server/src/browser_auth/mod.rs:223-232  pkg:cargo/causeway-server  score 0.31  via `apps/causeway-server/src/browser_auth/mod.rs` -> CONTAINS (apps/causeway-server/src/browser_auth/mod.rs:223-232, 1.00)  
`fn session_key(raw: &str) -> Result<Key, BrowserAuthStartupError>`  
A cookie key from a mounted secret: 64+ raw bytes, or base64 of them. (doc continues, 178 bytes)  
callers 4: `key_tests::a_short_session_key_is_refused_and_a_long_one_accepted` (apps/causeway-server/src/browser_auth/mod.rs:245), `key_tests::a_short_session_key_is_refused_and_a_long_one_accepted` (apps/causeway-server/src/browser_auth/mod.rs:248), `key_tests::a_short_session_key_is_refused_and_a_long_one_accepted` (apps/causeway-server/src/browser_auth/mod.rs:241) … 1 more  
imports 1: `cargo/base64` (apps/causeway-server/src/browser_auth/mod.rs:224)  
tests 1: `key_tests::a_short_session_key_is_refused_and_a_long_one_accepted` (apps/causeway-server/src/browser_auth/mod.rs:241)  
uses_types 1: `BrowserAuthStartupError` (apps/causeway-server/src/browser_auth/mod.rs:223)  

### 70. File apps/causeway-server/src/composition/state.rs
`file:apps/causeway-server/src/composition/state.rs`  apps/causeway-server/src/composition/state.rs  pkg:cargo/causeway-server  score 0.31  via `AppState` <- CONTAINS (apps/causeway-server/src/composition/state.rs:203-254, 1.00)  
members 29  
designs 1: `docs/rfd/RFD-2cqt6wf1-web-session-state-and-recovery.md` (docs/rfd/RFD-2cqt6wf1-web-session-state-and-recovery.md:83)  
documents 1: `docs/DESIGN-LOG.md` (docs/DESIGN-LOG.md:476)  
imports 26: `PlatformRuntime` (apps/causeway-server/src/composition/state.rs:32), `ObjectStoreUnavailable` (apps/causeway-server/src/composition/state.rs:28), `SemanticRuntime` (apps/causeway-server/src/composition/state.rs:38) … 23 more  

### 71. File apps/causeway-server/src/routes/mod.rs
`file:apps/causeway-server/src/routes/mod.rs`  apps/causeway-server/src/routes/mod.rs  pkg:cargo/causeway-server  score 0.31  via `build_router` <- CONTAINS (apps/causeway-server/src/routes/mod.rs:40-120, 1.00)  
members 20  
documents 1: `docs/DESIGN-LOG.md` (docs/DESIGN-LOG.md:658)  
imports 10: `TestCaller` (apps/causeway-server/src/routes/mod.rs:34), `cargo/tracing` (apps/causeway-server/src/routes/mod.rs:29), `RouteStartupError` (apps/causeway-server/src/routes/mod.rs:35) … 7 more  

### 72. Function current_browser_caller
`rust:apps/causeway-server/src/browser_auth/middleware.rs::current_browser_caller`  apps/causeway-server/src/browser_auth/middleware.rs:76-80  pkg:cargo/causeway-server  score 0.31  via `apps/causeway-server/src/browser_auth/middleware.rs` -> CONTAINS (apps/causeway-server/src/browser_auth/middleware.rs:76-80, 1.00)  
`pub(crate) fn current_browser_caller() -> Result<BrowserCaller, OutsideRequest>`  
The caller of the request being served. (doc continues, 101 bytes)  
callers 6: `RemoteCatalogClient::client_for_caller` (apps/causeway-server/src/composition/remote_services/catalog.rs:118), `outside_a_request_is_not_the_same_as_anonymous` (apps/causeway-server/src/browser_auth/tests.rs:616), `upgrade_caller` (apps/causeway-server/src/browser_auth/middleware.rs:112) … 3 more  
importers 1: `apps/causeway-server/src/browser_auth/mod.rs` (apps/causeway-server/src/browser_auth/mod.rs:28-31)  
tests 1: `outside_a_request_is_not_the_same_as_anonymous` (apps/causeway-server/src/browser_auth/tests.rs:607)  
uses_types 2: `BrowserCaller` (apps/causeway-server/src/browser_auth/middleware.rs:76), `OutsideRequest` (apps/causeway-server/src/browser_auth/middleware.rs:76)  

### 73. Function sign_in_response
`rust:apps/causeway-server/src/browser_auth/middleware.rs::sign_in_response`  apps/causeway-server/src/browser_auth/middleware.rs:144-162  pkg:cargo/causeway-server  score 0.31  via `apps/causeway-server/src/browser_auth/middleware.rs` -> CONTAINS (apps/causeway-server/src/browser_auth/middleware.rs:144-162, 1.00)  
`pub(crate) fn sign_in_response(method: &Method, uri: &Uri, headers: &HeaderMap) -> Response`  
The shell's one answer to a request that needs someone signed in and has (doc continues, 226 bytes)  
callers 6: `catalog_entry_by_id` (apps/causeway-server/src/routes/pages/catalog/by_id.rs:41), `sign_in_required` (apps/causeway-server/src/browser_auth/middleware.rs:137), `refused` (apps/causeway-server/src/routes/pages/refused.rs:82) … 3 more  
calls 7: `percent_encode` (apps/causeway-server/src/browser_auth/middleware.rs:153), `OAuthError::into_response` (apps/causeway-server/src/browser_auth/middleware.rs:155), `ContentSourceHttpError::into_response` (apps/causeway-server/src/browser_auth/middleware.rs:161) … 4 more  
importers 1: `apps/causeway-server/src/browser_auth/mod.rs` (apps/causeway-server/src/browser_auth/mod.rs:28-31)  

### 74. Package cargo/cookie
`pkg:cargo/cookie`  pkg:cargo/cookie  score 0.31  via `apps/causeway-server/src/browser_auth/session.rs` -> IMPORTS (apps/causeway-server/src/browser_auth/session.rs:6, 1.00)  
dependents 1: `cargo/causeway-server` (apps/causeway-server/Cargo.toml:172)  
importers 2: `apps/causeway-server/src/browser_auth/session.rs` (apps/causeway-server/src/browser_auth/session.rs:6), `apps/causeway-server/src/browser_auth/mod.rs` (apps/causeway-server/src/browser_auth/mod.rs:25)  

### 75. Package cargo/causeway-principal
`pkg:cargo/causeway-principal`  crates/causeway-principal/Cargo.toml:2  pkg:cargo/causeway-principal  score 0.30  via `apps/causeway-server/src/browser_auth/middleware.rs` -> IMPORTS (apps/causeway-server/src/browser_auth/middleware.rs:11, 1.00)  
`cargo/causeway-principal`  
members 7  
dependents 30: `cargo/flightdeck-auth` (crates/flightdeck-auth/Cargo.toml:34), `cargo/causeway-catalog-host` (apps/causeway-catalog-host/Cargo.toml:40), `cargo/flightdeck-server` (services/flightdeck/Cargo.toml:73) … 27 more  
dependencies 3: `cargo/thiserror` (crates/causeway-principal/Cargo.toml:1-23), `cargo/serde` (crates/causeway-principal/Cargo.toml:1-23), `cargo/serde_json` (crates/causeway-principal/Cargo.toml:1-23)  
designs 1: `docs/rfd/RFD-d4v9pg0h-identity-authorization-and-enforcement.md` (docs/rfd/RFD-d4v9pg0h-identity-authorization-and-enforcement.md:124)  
importers 81: `crates/platform-flags/tests/group_rules.rs` (crates/platform-flags/tests/group_rules.rs:15), `apps/causeway-server/src/composition/state/platform_runtime.rs` (apps/causeway-server/src/composition/state/platform_runtime.rs:7), `crates/drive-manifest/src/enums.rs` (crates/drive-manifest/src/enums.rs:16) … 78 more  

### 76. Method Shell::anonymous_router
`rust:apps/causeway-server/src/routes/pages/test_shell.rs::Shell::anonymous_router`  apps/causeway-server/src/routes/pages/test_shell.rs:126-128  pkg:cargo/causeway-server  score 0.30  via `build_router` <- CALLS (apps/causeway-server/src/routes/pages/test_shell.rs:127, 1.00)  
`pub(super) fn anonymous_router(&self) -> Router`  
The whole shell, as a browser reaches it: nobody is signed in (doc continues, 99 bytes)  
callers 17: `an_anonymous_caller_is_sent_to_sign_in` (apps/causeway-server/src/routes/pages/governance/placements/tests.rs:43), `an_anonymous_caller_gets_the_sign_in_answer` (apps/causeway-server/src/routes/pages/metadata_ops/trace/tests.rs:83), `an_anonymous_caller_is_sent_to_sign_in_and_sees_nothing` (apps/causeway-server/src/routes/pages/catalog/detail/tests.rs:82) … 14 more  
calls 2: `RouteAccess::for_tests` (apps/causeway-server/src/routes/pages/test_shell.rs:127), `build_router` (apps/causeway-server/src/routes/pages/test_shell.rs:127)  
tests 9: `an_anonymous_caller_is_sent_to_sign_in` (apps/causeway-server/src/routes/pages/catalog/by_id/tests.rs:25), `with_the_pure_flag_on_an_anonymous_caller_is_sent_to_sign_in` (apps/causeway-server/src/routes/pages/catalog/detail/tests.rs:270), `an_anonymous_caller_gets_the_sign_in_answer` (apps/causeway-server/src/routes/pages/metadata_ops/trace/tests.rs:79) … 6 more  

### 77. TypeAlias BrowserAccess
`rust:apps/causeway-server/src/browser_auth/middleware.rs::BrowserAccess`  apps/causeway-server/src/browser_auth/middleware.rs:18  pkg:cargo/causeway-server  score 0.29  via `apps/causeway-server/src/browser_auth/middleware.rs` -> CONTAINS (apps/causeway-server/src/browser_auth/middleware.rs:18, 1.00)  
`pub(crate) type BrowserAccess = Arc<Access<Authenticated>>;`  
The signed-in person's access proof: the session token, verified by the (doc continues, 121 bytes)  
importers 1: `apps/causeway-server/src/browser_auth/mod.rs` (apps/causeway-server/src/browser_auth/mod.rs:28-31)  
used_by 3: `RequestCaller` (apps/causeway-server/src/composition/state/feature_flags.rs:30), `access_for_tests` (apps/causeway-server/src/browser_auth/middleware.rs:86), `BrowserCaller::SignedIn` (apps/causeway-server/src/browser_auth/middleware.rs:31)  
uses_types 2: `Access` (apps/causeway-server/src/browser_auth/middleware.rs:18), `Authenticated` (apps/causeway-server/src/browser_auth/middleware.rs:18)  

### 78. File .mise/rhai/causeway/personas_test.rhai
`file:.mise/rhai/causeway/personas_test.rhai`  .mise/rhai/causeway/personas_test.rhai  pkg:npm/causeway-monorepo  score 0.29  via `test_a_session_adds_its_cookie_and_an_unsigned_one_adds_nothing` <- CONTAINS (.mise/rhai/causeway/personas_test.rhai:362-367, 1.00)  
members 47  
calls 2: `subject_of` (.mise/rhai/causeway/personas_test.rhai:222), `pick` (.mise/rhai/causeway/personas_test.rhai:88)  
designs 1: `docs/rfd/RFD-krvga4f3-one-directory-of-principals-for-non-production-environments.md` (docs/rfd/RFD-krvga4f3-one-directory-of-principals-for-non-production-environments.md:183)  
documents 1: `docs/DESIGN-LOG.md` (docs/DESIGN-LOG.md:2459)  

### 79. Function register
`rust:bootstrap/cw-host/src/globals.rs::register`  bootstrap/cw-host/src/globals.rs:81-153  pkg:cargo/cw-host  score 0.29  via `is_blank` <- DEFINES (bootstrap/cw-host/src/globals.rs:110-112, 1.00)  
`pub fn register(engine: &mut Engine)`  
members 13  
callers 1: `engine` (bootstrap/cw-host/src/lib.rs:95)  

### 80. Function upgrade_caller
`rust:apps/causeway-server/src/browser_auth/middleware.rs::upgrade_caller`  apps/causeway-server/src/browser_auth/middleware.rs:111-116  pkg:cargo/causeway-server  score 0.29  via `apps/causeway-server/src/browser_auth/middleware.rs` -> CONTAINS (apps/causeway-server/src/browser_auth/middleware.rs:111-116, 1.00)  
`pub(crate) fn upgrade_caller() -> BrowserCaller`  
The caller a websocket upgrade carries into its socket task. (doc continues, 224 bytes)  
callers 1: `upgrade` (apps/causeway-server/src/ws.rs:117)  
calls 1: `current_browser_caller` (apps/causeway-server/src/browser_auth/middleware.rs:112)  
importers 1: `apps/causeway-server/src/browser_auth/mod.rs` (apps/causeway-server/src/browser_auth/mod.rs:28-31)  
references 1: `BrowserCaller::Anonymous` (apps/causeway-server/src/browser_auth/middleware.rs:114)  
uses_types 1: `BrowserCaller` (apps/causeway-server/src/browser_auth/middleware.rs:111)  

### 81. Function answer
`rust:apps/causeway-server/src/routes/cors.rs::answer`  apps/causeway-server/src/routes/cors.rs:121-171  pkg:cargo/causeway-server  score 0.29  via `apps/causeway-server/src/routes/cors.rs` -> CONTAINS (apps/causeway-server/src/routes/cors.rs:121-171, 1.00)  
`async fn answer( State(origins): State<Arc<AllowedOrigins>>, request: Request, next: Next, ) -> Response`  
Answers a request under an allow-list. (doc continues, 420 bytes)  
calls 13: `joined` (apps/causeway-server/src/routes/cors.rs:138), `AllowedOrigins::of` (apps/causeway-server/src/routes/cors.rs:127), `joined` (apps/causeway-server/src/routes/cors.rs:142) … 10 more  
uses_types 1: `AllowedOrigins` (apps/causeway-server/src/routes/cors.rs:122)  

### 82. Method Server::run
`rust:apps/causeway-server/src/startup.rs::Server::run`  apps/causeway-server/src/startup.rs:189-271  pkg:cargo/causeway-server  score 0.29  via `build_router` <- CALLS (apps/causeway-server/src/startup.rs:243, 1.00)  
`pub async fn run(self) -> Result<Server<Running>, Failed>`  
Transition to Running: build AppState, router, and serve.  
callers 14: `an_agent_run_needs_a_signed_in_caller_cedar_admits` (apps/causeway-server/src/agui/agent_run/tests.rs:74), `an_agent_run_needs_a_signed_in_caller_cedar_admits` (apps/causeway-server/src/agui/agent_run/tests.rs:87), `answer` (apps/causeway-server/src/routes/cors.rs:151) … 11 more  
calls 14: `service_readiness_from_resources` (apps/causeway-server/src/startup.rs:231), `spawn_database_listeners` (apps/causeway-server/src/startup.rs:240), `set` (apps/causeway-server/src/startup.rs:211) … 11 more  
references 3: `chrome` (apps/causeway-server/src/startup.rs:212), `chrome` (apps/causeway-server/src/startup.rs:212), `chrome` (apps/causeway-server/src/startup.rs:212)  
tests 3: `an_agent_run_needs_a_signed_in_caller_cedar_admits` (apps/causeway-server/src/agui/agent_run/tests.rs:74), `the_test_topology_reads_the_catalog_as_the_signed_in_person` (apps/causeway-server/src/composition/remote_services/tests.rs:2819), `delegated_catalog_reads_carry_the_signed_in_reader_and_refuse_anonymous` (apps/causeway-server/src/composition/remote_services/tests.rs:2591)  
uses_types 4: `Failed` (apps/causeway-server/src/startup.rs:189), `Running` (apps/causeway-server/src/startup.rs:189), `SiteChrome` (apps/causeway-server/src/startup.rs:212) … 1 more  

### 83. Module tests
`rust:apps/causeway-server/src/browser_auth/session.rs::tests`  apps/causeway-server/src/browser_auth/session.rs:140-204  pkg:cargo/causeway-server  score 0.28  via `tests::another_key_cannot_read_the_cookie_and_clearing_expires_it` <- DEFINES (apps/causeway-server/src/browser_auth/session.rs:182-203, 1.00)  
`mod tests`  
members 3  
imports 1: `cargo/axum` (apps/causeway-server/src/browser_auth/session.rs:142)  

### 84. Function browser_session
`rhai:.mise/rhai/causeway/personas.rhai::browser_session`  .mise/rhai/causeway/personas.rhai:1008-1060  pkg:npm/causeway-monorepo  score 0.28  via `.mise/scripts/tasks/vsa/runtime-smoke.rhai` -> CALLS (.mise/scripts/tasks/vsa/runtime-smoke.rhai:56, 1.00)  
`fn browser_session(handle, base_url)`  
A browser session at the shell BASE_URL for the person HANDLE: #{ signed_in, handle, base, cookie, subject, groups }. (doc continues, 1432 bytes)  
callers 2: `.mise/scripts/tasks/vsa/runtime-smoke.rhai` (.mise/scripts/tasks/vsa/runtime-smoke.rhai:56), `.mise/scripts/tasks/vsa/live-preflight.rhai` (.mise/scripts/tasks/vsa/live-preflight.rhai:66)  
calls 33: `jar_after` (.mise/rhai/causeway/personas.rhai:1026), `with_login_hint` (.mise/rhai/causeway/personas.rhai:1028), `browser_get` (.mise/rhai/causeway/personas.rhai:1011) … 30 more  

### 85. Struct AuthenticationRequest
`rust:crates/causeway-access-control/src/lib.rs::AuthenticationRequest`  crates/causeway-access-control/src/lib.rs:72-77  pkg:cargo/causeway-access-control  score 0.28  via `apps/causeway-server/src/browser_auth/middleware.rs` -> IMPORTS (apps/causeway-server/src/browser_auth/middleware.rs:10, 1.00)  
`pub struct AuthenticationRequest`  
Captures credential and request metadata before authentication.  
members 7  
importers 23: `apps/causeway-server-web/src/access/tests.rs` (apps/causeway-server-web/src/access/tests.rs:3-7), `tests` (crates/connect-server-ext/src/lib.rs:371-374), `tests` (component/catalog/core/src/approval.rs:589-592) … 20 more  
used_by 26: `access_for_tests::Proves::authenticate` (apps/causeway-server/src/browser_auth/middleware.rs:92), `Anyone::authenticate` (apps/causeway-server-web/src/access/tests.rs:18), `tests::Proves::authenticate` (crates/flightdeck-auth/src/oauth.rs:144) … 23 more  
uses_types 4: `AuthorizationAuditHandle` (crates/causeway-access-control/src/lib.rs:76), `Justification` (crates/causeway-access-control/src/lib.rs:75), `BearerCredential` (crates/causeway-access-control/src/lib.rs:73) … 1 more  

### 86. Struct CorsTopology
`rust:torrentfx/crates/causeway-platform/src/config.rs::CorsTopology`  torrentfx/crates/causeway-platform/src/config.rs:2744-2748  pkg:cargo/causeway-platform  score 0.27  via `apps/causeway-server/src/routes/cors.rs` -> IMPORTS (apps/causeway-server/src/routes/cors.rs:20, 1.00)  
`pub struct CorsTopology`  
Which other origins' browsers may call this process (`[cors]`). (doc continues, 539 bytes)  
importers 2: `apps/causeway-server/src/routes/cors.rs` (apps/causeway-server/src/routes/cors.rs:20), `apps/causeway-server/src/routes/cors/tests.rs` (apps/causeway-server/src/routes/cors/tests.rs:8)  
used_by 6: `tests::outside_local_every_allowed_origin_is_https` (torrentfx/crates/causeway-platform/src/config.rs:4870), `TopologyDocument` (torrentfx/crates/causeway-platform/src/config.rs:1686), `CorsPolicy::from_topology` (apps/causeway-server/src/routes/cors.rs:73) … 3 more  
uses_types 1: `WebOrigin` (torrentfx/crates/causeway-platform/src/config.rs:2747)  

### 87. Function access_for_tests
`rust:apps/causeway-server/src/browser_auth/middleware.rs::access_for_tests`  apps/causeway-server/src/browser_auth/middleware.rs:86-104  pkg:cargo/causeway-server  score 0.27  via `apps/causeway-server/src/browser_auth/middleware.rs` -> CONTAINS (apps/causeway-server/src/browser_auth/middleware.rs:86-104, 1.00)  
`pub(crate) fn access_for_tests(identity: CallerIdentity) -> BrowserAccess`  
The access proof of `identity`, for a test that needs a signed-in caller (doc continues, 175 bytes)  
members 2  
callers 17: `every_twin_is_a_connect_method_cedar_decides` (apps/causeway-server/src/ws/router/governance/tests.rs:231), `signed_in_person` (apps/causeway-server/src/ws/tests.rs:267), `agent_for` (apps/causeway-server/src/routes/pages/test_shell.rs:241) … 14 more  
calls 8: `access_for_tests::Proves::authenticate` (apps/causeway-server/src/browser_auth/middleware.rs:101), `access_for_tests::Proves` (apps/causeway-server/src/browser_auth/middleware.rs:98), `AuthenticationPoint::new` (apps/causeway-server/src/browser_auth/middleware.rs:98) … 5 more  
importers 2: `tests` (apps/causeway-server/src/ws/session/subscription.rs:190), `apps/causeway-server/src/browser_auth/mod.rs` (apps/causeway-server/src/browser_auth/mod.rs:33)  
tests 8: `a_governed_dataset_loads_for_a_permitted_caller_only` (apps/causeway-server/src/ws/tests/explorer.rs:508), `tests::a_flag_is_evaluated_for_the_requests_verified_caller` (apps/causeway-server/src/composition/state/feature_flags.rs:143), `every_twin_is_a_connect_method_cedar_decides` (apps/causeway-server/src/ws/router/governance/tests.rs:231) … 5 more  
uses_types 2: `BrowserAccess` (apps/causeway-server/src/browser_auth/middleware.rs:86), `CallerIdentity` (apps/causeway-server/src/browser_auth/middleware.rs:86)  

### 88. Method BrowserSession::is_expired
`rust:apps/causeway-server/src/browser_auth/session.rs::BrowserSession::is_expired`  apps/causeway-server/src/browser_auth/session.rs:34-36  pkg:cargo/causeway-server  score 0.27  via `attach_browser_caller` -> CALLS (apps/causeway-server/src/browser_auth/middleware.rs:186, 0.75)  
`pub(crate) fn is_expired(&self, now: i64) -> bool`  
callers 2: `whoami` (apps/causeway-server/src/browser_auth/handlers.rs:349), `attach_browser_caller` (apps/causeway-server/src/browser_auth/middleware.rs:186)  

## Left out

33 included nodes have no source window for want of budget.  
- `rust:apps/causeway-server/src/browser_auth/handlers.rs::routes` Function 0.27 (over budget, 216 tokens)
- `rust:apps/causeway-server/src/browser_auth/handlers.rs::not_configured` Function 0.27 (over budget, 242 tokens)
- `rust:apps/causeway-server/src/browser_auth/mod.rs::handlers` Module 0.27 (over budget, 93 tokens)
- and 798 more

## Unresolved references in the seeds' files

- `headers_mut` at apps/causeway-server/src/browser_auth/handlers.rs:118 (no-declaration)
- `verifier` at apps/causeway-server/src/browser_auth/handlers.rs:157 (ambiguous)
- `url` at apps/causeway-server/src/browser_auth/handlers.rs:175 (ambiguous)
- `headers_mut` at apps/causeway-server/src/browser_auth/handlers.rs:202 (no-declaration)
- `post` at apps/causeway-server/src/browser_auth/handlers.rs:27 (ambiguous-receiver)
- `try_from` at apps/causeway-server/src/browser_auth/handlers.rs:317 (qualifier-mismatch)
- `now_utc` at apps/causeway-server/src/browser_auth/handlers.rs:346 (qualifier-mismatch)
- `Move` at apps/causeway-server/src/browser_auth/handlers.rs:402 (no-declaration)
- `Move` at apps/causeway-server/src/browser_auth/handlers.rs:414 (no-declaration)
- `trim_end_matches` at apps/causeway-server/src/browser_auth/handlers.rs:66 (no-declaration)

