use qwen_audio::{
    ChildMessage, ErrorCode, GuiLauncher, GuiProcess, IoFuture, LaunchedGui, SafeError,
    SendDisposition,
};
use serde_json::Value;
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc, Mutex,
};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, DuplexStream};

#[derive(Default)]
pub struct Life {
    pub exited: AtomicBool,
    pub kills: AtomicUsize,
    pub waits: AtomicUsize,
    notify: tokio::sync::Notify,
}
impl Life {
    fn exit(&self) {
        self.exited.store(true, Ordering::SeqCst);
        self.notify.notify_waiters();
    }
}
struct Process(Arc<Life>);
impl GuiProcess for Process {
    fn wait<'a>(&'a mut self) -> IoFuture<'a, Option<i32>> {
        Box::pin(async move {
            self.0.waits.fetch_add(1, Ordering::SeqCst);
            loop {
                let notified = self.0.notify.notified();
                if self.0.exited.load(Ordering::SeqCst) {
                    return Ok(Some(0));
                }
                notified.await;
            }
        })
    }
    fn kill(&mut self) -> Result<(), SafeError> {
        self.0.kills.fetch_add(1, Ordering::SeqCst);
        self.0.exit();
        Ok(())
    }
}

#[derive(Clone)]
pub struct ChildControl {
    sender: tokio::sync::mpsc::UnboundedSender<Vec<u8>>,
    pub parents: Arc<Mutex<Vec<Value>>>,
    pub life: Arc<Life>,
}
impl ChildControl {
    /// Report OS-process termination while the fixture keeps stdout open.
    /// This distinguishes process monitoring from merely noticing pipe EOF.
    pub fn exit_process_keep_stdout(&self) {
        self.life.exit();
    }
    pub fn emit(&self, event: ChildMessage) {
        self.emit_json(serde_json::to_value(event).unwrap());
    }
    pub fn emit_json(&self, value: Value) {
        let mut bytes = serde_json::to_vec(&value).unwrap();
        bytes.push(b'\n');
        self.sender.send(bytes).unwrap();
    }
    pub fn session(&self) -> qwen_audio::SessionId {
        let parents = self.parents.lock().unwrap();
        let boot = parents.iter().find(|v| v["event"] == "boot").unwrap();
        qwen_audio::SessionId::parse(boot["session_id"].as_str().unwrap()).unwrap()
    }
    pub fn count_event(&self, event: &str) -> usize {
        self.parents
            .lock()
            .unwrap()
            .iter()
            .filter(|v| v["event"] == event)
            .count()
    }
}

#[derive(Default)]
pub struct Launcher {
    pub children: Mutex<Vec<ChildControl>>,
    pub fail_next: AtomicBool,
    pub stubborn: AtomicBool,
    pub no_ready: AtomicBool,
}
impl GuiLauncher for Launcher {
    fn launch<'a>(&'a self) -> IoFuture<'a, LaunchedGui> {
        Box::pin(async move {
            if self.fail_next.swap(false, Ordering::SeqCst) {
                return Err(SafeError::new(
                    ErrorCode::GuiLaunch,
                    SendDisposition::NotSent,
                ));
            }
            assert!(
                self.children
                    .lock()
                    .unwrap()
                    .iter()
                    .all(|child| child.life.exited.load(Ordering::SeqCst)),
                "old GUI must be reaped before another launch"
            );
            let (parent_input, child_input) = tokio::io::duplex(1024 * 1024);
            let (mut child_output, parent_output) = tokio::io::duplex(1024 * 1024);
            let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel::<Vec<u8>>();
            let parents = Arc::new(Mutex::new(Vec::new()));
            let life = Arc::new(Life::default());
            let control = ChildControl {
                sender,
                parents: parents.clone(),
                life: life.clone(),
            };
            self.children.lock().unwrap().push(control);
            let stubborn = self.stubborn.load(Ordering::SeqCst);
            let no_ready = self.no_ready.load(Ordering::SeqCst);
            let child_life = life.clone();
            tokio::spawn(async move {
                let mut lines = BufReader::new(child_input).lines();
                loop {
                    tokio::select! {
                        line = lines.next_line() => {
                            let Ok(Some(line)) = line else { break };
                            let value: Value = serde_json::from_str(&line).unwrap();
                            parents.lock().unwrap().push(value.clone());
                            if value["event"] == "boot" && !no_ready {
                                let ready = serde_json::json!({"event":"ready", "session_id":value["session_id"]});
                                let bytes = format!("{ready}\n");
                                if child_output.write_all(bytes.as_bytes()).await.is_err() { break; }
                            }
                            if value["event"] == "close" && !stubborn { break; }
                        }
                        bytes = receiver.recv() => {
                            let Some(bytes) = bytes else { break };
                            if child_output.write_all(&bytes).await.is_err() { break; }
                        }
                    }
                }
                child_life.exit();
            });
            Ok(LaunchedGui {
                input: Box::new(parent_input),
                output: Box::new(parent_output),
                process: Box::new(Process(life)),
            })
        })
    }
}

pub struct Harness {
    pub launcher: Arc<Launcher>,
    pub writer: DuplexStream,
    reader: BufReader<DuplexStream>,
    server: Option<tokio::task::JoinHandle<Result<(), SafeError>>>,
    next_id: u64,
}
impl Harness {
    pub async fn new(launcher: Arc<Launcher>) -> Self {
        let (writer, server_reader) = tokio::io::duplex(1024 * 1024);
        let (server_writer, reader) = tokio::io::duplex(1024 * 1024);
        let server = tokio::spawn(qwen_audio::serve_mcp(
            server_reader,
            server_writer,
            launcher.clone(),
        ));
        let mut harness = Self {
            launcher,
            writer,
            reader: BufReader::new(reader),
            server: Some(server),
            next_id: 0,
        };
        let response = harness.rpc("initialize", serde_json::json!({"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"synthetic-test","version":"1"}})).await;
        assert_eq!(response["result"]["protocolVersion"], "2025-06-18");
        harness
            .raw(b"{\"jsonrpc\":\"2.0\",\"method\":\"notifications/initialized\"}\n")
            .await;
        harness
    }
    pub async fn raw(&mut self, bytes: &[u8]) {
        self.writer.write_all(bytes).await.unwrap();
    }
    pub async fn receive(&mut self) -> Value {
        let mut line = String::new();
        let n = tokio::time::timeout(
            std::time::Duration::from_secs(10),
            self.reader.read_line(&mut line),
        )
        .await
        .expect("bounded protocol response")
        .unwrap();
        assert!(n > 0, "unexpected MCP EOF");
        serde_json::from_str(&line).expect("stdout contains only JSON-RPC")
    }
    pub async fn rpc(&mut self, method: &str, params: Value) -> Value {
        self.next_id += 1;
        let id = self.next_id;
        let message = serde_json::json!({"jsonrpc":"2.0","id":id,"method":method,"params":params});
        self.raw(format!("{message}\n").as_bytes()).await;
        let response = self.receive().await;
        assert_eq!(response["id"], id);
        response
    }
    pub async fn tool(&mut self, name: &str, arguments: Value) -> Value {
        self.rpc(
            "tools/call",
            serde_json::json!({"name":name,"arguments":arguments}),
        )
        .await
    }
    pub fn child(&self, index: usize) -> ChildControl {
        self.launcher.children.lock().unwrap()[index].clone()
    }
    pub async fn finish(mut self) {
        self.writer.shutdown().await.unwrap();
        tokio::time::timeout(
            std::time::Duration::from_secs(10),
            self.server.take().unwrap(),
        )
        .await
        .expect("server must reap children")
        .unwrap()
        .unwrap();
    }
}
impl Drop for Harness {
    fn drop(&mut self) {
        if let Some(server) = self.server.take() {
            server.abort();
        }
    }
}

pub fn content(response: &Value) -> Value {
    let result = &response["result"];
    let text = result["content"][0]["text"]
        .as_str()
        .expect("tool text JSON");
    let decoded: Value = serde_json::from_str(text).expect("tool text must parse");
    if result["isError"] != true {
        assert!(
            result.get("structuredContent").is_some(),
            "success requires structuredContent"
        );
    }
    if let Some(structured) = result.get("structuredContent") {
        assert_eq!(structured, &decoded, "text and structured contents agree");
    }
    decoded
}

pub fn assert_tool_error(response: &Value, code: &str) {
    assert_eq!(response["result"]["isError"], true);
    let value = content(response);
    let actual = value
        .get("code")
        .or_else(|| value.get("error").and_then(|e| e.get("code")));
    assert_eq!(actual.and_then(Value::as_str), Some(code));
}
