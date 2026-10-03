use crate::error::{err, sent_err};
use crate::ipc::{read_line, write_message};
use crate::*;
use serde_json::{json, Value};
use std::{sync::Arc, time::Duration};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    sync::mpsc,
};
const IPC_LIMIT: usize = 512 * 1024;
struct Child {
    input: Box<dyn AsyncWrite + Unpin + Send>,
    process: Box<dyn GuiProcess>,
    reader: tokio::task::JoinHandle<()>,
    id: SessionId,
    reference: ReferenceText,
    deadline: Option<tokio::time::Instant>,
}
enum Input {
    Line(Vec<u8>),
    Closed,
    Invalid,
}
async fn close_child(child: &mut Option<Child>) {
    if let Some(mut c) = child.take() {
        let _ = write_message(&mut *c.input, &ParentMessage::Close, IPC_LIMIT).await;
        c.reader.abort();
        if !matches!(
            tokio::time::timeout(Duration::from_secs(2), c.process.wait()).await,
            Ok(Ok(_))
        ) {
            let _ = c.process.kill();
            let _ = c.process.wait().await;
        }
    }
}
fn protocol(id: Value, code: i32, message: &str) -> Value {
    json!({"jsonrpc":"2.0","id":id,"error":{"code":code,"message":message}})
}
fn tool_ok(id: Value, view: &SessionView) -> Value {
    let value = serde_json::to_value(view).unwrap();
    json!({"jsonrpc":"2.0","id":id,"result":{"content":[{"type":"text","text":value.to_string()}],"structuredContent":value,"isError":false}})
}
fn tool_error(id: Value, e: SafeError) -> Value {
    let value = serde_json::to_value(e).unwrap();
    json!({"jsonrpc":"2.0","id":id,"result":{"content":[{"type":"text","text":value.to_string()}],"structuredContent":value,"isError":true}})
}
fn tools() -> Value {
    json!({"tools":[{"name":"start_reading_session","description":"GUIで音声と有料送信を確認するセッションを開始する。送信は利用者の操作が必要。","inputSchema":{"type":"object","properties":{"reference_text":{"type":"string"}},"required":["reference_text"],"additionalProperties":false}},{"name":"get_reading_result","description":"現在状態を取得する。送信を行わない。","inputSchema":{"type":"object","properties":{"session_id":{"type":"string"}},"required":["session_id"],"additionalProperties":false}},{"name":"cancel_reading_session","description":"ローカル評価を取り消す。課金取消しを保証しない。","inputSchema":{"type":"object","properties":{"session_id":{"type":"string"}},"required":["session_id"],"additionalProperties":false}}]})
}
async fn start_gui(
    launcher: &Arc<dyn GuiLauncher>,
    view: &SessionView,
    reference: ReferenceText,
    tx: mpsc::Sender<Result<ChildMessage, SafeError>>,
) -> Result<Child, SafeError> {
    let mut gui = tokio::time::timeout(Duration::from_secs(5), launcher.launch())
        .await
        .map_err(|_| err(ErrorCode::GuiLaunch))?
        .map_err(|_| err(ErrorCode::GuiLaunch))?;
    let ready = async {
        write_message(
            &mut *gui.input,
            &ParentMessage::Boot {
                session_id: view.session_id.clone(),
                reference_text: reference.as_str().to_owned(),
            },
            IPC_LIMIT,
        )
        .await?;
        let line = read_line(&mut *gui.output, IPC_LIMIT)
            .await?
            .ok_or_else(|| err(ErrorCode::GuiLaunch))?;
        match serde_json::from_slice::<ChildMessage>(&line) {
            Ok(ChildMessage::Ready { session_id }) if session_id == view.session_id => Ok(()),
            _ => Err(err(ErrorCode::GuiLaunch)),
        }
    };
    if !matches!(
        tokio::time::timeout(Duration::from_secs(5), ready).await,
        Ok(Ok(()))
    ) {
        let _ = gui.process.kill();
        let _ = gui.process.wait().await;
        return Err(err(ErrorCode::GuiLaunch));
    }
    let mut output = gui.output;
    let reader = tokio::spawn(async move {
        loop {
            let message = match read_line(&mut *output, IPC_LIMIT).await {
                Ok(Some(line)) => serde_json::from_slice::<ChildMessage>(&line)
                    .map_err(|_| err(ErrorCode::GuiDisconnected)),
                _ => Err(err(ErrorCode::GuiDisconnected)),
            };
            let end = message.is_err();
            if tx.send(message).await.is_err() || end {
                break;
            }
        }
    });
    Ok(Child {
        input: gui.input,
        process: gui.process,
        reader,
        id: view.session_id.clone(),
        reference,
        deadline: None,
    })
}
async fn tool_call(
    id: Value,
    params: Value,
    sessions: &mut SessionManager,
    child: &mut Option<Child>,
    launcher: &Arc<dyn GuiLauncher>,
    child_tx: &mpsc::Sender<Result<ChildMessage, SafeError>>,
    child_rx: &mut mpsc::Receiver<Result<ChildMessage, SafeError>>,
) -> Value {
    let Some(obj) = params.as_object() else {
        return protocol(id, -32602, "Invalid params");
    };
    if params.get("_meta").is_some_and(|meta| {
        !meta.is_object()
            || meta
                .get("progressToken")
                .is_some_and(|token| !(token.is_string() || token.is_i64() || token.is_u64()))
    }) {
        return protocol(id, -32602, "Invalid params");
    }
    if obj
        .keys()
        .any(|k| !["name", "arguments", "_meta"].contains(&k.as_str()))
    {
        return protocol(id, -32602, "Invalid params");
    }
    let Some(name) = params.get("name").and_then(Value::as_str) else {
        return protocol(id, -32602, "Invalid params");
    };
    let Some(args) = params.get("arguments").and_then(Value::as_object) else {
        return protocol(id, -32602, "Invalid params");
    };
    let field = if name == "start_reading_session" {
        "reference_text"
    } else {
        "session_id"
    };
    if ![
        "start_reading_session",
        "get_reading_result",
        "cancel_reading_session",
    ]
    .contains(&name)
        || args.len() != 1
        || !args.get(field).is_some_and(Value::is_string)
    {
        return protocol(id, -32602, "Invalid params");
    }
    if name == "start_reading_session" {
        let reference = match ReferenceText::new(args[field].as_str().unwrap().to_owned()) {
            Ok(r) => r,
            Err(e) => return tool_error(id, e),
        };
        let view = match sessions.start() {
            Ok(v) => v,
            Err(e) => return tool_error(id, e),
        };
        close_child(child).await;
        while child_rx.try_recv().is_ok() {}
        match start_gui(launcher, &view, reference, child_tx.clone()).await {
            Ok(c) => {
                *child = Some(c);
                tool_ok(id, &view)
            }
            Err(e) => {
                let _ = sessions.fail(&view.session_id, e.clone());
                tool_error(id, e)
            }
        }
    } else {
        let session_id = match SessionId::parse(args[field].as_str().unwrap()) {
            Ok(s) => s,
            Err(e) => return tool_error(id, e),
        };
        let view = if name == "get_reading_result" {
            sessions.get(&session_id)
        } else {
            sessions.cancel(&session_id)
        };
        match view {
            Err(e) => tool_error(id, e),
            Ok(view) => {
                if name == "cancel_reading_session" {
                    if let Some(c) = child.as_mut().filter(|c| c.id == session_id) {
                        let _ = write_message(
                            &mut *c.input,
                            &ParentMessage::Cancel { session_id },
                            IPC_LIMIT,
                        )
                        .await;
                        let _ = write_message(
                            &mut *c.input,
                            &ParentMessage::Committed { view: view.clone() },
                            IPC_LIMIT,
                        )
                        .await;
                    }
                }
                tool_ok(id, &view)
            }
        }
    }
}
async fn child_event(
    message: Option<Result<ChildMessage, SafeError>>,
    sessions: &mut SessionManager,
    c: &mut Child,
) -> Result<(), SafeError> {
    let current = sessions.get(&c.id)?;
    let view = match message {
        Some(Ok(ChildMessage::SendIntent {
            session_id,
            reference_text,
        })) if session_id == c.id => {
            if current.state.terminal() {
                Some(current)
            } else {
                match ReferenceText::new(reference_text) {
                    Ok(reference) => {
                        c.reference = reference;
                        match sessions.mark_running(&c.id) {
                            Ok(_) => {
                                c.deadline =
                                    Some(tokio::time::Instant::now() + Duration::from_secs(180));
                                if write_message(
                                    &mut *c.input,
                                    &ParentMessage::SendGranted {
                                        session_id: c.id.clone(),
                                    },
                                    IPC_LIMIT,
                                )
                                .await
                                .is_err()
                                {
                                    Some(
                                        sessions
                                            .fail(&c.id, sent_err(ErrorCode::GuiDisconnected))?,
                                    )
                                } else {
                                    None
                                }
                            }
                            Err(_) => {
                                Some(sessions.fail(&c.id, sent_err(ErrorCode::GuiDisconnected))?)
                            }
                        }
                    }
                    Err(_) => Some(sessions.fail(&c.id, err(ErrorCode::ProtocolInvalid))?),
                }
            }
        }
        Some(Ok(ChildMessage::Completed { session_id, result })) if session_id == c.id => {
            if current.state.terminal() {
                Some(current)
            } else {
                match crate::provider::validate_result(&result, &c.reference) {
                    Ok(()) => match sessions.complete(&c.id, result) {
                        Ok(v) => Some(v),
                        Err(_) => Some(sessions.fail(&c.id, err(ErrorCode::GuiDisconnected))?),
                    },
                    Err(e) => Some(sessions.fail(&c.id, e)?),
                }
            }
        }
        Some(Ok(ChildMessage::Failed { session_id, error })) if session_id == c.id => {
            Some(sessions.fail(&c.id, error)?)
        }
        Some(Ok(ChildMessage::Closed { session_id })) if session_id == c.id => {
            Some(sessions.cancel(&c.id)?)
        }
        _ => {
            let sent = if matches!(current.state, SessionState::Running) {
                SendDisposition::MayHaveBeenSent
            } else {
                SendDisposition::NotSent
            };
            Some(sessions.fail(&c.id, SafeError::new(ErrorCode::GuiDisconnected, sent))?)
        }
    };
    if let Some(view) = view {
        c.deadline = None;
        c.reference = ReferenceText::new("discarded".into()).unwrap();
        let _ = write_message(&mut *c.input, &ParentMessage::Committed { view }, IPC_LIMIT).await;
    }
    Ok(())
}
async fn request(
    line: &[u8],
    sessions: &mut SessionManager,
    child: &mut Option<Child>,
    launcher: &Arc<dyn GuiLauncher>,
    child_tx: &mpsc::Sender<Result<ChildMessage, SafeError>>,
    child_rx: &mut mpsc::Receiver<Result<ChildMessage, SafeError>>,
) -> Option<Value> {
    let value: Value = match serde_json::from_slice(line) {
        Ok(v) => v,
        Err(_) => return Some(protocol(Value::Null, -32700, "Parse error")),
    };
    let id = value.get("id").cloned().unwrap_or(Value::Null);
    let valid = value.as_object().is_some_and(|o| {
        o.keys()
            .all(|k| ["jsonrpc", "id", "method", "params"].contains(&k.as_str()))
    }) && value.get("jsonrpc").and_then(Value::as_str) == Some("2.0")
        && value.get("method").is_some_and(Value::is_string)
        && !value
            .get("id")
            .is_some_and(|i| !(i.is_string() || i.is_number() || i.is_null()));
    if !valid {
        return Some(protocol(id, -32600, "Invalid request"));
    }
    if value.get("id").is_none() {
        return None;
    }
    let method = value["method"].as_str().unwrap();
    let params = value.get("params").cloned().unwrap_or(json!({}));
    Some(match method {
        "initialize" => {
            json!({"jsonrpc":"2.0","id":id,"result":{"protocolVersion":"2025-06-18","capabilities":{"tools":{}},"serverInfo":{"name":"qwen-audio","version":"0.1.0"}}})
        }
        "ping" => json!({"jsonrpc":"2.0","id":id,"result":{}}),
        "tools/list" => json!({"jsonrpc":"2.0","id":id,"result":tools()}),
        "tools/call" => tool_call(id, params, sessions, child, launcher, child_tx, child_rx).await,
        _ => protocol(id, -32601, "Method not found"),
    })
}
pub async fn serve_mcp<R, W>(
    mut reader: R,
    mut writer: W,
    launcher: Arc<dyn GuiLauncher>,
) -> Result<(), SafeError>
where
    R: AsyncRead + Unpin + Send + 'static,
    W: AsyncWrite + Unpin + Send + 'static,
{
    let (tx, mut rx) = mpsc::channel(8);
    let input = tokio::spawn(async move {
        loop {
            let event = match read_line(&mut reader, 128 * 1024).await {
                Ok(Some(line)) => Input::Line(line),
                Ok(None) => Input::Closed,
                Err(_) => Input::Invalid,
            };
            let end = !matches!(event, Input::Line(_));
            if tx.send(event).await.is_err() || end {
                break;
            }
        }
    });
    let (child_tx, mut child_rx) = mpsc::channel(8);
    let mut sessions = SessionManager::new();
    let mut child: Option<Child> = None;
    let mut outcome = Ok(());
    loop {
        let deadline = child.as_ref().and_then(|c| c.deadline);
        tokio::select! {biased;
        event=rx.recv()=>{
            let line=match event {
                Some(Input::Line(line))=>line,
                Some(Input::Invalid)=>{
                    let _=write_message(&mut writer,&protocol(Value::Null,-32600,"Invalid request"),IPC_LIMIT).await;
                    break;
                }
                _=>break,
            };
            if let Some(response)=request(&line,&mut sessions,&mut child,&launcher,&child_tx,&mut child_rx).await {
                if write_message(&mut writer,&response,IPC_LIMIT).await.is_err() {
                    outcome=Err(err(ErrorCode::ProtocolInvalid));
                    break;
                }
            }
        },
        _=async{if let Some(deadline)=deadline{tokio::time::sleep_until(deadline).await}else{std::future::pending::<()>().await}},if deadline.is_some()=>{
            if let Some(c)=child.as_mut(){
                c.deadline=None;
                let view=sessions.fail(&c.id,sent_err(ErrorCode::Timeout))?;
                let _=write_message(&mut *c.input,&ParentMessage::Cancel{session_id:c.id.clone()},IPC_LIMIT).await;
                let _=write_message(&mut *c.input,&ParentMessage::Committed{view},IPC_LIMIT).await;
            }
        },
        message=child_rx.recv(),if child.is_some()=>{
            if let Some(c)=child.as_mut(){
                if let Err(e)=child_event(message,&mut sessions,c).await{outcome=Err(e);break}
            }
        },
        _=async{if let Some(c)=child.as_mut(){let _=c.process.wait().await;}else{std::future::pending::<()>().await}},if child.is_some()=>{
            if let Some(c)=child.as_ref(){
                let current=sessions.get(&c.id)?;
                let sent=if matches!(current.state,SessionState::Running){SendDisposition::MayHaveBeenSent}else{SendDisposition::NotSent};
                let _=sessions.fail(&c.id,SafeError::new(ErrorCode::GuiDisconnected,sent));
            }
            close_child(&mut child).await;
        }
        }
    }
    if let Some(c) = child.as_mut() {
        if let Ok(v) = sessions.cancel(&c.id) {
            let _ = write_message(
                &mut *c.input,
                &ParentMessage::Cancel {
                    session_id: c.id.clone(),
                },
                IPC_LIMIT,
            )
            .await;
            let _ = write_message(
                &mut *c.input,
                &ParentMessage::Committed { view: v },
                IPC_LIMIT,
            )
            .await;
        }
    }
    close_child(&mut child).await;
    input.abort();
    outcome
}
