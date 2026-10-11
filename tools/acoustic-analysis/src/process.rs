//! Private bounded child-process interface; never exposes diagnostic bytes on error.

use std::{ffi::OsString, path::PathBuf, time::Instant};
use tokio_util::sync::CancellationToken;

use crate::AnalysisErrorKind;

#[cfg(windows)]
#[path = "process_windows.rs"]
mod platform;

fn checkpoint(deadline: Instant, cancel: &CancellationToken) -> Result<(), ProcessFailure> {
    let kind = if cancel.is_cancelled() {
        AnalysisErrorKind::Cancelled
    } else if Instant::now() >= deadline {
        AnalysisErrorKind::TimedOut
    } else {
        return Ok(());
    };
    Err(ProcessFailure {
        kind,
        reason: ProcessFailureReason::Wait,
    })
}

pub(crate) struct ProcessSpec {
    pub executable: PathBuf,
    pub args: Vec<OsString>,
    pub cwd: PathBuf,
}

pub(crate) struct ProcessLimits {
    pub stdout_bytes: usize,
    pub stderr_bytes: usize,
    pub memory_bytes: usize,
}

#[derive(Debug)]
pub(crate) struct ProcessOutput {
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub exit_code: u32,
    pub peak_job_bytes: usize,
    pub memory_limit_observed: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ProcessFailureReason {
    Busy,
    Setup,
    Assign,
    Resume,
    Pipe,
    Wait,
    OutputLimit,
    MemoryLimit,
    UnclassifiedAbnormalExit,
    BackendDiagnostic,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ProcessFailure {
    pub kind: AnalysisErrorKind,
    pub reason: ProcessFailureReason,
}

/// A failed OS reap permanently prevents releasing the adapter's process lease.
/// This is separate from the originally latched cancellation/deadline error.
pub(crate) fn reap_unconfirmed() -> bool {
    #[cfg(windows)]
    {
        platform::reap_unconfirmed()
    }
    #[cfg(not(windows))]
    {
        false
    }
}

pub(crate) fn supervise(
    spec: &ProcessSpec,
    limits: &ProcessLimits,
    deadline: Instant,
    cancel: &CancellationToken,
) -> Result<ProcessOutput, ProcessFailure> {
    checkpoint(deadline, cancel)?;
    #[cfg(windows)]
    {
        platform::run(
            spec,
            limits,
            deadline,
            cancel,
            #[cfg(test)]
            None,
        )
    }
    #[cfg(not(windows))]
    {
        let _ = (spec, limits);
        Err(ProcessFailure {
            kind: AnalysisErrorKind::BackendUnavailable,
            reason: ProcessFailureReason::Setup,
        })
    }
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FaultPoint {
    AfterCreate,
    BeforeAssign,
    AfterAssign,
    BeforeResume,
    BeforeCleanup,
}

#[cfg(test)]
pub(crate) fn supervise_with_hook(
    spec: &ProcessSpec,
    limits: &ProcessLimits,
    deadline: Instant,
    cancel: &CancellationToken,
    hook: &dyn Fn(FaultPoint, u32) -> Result<(), ProcessFailure>,
) -> Result<ProcessOutput, ProcessFailure> {
    checkpoint(deadline, cancel)?;
    #[cfg(windows)]
    {
        platform::run(spec, limits, deadline, cancel, Some(hook))
    }
    #[cfg(not(windows))]
    {
        let _ = hook;
        supervise(spec, limits, deadline, cancel)
    }
}
