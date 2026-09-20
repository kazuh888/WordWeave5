//! Offline test fixture. It never starts Codex or connects to a service.
use fs2::FileExt;
use serde_json::{json, Value};
use std::{
    fs::{self, OpenOptions},
    io::{self, BufRead, Write},
};

fn send(value: Value) {
    let mut bytes = serde_json::to_vec(&value).unwrap();
    bytes.push(b'\n');
    let mut out = io::stdout().lock();
    // Separate writes exercise JSON-line assembly across pipe reads.
    out.write_all(&bytes[..7]).unwrap();
    out.flush().unwrap();
    std::thread::sleep(std::time::Duration::from_millis(2));
    out.write_all(&bytes[7..]).unwrap();
    out.flush().unwrap();
}

fn main() {
    // For the copied Windows wrapper fixture only. This records no user data.
    let probe = std::env::current_exe().ok().filter(|p| p.file_name().is_some_and(|n| n == "mock.exe" || n.eq_ignore_ascii_case("volta.exe")))
        .and_then(|p| p.parent().map(|dir| dir.join("mock-stage.txt")));
    let mark = |stage: &str| {
        if let Some(path) = &probe { fs::write(path, stage).unwrap(); }
    };
    mark("mock.exe開始");
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let is_volta_probe = std::env::current_exe().ok().is_some_and(|p| p.file_name().is_some_and(|n| n.eq_ignore_ascii_case("volta.exe")));
    if is_volta_probe {
        assert_eq!(args, ["run", "codex", "app-server"]);
    } else {
        assert_eq!(args, ["app-server"]);
    }
    mark("app-server引数確認済み");
    if let Some(path) = &probe {
        // PATH-discovery fixtures keep the executable and working directory
        // separate. The expected directory is test-only data beside the mock.
        let expected_dir = fs::read_to_string(path.parent().unwrap().join("expected-cwd.txt"))
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|_| path.parent().unwrap().to_path_buf());
        let expected = fs::canonicalize(expected_dir).unwrap();
        let actual = fs::canonicalize(std::env::current_dir().unwrap()).unwrap();
        if actual != expected {
            mark("作業ディレクトリ不一致");
            panic!("Wrapper did not preserve the working directory");
        }
    }
    mark("作業ディレクトリ確認済み");
    let mode = fs::read_to_string("mode.txt").expect("Run through cargo test; missing fixture mode").trim().to_owned();
    mark("mode.txt読み取り済み");
    if mode == "early_exit" || mode == "bad_json" {
        // Wait for initialize, so this tests receive/EOF rather than a spawn race.
        let mut initial = String::new();
        io::stdin().read_line(&mut initial).unwrap();
    }
    if mode == "early_exit" {
        eprintln!("volta: command not found https://example.test/SECRET access_token=SECRET");
        std::process::exit(7);
    }
    if mode == "bad_json" {
        println!("not JSON");
        return;
    }
    let process_lock = OpenOptions::new()
        .read(true).write(true).create(true).truncate(false)
        .open("process.lock").unwrap();
    process_lock.lock_exclusive().unwrap();
    let mut requests = OpenOptions::new().create(true).append(true)
        .open("requests.log").unwrap();
    for line in io::stdin().lock().lines() {
        let v: Value = serde_json::from_str(&line.unwrap()).unwrap();
        let method = v["method"].as_str().unwrap();
        mark("JSON要求受信済み");
        writeln!(requests, "{method}").unwrap();
        requests.flush().unwrap();
        match method {
            "initialize" => send(json!({"id":v["id"],"result":{"userAgent":"mock"}})),
            "initialized" => {},
            "account/read" if mode == "noauth" => send(json!({"id":v["id"],"result":{"account":null,"requiresOpenaiAuth":true}})),
            "account/read" => send(json!({"id":v["id"],"result":{"account":{
                "type": if mode == "paid" { "apiKey" } else { "chatgpt" },
                "planType":"test"
            }}})),
            "thread/start" => {
                fs::write("thread-start.json", serde_json::to_vec(&v["params"]).unwrap()).unwrap();
                assert_eq!(v["params"]["ephemeral"], false);
                assert_eq!(v["params"]["sandbox"], "read-only");
                assert_eq!(v["params"]["modelProvider"], "openai");
                let mut result = json!({
                    "thread":{"id":"t"},"modelProvider":"openai","model":"test"
                });
                if mode == "reported_settings" {
                    assert_eq!(v["params"]["model"], "requested-model");
                    result["model"] = json!("returned-model");
                    result["reasoningEffort"] = json!("high");
                }
                if mode == "null_effort" { result["reasoningEffort"] = Value::Null; }
                if mode == "effort" { result["reasoningEffort"] = json!("low"); }
                if mode == "effort_rerouted" { result["model"] = json!("other"); }
                send(json!({"id":v["id"],"result":result}));
            },
            "thread/read" => {
                assert_eq!(v["params"]["threadId"], "t");
                assert_eq!(v["params"]["includeTurns"], true);
                if mode=="large_lost_result" {
                    let input:Value=serde_json::from_slice(&fs::read("media-input.json").unwrap()).unwrap();
                    send(json!({"id":v["id"],"result":{"thread":{"id":"t","turns":[{"id":"u","status":"completed","items":[{"id":"user","type":"userMessage","content":input},{"id":"answer","type":"agentMessage","text":"large recovered"}]}]}}}));
                    continue;
                }
                send(json!({"id":v["id"],"result":{"thread":{"id":"t","turns":[{
                    "id": if mode=="wrong_turn" {"other"} else {"u"}, "status":"completed",
                    "items":[{"id":"a","type":"agentMessage","phase":"final_answer","text":"recovered"}]
                }]}}}));
            },
            "model/list" if mode.starts_with("effort") => {
                if mode == "effort_unavailable" {
                    send(json!({"id":v["id"],"error":{"code":-1,"message":"unavailable"}}));
                } else if mode == "effort_cycle" {
                    send(json!({"id":v["id"],"result":{"data":[],"nextCursor":"repeat"}}));
                } else if mode == "effort_missing" {
                    send(json!({"id":v["id"],"result":{"data":[{"model":"test"}],"nextCursor":null}}));
                } else if v["params"]["cursor"].is_null() {
                    send(json!({"id":v["id"],"result":{"data":[{
                        "model":"other","displayName":"Other model","isDefault":true,
                        "supportedReasoningEfforts":[{"reasoningEffort":"low","description":"less"}],
                        "defaultReasoningEffort":"low"
                    }],"nextCursor":"page-two"}}));
                } else {
                    assert_eq!(v["params"]["cursor"], "page-two");
                    send(json!({"id":v["id"],"result":{"data":[{
                        "model":"test","displayName":"Test model",
                        "supportedReasoningEfforts":[{"reasoningEffort":"high","description":"more"},{"reasoningEffort":"future-effort","description":"dynamic"}],
                        "defaultReasoningEffort":"high"
                    },{"model":"no-capabilities"}],"nextCursor":null}}));
                }
            },
            "model/list" => send(json!({"id":v["id"],"result":{
                "data":[{"model":"test","inputModalities":
                    if mode == "audio" { vec!["text", "audio"] } else if mode == "image" || mode=="large_lost_result" { vec!["text", "image"] } else { vec!["text"] }}],
                "nextCursor":null
            }})),
            "turn/start" => {
                if mode=="lost_id" { return; }
                if mode=="lost_result" || mode=="large_lost_result" {
                    if mode=="large_lost_result" {fs::write("media-input.json",serde_json::to_vec(&v["params"]["input"]).unwrap()).unwrap();}
                    send(json!({"id":v["id"],"result":{"turn":{"id":"u"}}}));
                    return;
                }
                if mode == "chat" {
                    let expected: Value = serde_json::from_str(&fs::read_to_string("expected-input.json").unwrap()).unwrap();
                    let actual: Value = serde_json::from_str(v["params"]["input"][0]["text"].as_str().unwrap()).unwrap();
                    assert_eq!(actual, expected, "Chat context was not transmitted intact");
                }
                if mode == "cancel" {
                    fs::write("waiting", b"ready").unwrap();
                    // Keep the process alive even if stdin closes. The test must kill it.
                    loop { std::thread::park(); }
                }
                let response = if mode == "structured_chat" {
                    assert_eq!(v["params"]["outputSchema"], wordweave5::chat_action::schema());
                    json!({"answer":"apologize for を新規登録する案を確認してください。",
                        "title":"apologize for の登録", "action":{"operation":"new",
                        "base":"apologize for", "entry_id":null}}).to_string()
                } else { "{\"answer\":\"ok\"}".into() };
                send(json!({"method":"item/completed","params":{
                    "threadId":"t","turnId":"u","item":{
                        "id":"a","type":"agentMessage","phase":"final_answer",
                        "text":response
                    }
                }}));
                send(json!({"method":"turn/completed","params":{
                    "threadId":"t","turn":{"id":"u",
                        "status":if mode == "failed" { "failed" } else if mode=="interrupted" { "interrupted" } else { "completed" },
                        "error":null
                    }
                }}));
                // Deliberately deliver the start response after completion events.
                send(json!({"id":v["id"],"result":{"turn":{"id":"u"}}}));
            },
            other => panic!("Unexpected method: {other}"),
        }
    }
}
