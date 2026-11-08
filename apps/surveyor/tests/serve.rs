//! `surveyor serve` over the `mini` fixture: every JSON route answers what the CLI's
//! `--json` prints for the same command, every MCP tool over streamable HTTP answers
//! what the stdio server does, a bearer token gates the port, an `update` into the
//! dataset reaches the server without a restart, and a second repository without a
//! checkout serves everything but source.

// Rust guideline compliant 2026-02-21

use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use serde_json::{Value, json};

const TASK: &str = "The outbox relay drains pending rows one at a time; make Relay::drain take a batch size and send rows in batches.";
const DRAIN: &str = "rust:crates/outbox/src/relay.rs::Relay::drain";

fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/mini")
        .canonicalize()
        .expect("fixture exists")
}

/// A private copy of the fixture, so a test can change files and `update`.
fn copy_fixture(into: &Path) -> PathBuf {
    fn copy_dir(from: &Path, to: &Path) {
        std::fs::create_dir_all(to).expect("mkdir");
        for entry in std::fs::read_dir(from).expect("read dir") {
            let entry = entry.expect("entry");
            let target = to.join(entry.file_name());
            if entry.file_name() == ".surveyor" {
                continue;
            }
            if entry.file_type().expect("type").is_dir() {
                copy_dir(&entry.path(), &target);
            } else {
                std::fs::copy(entry.path(), &target).expect("copy");
            }
        }
    }
    let root = into.join("mini");
    copy_dir(&fixture(), &root);
    root.canonicalize().expect("copy exists")
}

/// A repository of one manifest that depends on `@mini/web`, without a checkout on
/// the server.
fn consumer(dir: &Path) -> PathBuf {
    let root = dir.join("consumer");
    std::fs::create_dir_all(root.join("src")).expect("mkdir");
    std::fs::write(
        root.join("package.json"),
        "{\n  \"name\": \"consumer\",\n  \"version\": \"0.1.0\",\n  \"dependencies\": {\n    \"@mini/web\": \"file:../mini/web\"\n  }\n}\n",
    )
    .expect("package.json");
    std::fs::write(
        root.join("src/index.ts"),
        "import { drain } from \"@mini/web\";\nexport const run = () => drain();\n",
    )
    .expect("index.ts");
    std::fs::write(
        root.join("surveyor.toml"),
        "[repository]\nid = \"consumer\"\n",
    )
    .expect("profile");
    root.canonicalize().expect("consumer exists")
}

fn surveyor(root: &Path, store: &Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_surveyor"))
        .arg("--repo-root")
        .arg(root)
        .arg("--store")
        .arg(store)
        .args(args)
        .output()
        .expect("runs")
}

/// The CLI's `--json` answer; `check` may exit 1 and still print its report.
fn cli_json(root: &Path, store: &Path, args: &[&str]) -> Value {
    let o = surveyor(root, store, &[&["--json"], args].concat());
    assert!(
        o.status.success() || args.first() == Some(&"check"),
        "{args:?}: {}",
        String::from_utf8_lossy(&o.stderr)
    );
    serde_json::from_slice(&o.stdout).unwrap_or_else(|e| {
        panic!(
            "json for {args:?}: {e}\n{}",
            String::from_utf8_lossy(&o.stdout)
        )
    })
}

/// A running `surveyor serve`, killed on drop.
struct Server {
    child: Child,
    base: String,
}

impl Server {
    /// Starts the server on an ephemeral port and reads its announcement.
    fn start(root: &Path, store: &Path, extra: &[&str]) -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_surveyor"))
            .arg("--repo-root")
            .arg(root)
            .arg("--store")
            .arg(store)
            .arg("--json")
            .args(["serve", "--listen", "127.0.0.1:0", "--reload-every", "1"])
            .args(extra)
            .stdout(Stdio::piped())
            // Not inherited: a server outliving a failed test would hold cargo's pipe open.
            .stderr(Stdio::null())
            .spawn()
            .expect("serve starts");
        let stdout = child.stdout.take().expect("stdout");
        // Held before reading, so a panic below still kills the server.
        let mut server = Self {
            child,
            base: String::new(),
        };
        let mut text = String::new();
        BufReader::new(stdout)
            .read_announcement(&mut text)
            .expect("announcement");
        let announced: Value =
            serde_json::from_str(&text).unwrap_or_else(|e| panic!("announcement {e}: {text}"));
        let listen = announced["listen"].as_str().expect("listen").to_owned();
        server.base = format!("http://{listen}");
        server
    }

    fn url(&self, path: &str) -> String {
        format!("{}{path}", self.base)
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// Reads the pretty-printed JSON object the server prints first: up to the unindented
/// brace that closes it.
trait ReadAnnouncement {
    fn read_announcement(&mut self, out: &mut String) -> std::io::Result<()>;
}

impl<R: BufRead> ReadAnnouncement for R {
    fn read_announcement(&mut self, out: &mut String) -> std::io::Result<()> {
        let mut line = String::new();
        loop {
            line.clear();
            if self.read_line(&mut line)? == 0 {
                return Ok(());
            }
            out.push_str(&line);
            if line.trim_end() == "}" {
                return Ok(());
            }
        }
    }
}

async fn get(client: &reqwest::Client, url: &str) -> (u16, Value) {
    get_with(client, url, None).await
}

async fn get_with(client: &reqwest::Client, url: &str, token: Option<&str>) -> (u16, Value) {
    let mut request = client.get(url);
    if let Some(token) = token {
        request = request.bearer_auth(token);
    }
    let response = request.send().await.expect("response");
    let status = response.status().as_u16();
    let text = response.text().await.expect("body");
    let value = serde_json::from_str(&text).unwrap_or_else(|e| panic!("{url}: {e}\n{text}"));
    (status, value)
}

/// A JSON-RPC client to an MCP server, over streamable HTTP or stdio.
enum Mcp {
    Http {
        client: reqwest::Client,
        url: String,
        token: Option<String>,
        session: Option<String>,
        protocol: String,
    },
    Stdio {
        child: Child,
        reader: BufReader<std::process::ChildStdout>,
    },
}

impl Mcp {
    fn http(url: &str, token: Option<&str>) -> Self {
        Self::Http {
            client: reqwest::Client::new(),
            url: url.to_owned(),
            token: token.map(str::to_owned),
            session: None,
            protocol: "2025-03-26".to_owned(),
        }
    }

    fn stdio(root: &Path, store: &Path) -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_surveyor"))
            .arg("--repo-root")
            .arg(root)
            .arg("--store")
            .arg(store)
            .arg("mcp")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .expect("mcp starts");
        let reader = BufReader::new(child.stdout.take().expect("stdout"));
        Self::Stdio { child, reader }
    }

    /// Sends `initialize` and `notifications/initialized`.
    async fn handshake(&mut self) -> Value {
        let result = self
            .request(
                1,
                "initialize",
                json!({
                    "protocolVersion": "2025-03-26",
                    "capabilities": {},
                    "clientInfo": { "name": "serve-test", "version": "0" }
                }),
            )
            .await;
        if let Self::Http { protocol, .. } = self
            && let Some(v) = result["protocolVersion"].as_str()
        {
            v.clone_into(protocol);
        }
        self.notify("notifications/initialized").await;
        result
    }

    async fn notify(&mut self, method: &str) {
        let message = json!({ "jsonrpc": "2.0", "method": method });
        match self {
            Self::Http { .. } => {
                let _ = self.post(&message).await;
            }
            Self::Stdio { child, .. } => {
                let stdin = child.stdin.as_mut().expect("stdin");
                writeln!(stdin, "{message}").expect("write");
            }
        }
    }

    /// Sends a request and returns its `result`, panicking on an error response.
    async fn request(&mut self, id: u64, method: &str, params: Value) -> Value {
        let response = self.exchange(id, method, params).await;
        assert!(
            response.get("error").is_none(),
            "{method} failed: {}",
            response["error"]
        );
        response["result"].clone()
    }

    /// Sends a request and returns the whole response, error or result.
    async fn exchange(&mut self, id: u64, method: &str, params: Value) -> Value {
        let message = json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params });
        match self {
            Self::Http { .. } => {
                let (_, messages) = self.post(&message).await;
                messages
                    .into_iter()
                    .find(|m| m["id"] == id)
                    .expect("a response with the request's id")
            }
            Self::Stdio { child, reader } => {
                let stdin = child.stdin.as_mut().expect("stdin");
                writeln!(stdin, "{message}").expect("write");
                let mut line = String::new();
                loop {
                    line.clear();
                    assert!(
                        reader.read_line(&mut line).expect("read") > 0,
                        "server closed"
                    );
                    let value: Value = serde_json::from_str(line.trim()).expect("json line");
                    if value["id"] == id {
                        return value;
                    }
                }
            }
        }
    }

    /// Posts one message; returns the status and every JSON-RPC message in the reply,
    /// whether it came as JSON or as a server-sent event stream.
    async fn post(&mut self, message: &Value) -> (u16, Vec<Value>) {
        let Self::Http {
            client,
            url,
            token,
            session,
            protocol,
        } = self
        else {
            unreachable!("post is the HTTP transport's")
        };
        let mut request = client
            .post(url.as_str())
            .header("Content-Type", "application/json")
            .header("Accept", "application/json, text/event-stream")
            .header("MCP-Protocol-Version", protocol.as_str())
            .json(message);
        if let Some(token) = token {
            request = request.bearer_auth(token);
        }
        if let Some(session) = session {
            request = request.header("Mcp-Session-Id", session.as_str());
        }
        let response = request.send().await.expect("response");
        let status = response.status().as_u16();
        if let Some(id) = response
            .headers()
            .get("mcp-session-id")
            .and_then(|v| v.to_str().ok())
        {
            *session = Some(id.to_owned());
        }
        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or_default()
            .to_owned();
        let text = response.text().await.expect("body");
        let messages = if content_type.starts_with("text/event-stream") {
            text.lines()
                .filter_map(|l| l.strip_prefix("data:"))
                .filter_map(|d| serde_json::from_str::<Value>(d.trim()).ok())
                .collect()
        } else {
            serde_json::from_str::<Value>(&text)
                .ok()
                .into_iter()
                .collect()
        };
        (status, messages)
    }

    async fn call(&mut self, id: u64, tool: &str, arguments: Value) -> Value {
        self.request(
            id,
            "tools/call",
            json!({ "name": tool, "arguments": arguments }),
        )
        .await
    }
}

impl Drop for Mcp {
    fn drop(&mut self) {
        if let Self::Stdio { child, .. } = self {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

/// The JSON a tool result carries in its first text block.
fn text_json(result: &Value) -> Value {
    let text = result["content"][0]["text"]
        .as_str()
        .unwrap_or_else(|| panic!("text content: {result}"));
    serde_json::from_str(text).unwrap_or_else(|_| Value::String(text.to_owned()))
}

#[tokio::test]
async fn every_json_route_answers_what_the_cli_prints() {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = copy_fixture(dir.path());
    let store = dir.path().join("ds");
    let o = surveyor(&root, &store, &["index"]);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let server = Server::start(&root, &store, &[]);
    let client = reqwest::Client::new();

    let (status, repos) = get(&client, &server.url("/v1/repos")).await;
    assert_eq!(status, 200, "{repos}");
    assert_eq!(repos["repositories"][0]["repo"], "mini", "{repos}");
    assert_eq!(repos["repositories"][0]["mcp"], "/mcp/mini");
    assert_eq!(
        repos["repositories"][0]["root"],
        root.display().to_string(),
        "{repos}"
    );
    assert_eq!(repos["repositories"][0]["rules"], 1, "{repos}");

    let same: Vec<(&str, Vec<&str>)> = vec![
        ("/v1/mini/stats", vec!["stats"]),
        (
            "/v1/mini/explain?q=Relay::drain",
            vec!["explain", "Relay::drain"],
        ),
        (
            "/v1/mini/card?q=Relay::drain,cw_outbox.outbox",
            vec!["card", "Relay::drain", "cw_outbox.outbox"],
        ),
        (
            "/v1/mini/affected?q=cw_outbox.outbox&depth=4",
            vec!["affected", "cw_outbox.outbox", "--depth", "4"],
        ),
        (
            "/v1/mini/path?from=crates/api/src/main.rs::main&to=cw_outbox.outbox&mode=directed",
            vec![
                "path",
                "crates/api/src/main.rs::main",
                "cw_outbox.outbox",
                "--directed",
            ],
        ),
        (
            "/v1/mini/query?q=outbox&limit=20",
            vec!["query", "outbox", "--limit", "20"],
        ),
        (
            "/v1/mini/search?q=who+drains+the+outbox&limit=10",
            vec!["search", "who drains the outbox", "--limit", "10"],
        ),
        (
            "/v1/mini/source?target=rust:crates/outbox/src/relay.rs::Relay::drain&context=2",
            vec!["source", DRAIN, "--context", "2"],
        ),
        ("/v1/mini/check", vec!["check"]),
    ];
    for (route, args) in same {
        let (status, served) = get(&client, &server.url(route)).await;
        assert_eq!(status, 200, "{route}: {served}");
        let printed = cli_json(&root, &store, &args);
        assert_eq!(served, printed, "{route} differs from surveyor {args:?}");
    }

    // The context pack: same task, same budget, same source and rules.
    let task = TASK.replace(' ', "+").replace(';', "%3B");
    let (status, served) = get(
        &client,
        &server.url(&format!("/v1/mini/context?task={task}&budget=5000")),
    )
    .await;
    assert_eq!(status, 200, "{served}");
    let printed = cli_json(&root, &store, &["context", TASK, "--budget", "5000"]);
    assert_eq!(served, printed, "context differs");

    // Refusals say why.
    let (status, body) = get(&client, &server.url("/v1/nothing/stats")).await;
    assert_eq!(status, 404, "{body}");
    assert!(body["error"].as_str().unwrap().contains("mini"), "{body}");
    let (status, body) = get(
        &client,
        &server.url("/v1/mini/explain?q=no_such_symbol_here"),
    )
    .await;
    assert_eq!(status, 400, "{body}");
    assert!(
        body["error"].as_str().unwrap().contains("no node matches"),
        "{body}"
    );
    let (status, body) = get(&client, &server.url("/v1/mini/affected?q=drain&kinds=NOPE")).await;
    assert_eq!(status, 400, "{body}");
}

#[tokio::test]
async fn mcp_over_http_answers_what_stdio_answers() {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = copy_fixture(dir.path());
    let store = dir.path().join("ds");
    let o = surveyor(&root, &store, &["index"]);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let server = Server::start(&root, &store, &[]);

    let mut http = Mcp::http(&server.url("/mcp/mini"), None);
    let mut stdio = Mcp::stdio(&root, &store);
    let over_http = http.handshake().await;
    let over_stdio = stdio.handshake().await;
    assert_eq!(over_http["serverInfo"], over_stdio["serverInfo"]);
    assert_eq!(over_http["instructions"], over_stdio["instructions"]);

    let tools_http = http.request(2, "tools/list", json!({})).await;
    let tools_stdio = stdio.request(2, "tools/list", json!({})).await;
    assert_eq!(tools_http["tools"], tools_stdio["tools"]);
    let names: Vec<&str> = tools_http["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["name"].as_str().unwrap())
        .collect();
    assert_eq!(names.len(), 12, "{names:?}");

    let calls: Vec<(&str, Value)> = vec![
        (
            "query_graph",
            json!({ "query": "who calls drain", "limit": 3 }),
        ),
        ("get_node", json!({ "node_id": "Relay::drain" })),
        ("get_neighbors", json!({ "node_id": "Relay::drain" })),
        (
            "shortest_path",
            json!({ "from": "crates/api/src/main.rs::main", "to": "cw_outbox.outbox", "directed": true }),
        ),
        ("god_nodes", json!({ "limit": 5 })),
        ("get_community", json!({ "node_id": "Relay::drain" })),
        ("read_project_source", json!({ "path": DRAIN })),
        (
            "affected",
            json!({ "node_id": "cw_outbox.outbox", "depth": 4 }),
        ),
        ("find_nodes", json!({ "query": "outbox" })),
        ("check_invariants", json!({})),
        ("graph_stats", json!({})),
        (
            "context_pack",
            json!({ "task": TASK, "budget": 5000, "format": "json" }),
        ),
    ];
    for (i, (tool, arguments)) in calls.into_iter().enumerate() {
        let id = 10 + i as u64;
        let a = http.call(id, tool, arguments.clone()).await;
        let b = stdio.call(id, tool, arguments).await;
        assert_eq!(text_json(&a), text_json(&b), "{tool} differs over HTTP");
    }
    // The MCP tool and the JSON route render the same graph.
    let affected = http
        .call(
            40,
            "affected",
            json!({ "node_id": "cw_outbox.outbox", "depth": 4 }),
        )
        .await;
    let (_, route) = get(
        &reqwest::Client::new(),
        &server.url("/v1/mini/affected?q=cw_outbox.outbox&depth=4"),
    )
    .await;
    assert_eq!(
        text_json(&affected)["count"],
        route["affected"].as_array().unwrap().len()
    );
}

#[tokio::test]
async fn a_bearer_token_gates_every_route() {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = copy_fixture(dir.path());
    let store = dir.path().join("ds");
    let o = surveyor(&root, &store, &["index"]);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let server = Server::start(&root, &store, &["--token", "s3cret"]);
    let client = reqwest::Client::new();

    let (status, body) = get(&client, &server.url("/v1/repos")).await;
    assert_eq!(status, 401, "{body}");
    let (status, _) = get_with(&client, &server.url("/v1/repos"), Some("wrong")).await;
    assert_eq!(status, 401);
    let (status, body) = get_with(&client, &server.url("/v1/repos"), Some("s3cret")).await;
    assert_eq!(status, 200, "{body}");

    let mut without = Mcp::http(&server.url("/mcp/mini"), None);
    let init = json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {
        "protocolVersion": "2025-03-26", "capabilities": {}, "clientInfo": { "name": "t", "version": "0" } } });
    let (status, _) = without.post(&init).await;
    assert_eq!(status, 401);
    let mut with = Mcp::http(&server.url("/mcp/mini"), Some("s3cret"));
    let result = with.handshake().await;
    assert!(result["serverInfo"].is_object(), "{result}");
}

#[tokio::test]
async fn an_update_to_the_dataset_is_served_without_a_restart() {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = copy_fixture(dir.path());
    let store = dir.path().join("ds");
    let o = surveyor(&root, &store, &["index"]);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let server = Server::start(&root, &store, &[]);
    let client = reqwest::Client::new();

    let (_, before) = get(&client, &server.url("/v1/repos")).await;
    let generation = before["generation"].as_u64().unwrap();
    let (_, found) = get(&client, &server.url("/v1/mini/query?q=freshly_added_fn")).await;
    assert_eq!(found.as_array().unwrap().len(), 0, "{found}");

    // A new function lands and `update` writes it into the shared dataset.
    let relay = root.join("crates/outbox/src/relay.rs");
    let mut text = std::fs::read_to_string(&relay).expect("relay.rs");
    text.push_str("\n/// Added while the server runs.\npub fn freshly_added_fn() {}\n");
    std::fs::write(&relay, text).expect("write relay.rs");
    let o = surveyor(&root, &store, &["update"]);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));

    let deadline = Instant::now() + Duration::from_secs(20);
    loop {
        let (_, found) = get(&client, &server.url("/v1/mini/query?q=freshly_added_fn")).await;
        if found.as_array().is_some_and(|a| a.len() == 1) {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "the server did not reload within 20 s: {found}"
        );
        tokio::time::sleep(Duration::from_millis(250)).await;
    }
    let (_, after) = get(&client, &server.url("/v1/repos")).await;
    assert!(
        after["generation"].as_u64().unwrap() > generation,
        "{after}"
    );
    assert_eq!(
        after["repositories"][0]["loaded_generation"], after["generation"],
        "{after}"
    );
    // A session opened before the reload sees the new graph too.
    let mut http = Mcp::http(&server.url("/mcp/mini"), None);
    http.handshake().await;
    let found = http
        .call(3, "find_nodes", json!({ "query": "freshly_added_fn" }))
        .await;
    assert_eq!(text_json(&found).as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn a_repository_without_a_checkout_serves_everything_but_source() {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = copy_fixture(dir.path());
    let other = consumer(dir.path());
    let store = dir.path().join("ds");
    for repo in [&root, &other] {
        let o = surveyor(repo, &store, &["index"]);
        assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    }
    // Two repositories: --repo-root is nobody's checkout, --root names mini's.
    let server = Server::start(
        &root,
        &store,
        &["--root", &format!("mini={}", root.display())],
    );
    let client = reqwest::Client::new();

    let (_, repos) = get(&client, &server.url("/v1/repos")).await;
    let listed = repos["repositories"].as_array().unwrap();
    assert_eq!(listed.len(), 2, "{repos}");
    let consumer = listed.iter().find(|r| r["repo"] == "consumer").unwrap();
    assert!(consumer.get("root").is_none(), "{consumer}");
    assert_eq!(consumer["mcp"], "/mcp/consumer");

    let (status, found) = get(&client, &server.url("/v1/consumer/query?q=run")).await;
    assert_eq!(status, 200, "{found}");
    assert_eq!(found[0]["repo"], "consumer", "{found}");
    let (status, body) = get(
        &client,
        &server.url("/v1/consumer/source?target=file:src/index.ts:1-2"),
    )
    .await;
    assert_eq!(status, 409, "{body}");
    assert!(
        body["error"].as_str().unwrap().contains("--root consumer="),
        "{body}"
    );
    let (status, body) = get(&client, &server.url("/v1/consumer/check")).await;
    assert_eq!(status, 404, "{body}");
    // mini still reads its source, and the consumer's federated edge is in its graph.
    let (status, _) = get(&client, &server.url("/v1/mini/source?target=Relay::drain")).await;
    assert_eq!(status, 200);
    let (_, web) = get(&client, &server.url("/v1/mini/explain?q=pkg:npm/@mini/web")).await;
    assert!(
        web["incoming"]
            .as_array()
            .unwrap()
            .iter()
            .any(|r| r["node"]["repo"] == "consumer"),
        "{web}"
    );

    let mut mcp = Mcp::http(&server.url("/mcp/consumer"), None);
    mcp.handshake().await;
    let response = mcp
        .exchange(
            5,
            "tools/call",
            json!({ "name": "read_project_source", "arguments": { "path": "src/index.ts" } }),
        )
        .await;
    let refused = response["error"]["message"]
        .as_str()
        .or_else(|| response["result"]["content"][0]["text"].as_str())
        .unwrap_or_default();
    assert!(refused.contains("no checkout"), "{response}");
}
