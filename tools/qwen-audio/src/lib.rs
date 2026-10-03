mod audio;
mod audio_backend;
mod audio_jobs;
mod audio_parser;
#[cfg(feature = "gui")]
mod controls;
mod credentials;
mod error;
#[cfg(feature = "gui")]
pub mod gui;
#[cfg(feature = "mcp")]
mod ipc;
#[cfg(feature = "mcp")]
mod mcp;
mod probe;
mod provider;
#[cfg(feature = "mcp")]
mod session;
pub use audio::{read_audio, read_wav, AudioInfo, AudioInput, ReferenceText};
pub use audio_backend::take_audio_cleanup_warning;
pub use audio_jobs::{shutdown_audio_jobs, AudioLoadJob};
#[cfg(all(windows, feature = "windows-credentials"))]
pub use credentials::WindowsCredentialStore;
pub use credentials::{ApiHost, ApiKey, Connection, CredentialStore, Region, TokyoHost};
pub use error::{ErrorCode, SafeError, SendDisposition};
#[cfg(all(windows, feature = "windows-job"))]
pub use ipc::NativeGuiLauncher;
#[cfg(feature = "mcp")]
pub use ipc::{
    read_child_boot, write_child_event, ChildMessage, GuiLauncher, GuiProcess, IoFuture,
    LaunchedGui, ParentMessage,
};
#[cfg(feature = "mcp")]
pub use mcp::serve_mcp;
pub use probe::{
    probe_connection, ProbeError, ProbeRequest, ProbeSuccess, ProbeTransport, ReqwestProbeTransport,
};
pub use provider::{
    evaluate, validate_feedback, Assessment, BodyStream, EvaluationResult, EvaluationSnapshot,
    Feedback, Improvement, ProviderRequest, ReqwestTransport, Transport, TransportFailure,
    TransportFuture, TransportResponse, Usage,
};
#[cfg(feature = "mcp")]
pub use session::{SessionId, SessionManager, SessionState, SessionView};
pub const REQUESTED_MODEL: &str = "qwen3.8-omni-flash";
pub const REQUESTED_EFFORT: &str = "medium";
pub const MAX_AUDIO_BYTES: usize = 6 * 1024 * 1024;
pub const MAX_REFERENCE_SCALARS: usize = 10_000;
