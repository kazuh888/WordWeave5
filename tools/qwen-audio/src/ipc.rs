use crate::error::err;
use crate::*;
use serde::{Deserialize, Serialize};
use std::{pin::Pin, sync::Arc, time::Duration};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
#[derive(Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case", deny_unknown_fields)]
pub enum ParentMessage {
    Boot {
        session_id: SessionId,
        reference_text: String,
    },
    SendGranted {
        session_id: SessionId,
    },
    Committed {
        view: SessionView,
    },
    Cancel {
        session_id: SessionId,
    },
    Close,
}
#[derive(Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case", deny_unknown_fields)]
pub enum ChildMessage {
    Ready {
        session_id: SessionId,
    },
    SendIntent {
        session_id: SessionId,
        reference_text: String,
    },
    Completed {
        session_id: SessionId,
        result: EvaluationResult,
    },
    Failed {
        session_id: SessionId,
        error: SafeError,
    },
    Closed {
        session_id: SessionId,
    },
}
pub type IoFuture<'a, T> =
    Pin<Box<dyn std::future::Future<Output = Result<T, SafeError>> + Send + 'a>>;
pub trait GuiProcess: Send {
    fn wait<'a>(&'a mut self) -> IoFuture<'a, Option<i32>>;
    fn kill(&mut self) -> Result<(), SafeError>;
}
pub struct LaunchedGui {
    pub input: Box<dyn AsyncWrite + Unpin + Send>,
    pub output: Box<dyn AsyncRead + Unpin + Send>,
    pub process: Box<dyn GuiProcess>,
}
pub trait GuiLauncher: Send + Sync {
    fn launch<'a>(&'a self) -> IoFuture<'a, LaunchedGui>;
}
#[doc(hidden)]
pub async fn read_child_boot<R: AsyncRead + Unpin>(
    reader: &mut R,
) -> Result<ParentMessage, SafeError> {
    let line = read_line(reader, 512 * 1024)
        .await?
        .ok_or_else(|| err(ErrorCode::GuiDisconnected))?;
    serde_json::from_slice(&line).map_err(|_| err(ErrorCode::ProtocolInvalid))
}
#[doc(hidden)]
pub async fn write_child_event<W: AsyncWrite + Unpin>(
    writer: &mut W,
    value: &ChildMessage,
) -> Result<(), SafeError> {
    write_message(writer, value, 512 * 1024).await
}
pub(crate) async fn read_line<R: AsyncRead + Unpin + ?Sized>(
    reader: &mut R,
    max: usize,
) -> Result<Option<Vec<u8>>, SafeError> {
    let mut line = Vec::new();
    loop {
        let mut b = [0];
        let n = reader
            .read(&mut b)
            .await
            .map_err(|_| err(ErrorCode::ProtocolInvalid))?;
        if n == 0 {
            return if line.is_empty() {
                Ok(None)
            } else {
                Err(err(ErrorCode::ProtocolInvalid))
            };
        }
        if b[0] == b'\n' {
            if line.last() == Some(&b'\r') {
                line.pop();
            }
            return Ok(Some(line));
        }
        if line.len() >= max {
            return Err(err(ErrorCode::ProtocolInvalid));
        }
        line.push(b[0]);
    }
}
pub(crate) async fn write_message<W: AsyncWrite + Unpin + ?Sized, T: Serialize>(
    writer: &mut W,
    value: &T,
    max: usize,
) -> Result<(), SafeError> {
    let mut bytes = serde_json::to_vec(value).map_err(|_| err(ErrorCode::ProtocolInvalid))?;
    if bytes.len() > max {
        return Err(err(ErrorCode::ProtocolInvalid));
    }
    bytes.push(b'\n');
    tokio::time::timeout(Duration::from_secs(2), async {
        writer.write_all(&bytes).await?;
        writer.flush().await
    })
    .await
    .map_err(|_| err(ErrorCode::GuiDisconnected))?
    .map_err(|_| err(ErrorCode::GuiDisconnected))
}
#[cfg(all(windows, feature = "windows-job"))]
struct Job(windows::Win32::Foundation::HANDLE);
#[cfg(all(windows, feature = "windows-job"))]
unsafe impl Send for Job {}
#[cfg(all(windows, feature = "windows-job"))]
unsafe impl Sync for Job {}
#[cfg(all(windows, feature = "windows-job"))]
impl Drop for Job {
    fn drop(&mut self) {
        unsafe {
            let _ = windows::Win32::Foundation::CloseHandle(self.0);
        }
    }
}
#[cfg(all(windows, feature = "windows-job"))]
pub struct NativeGuiLauncher {
    job: Arc<Job>,
}
#[cfg(all(windows, feature = "windows-job"))]
impl NativeGuiLauncher {
    pub fn new() -> Result<Self, SafeError> {
        use windows::{core::PCWSTR, Win32::System::JobObjects::*};
        unsafe {
            let h =
                CreateJobObjectW(None, PCWSTR::null()).map_err(|_| err(ErrorCode::GuiLaunch))?;
            let job = Arc::new(Job(h));
            let mut info = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
            info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
            SetInformationJobObject(
                h,
                JobObjectExtendedLimitInformation,
                &info as *const _ as *const _,
                std::mem::size_of_val(&info) as u32,
            )
            .map_err(|_| err(ErrorCode::GuiLaunch))?;
            Ok(Self { job })
        }
    }
}
#[cfg(all(windows, feature = "windows-job"))]
struct NativeProcess {
    child: tokio::process::Child,
    _job: Arc<Job>,
}
#[cfg(all(windows, feature = "windows-job"))]
impl GuiProcess for NativeProcess {
    fn wait<'a>(&'a mut self) -> IoFuture<'a, Option<i32>> {
        Box::pin(async move {
            self.child
                .wait()
                .await
                .map(|s| s.code())
                .map_err(|_| err(ErrorCode::GuiDisconnected))
        })
    }
    fn kill(&mut self) -> Result<(), SafeError> {
        self.child
            .start_kill()
            .map_err(|_| err(ErrorCode::GuiDisconnected))
    }
}
#[cfg(all(windows, feature = "windows-job"))]
impl GuiLauncher for NativeGuiLauncher {
    fn launch<'a>(&'a self) -> IoFuture<'a, LaunchedGui> {
        Box::pin(async move {
            use windows::Win32::{
                Foundation::CloseHandle,
                System::{JobObjects::*, Threading::*},
            };
            let exe = std::env::current_exe().map_err(|_| err(ErrorCode::GuiLaunch))?;
            let mut child = tokio::process::Command::new(exe)
                .arg("--gui-child")
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::null())
                .kill_on_drop(true)
                .creation_flags(0x08000000)
                .spawn()
                .map_err(|_| err(ErrorCode::GuiLaunch))?;
            let assign = unsafe {
                OpenProcess(
                    PROCESS_SET_QUOTA | PROCESS_TERMINATE,
                    false,
                    child.id().ok_or_else(|| err(ErrorCode::GuiLaunch))?,
                )
                .and_then(|h| {
                    let r = AssignProcessToJobObject(self.job.0, h);
                    let _ = CloseHandle(h);
                    r
                })
            };
            if assign.is_err() {
                let _ = child.start_kill();
                let _ = child.wait().await;
                return Err(err(ErrorCode::GuiLaunch));
            }
            let input = child
                .stdin
                .take()
                .ok_or_else(|| err(ErrorCode::GuiLaunch))?;
            let output = child
                .stdout
                .take()
                .ok_or_else(|| err(ErrorCode::GuiLaunch))?;
            Ok(LaunchedGui {
                input: Box::new(input),
                output: Box::new(output),
                process: Box::new(NativeProcess {
                    child,
                    _job: self.job.clone(),
                }),
            })
        })
    }
}
