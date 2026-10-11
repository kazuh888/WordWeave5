//! Windows-only ownership and containment for a single bounded child tree.

use super::*;
use std::{
    ffi::OsStr,
    mem::size_of,
    os::windows::ffi::OsStrExt,
    sync::atomic::{AtomicBool, Ordering},
    time::{Duration, Instant},
};
use tokio_util::sync::CancellationToken;
use windows::core::{HRESULT, PCWSTR, PWSTR};
use windows::Win32::{
    Foundation::{
        CloseHandle, SetHandleInformation, BOOL, ERROR_BROKEN_PIPE, GENERIC_READ, HANDLE,
        HANDLE_FLAGS, HANDLE_FLAG_INHERIT, INVALID_HANDLE_VALUE, WAIT_OBJECT_0, WAIT_TIMEOUT,
    },
    Security::SECURITY_ATTRIBUTES,
    Storage::FileSystem::{
        CreateFileW, ReadFile, FILE_ATTRIBUTE_NORMAL, FILE_SHARE_READ, FILE_SHARE_WRITE,
        OPEN_EXISTING,
    },
    System::{
        JobObjects::*,
        Pipes::{CreatePipe, PeekNamedPipe},
        SystemServices::{JOB_OBJECT_MSG_JOB_MEMORY_LIMIT, JOB_OBJECT_MSG_PROCESS_MEMORY_LIMIT},
        Threading::*,
        IO::{CreateIoCompletionPort, GetQueuedCompletionStatus},
    },
};

fn failure(reason: ProcessFailureReason) -> ProcessFailure {
    ProcessFailure {
        kind: AnalysisErrorKind::ResourceLimit,
        reason,
    }
}

struct Handle(HANDLE);

impl Drop for Handle {
    fn drop(&mut self) {
        // SAFETY: each Handle uniquely owns one valid non-pseudo OS handle.
        unsafe {
            let _ = CloseHandle(self.0);
        }
    }
}

struct Job {
    handle: Handle,
    port: Handle,
}

// A failed reap prevents further uncontained work, including after unwinding.
static REAP_FAILED: AtomicBool = AtomicBool::new(false);

pub(super) fn reap_unconfirmed() -> bool {
    REAP_FAILED.load(Ordering::Acquire)
}

struct ChildGuard {
    job: Job,
    process: Handle,
    thread: Handle,
    stdout: Handle,
    stderr: Handle,
    pid: u32,
    assigned: bool,
    reaped: bool,
}

impl ChildGuard {
    fn stop(&mut self) -> Result<(), ProcessFailure> {
        if self.reaped {
            return Ok(());
        }
        let result = self.stop_inner();
        if result.is_err() {
            REAP_FAILED.store(true, Ordering::SeqCst);
        }
        result
    }

    fn stop_inner(&mut self) -> Result<(), ProcessFailure> {
        // SAFETY: these handles uniquely identify our child and its private Job.
        // Termination is followed by positive wait/accounting evidence, not PID lookup.
        unsafe {
            if self.assigned {
                TerminateJobObject(self.job.handle.0, 1)
                    .map_err(|_| failure(ProcessFailureReason::Wait))?;
            } else if WaitForSingleObject(self.process.0, 0) != WAIT_OBJECT_0 {
                TerminateProcess(self.process.0, 1)
                    .map_err(|_| failure(ProcessFailureReason::Wait))?;
            }
            if WaitForSingleObject(self.process.0, INFINITE) != WAIT_OBJECT_0 {
                return Err(failure(ProcessFailureReason::Wait));
            }
        }
        if self.assigned {
            while self.job.active()? != 0 {
                std::thread::sleep(Duration::from_millis(5));
            }
        }
        self.reaped = true;
        Ok(())
    }
}

impl Drop for ChildGuard {
    fn drop(&mut self) {
        // No panic or formatting during unwind. Job close is an additional
        // kill-on-close safeguard if explicit stopping cannot be confirmed.
        let _ = self.stop();
    }
}

impl Job {
    /// Drain a bounded number of notifications per observation round. `true`
    /// means the queue reached empty; success waits for that evidence too.
    fn notifications(&self) -> Result<bool, ProcessFailure> {
        for _ in 0..64 {
            let (mut message, mut key, mut overlapped) = (0, 0, std::ptr::null_mut());
            // SAFETY: completion data are integer/PID payloads, never dereferenced.
            match unsafe {
                GetQueuedCompletionStatus(self.port.0, &mut message, &mut key, &mut overlapped, 0)
            } {
                Ok(()) => {
                    if key != 1 {
                        return Err(failure(ProcessFailureReason::Wait));
                    }
                    if matches!(
                        message,
                        JOB_OBJECT_MSG_JOB_MEMORY_LIMIT | JOB_OBJECT_MSG_PROCESS_MEMORY_LIMIT
                    ) {
                        return Err(failure(ProcessFailureReason::MemoryLimit));
                    }
                }
                Err(error) if error.code() == HRESULT::from_win32(WAIT_TIMEOUT.0) => {
                    return Ok(true)
                }
                Err(_) => return Err(failure(ProcessFailureReason::Wait)),
            }
        }
        Ok(false)
    }
    fn active(&self) -> Result<u32, ProcessFailure> {
        let mut value = JOBOBJECT_BASIC_ACCOUNTING_INFORMATION::default();
        // SAFETY: owned Job and exact output structure size.
        unsafe {
            QueryInformationJobObject(
                self.handle.0,
                JobObjectBasicAccountingInformation,
                (&mut value as *mut JOBOBJECT_BASIC_ACCOUNTING_INFORMATION).cast(),
                size_of::<JOBOBJECT_BASIC_ACCOUNTING_INFORMATION>() as u32,
                None,
            )
        }
        .map_err(|_| failure(ProcessFailureReason::Wait))?;
        Ok(value.ActiveProcesses)
    }
    fn new(memory: usize) -> Result<Self, ProcessFailure> {
        if memory == 0 {
            return Err(failure(ProcessFailureReason::Setup));
        }
        // SAFETY: anonymous handles are non-inheritable; all structures remain
        // live for the synchronous calls, with their exact C-layout byte sizes.
        unsafe {
            let handle = Handle(
                CreateJobObjectW(None, None).map_err(|_| failure(ProcessFailureReason::Setup))?,
            );
            let port = Handle(
                CreateIoCompletionPort(INVALID_HANDLE_VALUE, HANDLE::default(), 0, 1)
                    .map_err(|_| failure(ProcessFailureReason::Setup))?,
            );
            let mut limits = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
            limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_PROCESS_MEMORY
                | JOB_OBJECT_LIMIT_JOB_MEMORY
                | JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
            limits.ProcessMemoryLimit = memory;
            limits.JobMemoryLimit = memory;
            SetInformationJobObject(
                handle.0,
                JobObjectExtendedLimitInformation,
                (&limits as *const JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
                size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            )
            .map_err(|_| failure(ProcessFailureReason::Setup))?;
            let association = JOBOBJECT_ASSOCIATE_COMPLETION_PORT {
                // An opaque integer completion key, never a dereferenced pointer.
                CompletionKey: std::ptr::without_provenance_mut(1),
                CompletionPort: port.0,
            };
            SetInformationJobObject(
                handle.0,
                JobObjectAssociateCompletionPortInformation,
                (&association as *const JOBOBJECT_ASSOCIATE_COMPLETION_PORT).cast(),
                size_of::<JOBOBJECT_ASSOCIATE_COMPLETION_PORT>() as u32,
            )
            .map_err(|_| failure(ProcessFailureReason::Setup))?;
            let job = Self { handle, port };
            let confirmed = job.limits()?;
            if confirmed.BasicLimitInformation.LimitFlags != limits.BasicLimitInformation.LimitFlags
                || confirmed.ProcessMemoryLimit != memory
                || confirmed.JobMemoryLimit != memory
            {
                return Err(failure(ProcessFailureReason::Setup));
            }
            Ok(job)
        }
    }

    fn limits(&self) -> Result<JOBOBJECT_EXTENDED_LIMIT_INFORMATION, ProcessFailure> {
        let mut value = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
        // SAFETY: live owned Job, correctly sized writable output structure.
        unsafe {
            QueryInformationJobObject(
                self.handle.0,
                JobObjectExtendedLimitInformation,
                (&mut value as *mut JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
                size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
                None,
            )
        }
        .map_err(|_| failure(ProcessFailureReason::Wait))?;
        Ok(value)
    }
}

fn security() -> SECURITY_ATTRIBUTES {
    SECURITY_ATTRIBUTES {
        nLength: size_of::<SECURITY_ATTRIBUTES>() as u32,
        lpSecurityDescriptor: std::ptr::null_mut(),
        bInheritHandle: BOOL(1),
    }
}

fn pipe() -> Result<(Handle, Handle), ProcessFailure> {
    let (mut read, mut write) = (HANDLE::default(), HANDLE::default());
    // SAFETY: valid writable handle slots and synchronous security attributes.
    unsafe { CreatePipe(&mut read, &mut write, Some(&security()), 8192) }
        .map_err(|_| failure(ProcessFailureReason::Setup))?;
    let (read, write) = (Handle(read), Handle(write));
    // Only the write side belongs in the child's explicit inheritance list.
    unsafe { SetHandleInformation(read.0, HANDLE_FLAG_INHERIT.0, HANDLE_FLAGS(0)) }
        .map_err(|_| failure(ProcessFailureReason::Setup))?;
    Ok((read, write))
}

struct Attributes<'a> {
    storage: Vec<usize>,
    initialized: bool,
    // Attribute values are borrowed by Windows until the list is deleted.
    handles: &'a [HANDLE],
    jobs: &'a [HANDLE],
}
impl<'a> Attributes<'a> {
    fn pointer(&mut self) -> LPPROC_THREAD_ATTRIBUTE_LIST {
        LPPROC_THREAD_ATTRIBUTE_LIST(self.storage.as_mut_ptr().cast())
    }

    fn new(handles: &'a [HANDLE], jobs: &'a [HANDLE]) -> Result<Self, ProcessFailure> {
        let mut bytes = 0;
        // SAFETY: the first call only obtains required bytes. usize storage
        // provides pointer alignment and remains live until list deletion.
        unsafe {
            let _ = InitializeProcThreadAttributeList(
                LPPROC_THREAD_ATTRIBUTE_LIST::default(),
                2,
                0,
                &mut bytes,
            );
        }
        if bytes == 0 {
            return Err(failure(ProcessFailureReason::Setup));
        }
        let count = bytes.div_ceil(size_of::<usize>());
        let mut storage = Vec::new();
        storage
            .try_reserve_exact(count)
            .map_err(|_| failure(ProcessFailureReason::Setup))?;
        storage.resize(count, 0);
        let mut attributes = Self {
            storage,
            initialized: false,
            handles,
            jobs,
        };
        unsafe {
            InitializeProcThreadAttributeList(attributes.pointer(), 2, 0, &mut bytes)
                .map_err(|_| failure(ProcessFailureReason::Setup))?;
            attributes.initialized = true;
            UpdateProcThreadAttribute(
                attributes.pointer(),
                0,
                PROC_THREAD_ATTRIBUTE_HANDLE_LIST as usize,
                Some(attributes.handles.as_ptr().cast()),
                std::mem::size_of_val(attributes.handles),
                None,
                None,
            )
            .map_err(|_| failure(ProcessFailureReason::Setup))?;
            UpdateProcThreadAttribute(
                attributes.pointer(),
                0,
                PROC_THREAD_ATTRIBUTE_JOB_LIST as usize,
                Some(attributes.jobs.as_ptr().cast()),
                std::mem::size_of_val(attributes.jobs),
                None,
                None,
            )
            .map_err(|_| failure(ProcessFailureReason::Setup))?;
        }
        Ok(attributes)
    }
}
impl Drop for Attributes<'_> {
    fn drop(&mut self) {
        if self.initialized {
            // SAFETY: storage and both borrowed value arrays are still live.
            unsafe {
                DeleteProcThreadAttributeList(self.pointer());
            }
        }
    }
}

fn wide(value: &OsStr) -> Result<Vec<u16>, ProcessFailure> {
    let mut units: Vec<_> = value.encode_wide().collect();
    if units.contains(&0) {
        return Err(failure(ProcessFailureReason::Setup));
    }
    units.push(0);
    Ok(units)
}

fn command_line(spec: &ProcessSpec) -> Result<Vec<u16>, ProcessFailure> {
    let mut result = Vec::new();
    for arg in std::iter::once(spec.executable.as_os_str())
        .chain(spec.args.iter().map(|arg| arg.as_os_str()))
    {
        if !result.is_empty() {
            result.push(b' ' as u16);
        }
        result.push(b'"' as u16);
        let mut slashes = 0;
        for unit in arg.encode_wide() {
            if unit == 0 {
                return Err(failure(ProcessFailureReason::Setup));
            }
            if unit == b'\\' as u16 {
                slashes += 1;
                continue;
            }
            let count = if unit == b'"' as u16 {
                slashes * 2 + 1
            } else {
                slashes
            };
            result.extend(std::iter::repeat_n(b'\\' as u16, count));
            result.push(unit);
            slashes = 0;
        }
        result.extend(std::iter::repeat_n(b'\\' as u16, slashes * 2));
        result.push(b'"' as u16);
    }
    if result.len() >= 32767 {
        return Err(failure(ProcessFailureReason::Setup));
    }
    result.push(0);
    Ok(result)
}

fn spawn(spec: &ProcessSpec, job: Job) -> Result<ChildGuard, ProcessFailure> {
    if !spec.executable.is_absolute() || !spec.cwd.is_absolute() {
        return Err(failure(ProcessFailureReason::Setup));
    }
    let app = wide(spec.executable.as_os_str())?;
    let cwd = wide(spec.cwd.as_os_str())?;
    let mut command = command_line(spec)?;
    let (stdout, child_stdout) = pipe()?;
    let (stderr, child_stderr) = pipe()?;
    let null = wide(OsStr::new("NUL"))?;
    // SAFETY: null-terminated path and live security descriptor for this call.
    let stdin = Handle(
        unsafe {
            CreateFileW(
                PCWSTR(null.as_ptr()),
                GENERIC_READ.0,
                FILE_SHARE_READ | FILE_SHARE_WRITE,
                Some(&security()),
                OPEN_EXISTING,
                FILE_ATTRIBUTE_NORMAL,
                HANDLE::default(),
            )
        }
        .map_err(|_| failure(ProcessFailureReason::Setup))?,
    );
    let handles = [stdin.0, child_stdout.0, child_stderr.0];
    let jobs = [job.handle.0];
    let mut attributes = Attributes::new(&handles, &jobs)?;
    let mut info = STARTUPINFOEXW::default();
    info.StartupInfo.cb = size_of::<STARTUPINFOEXW>() as u32;
    info.StartupInfo.dwFlags = STARTF_USESTDHANDLES;
    info.StartupInfo.hStdInput = stdin.0;
    info.StartupInfo.hStdOutput = child_stdout.0;
    info.StartupInfo.hStdError = child_stderr.0;
    info.lpAttributeList = attributes.pointer();
    let mut process = PROCESS_INFORMATION::default();
    // SAFETY: all strings, both attribute value arrays and STARTUPINFOEX storage
    // remain live. JOB_LIST assigns the suspended child at creation, so even
    // forced parent termination cannot leave it outside kill-on-close ownership.
    unsafe {
        CreateProcessW(
            PCWSTR(app.as_ptr()),
            PWSTR(command.as_mut_ptr()),
            None,
            None,
            true,
            CREATE_SUSPENDED | CREATE_NO_WINDOW | EXTENDED_STARTUPINFO_PRESENT,
            None,
            PCWSTR(cwd.as_ptr()),
            &info.StartupInfo,
            &mut process,
        )
    }
    .map_err(|_| ProcessFailure {
        kind: AnalysisErrorKind::BackendUnavailable,
        reason: ProcessFailureReason::Setup,
    })?;
    let guard = ChildGuard {
        job,
        process: Handle(process.hProcess),
        thread: Handle(process.hThread),
        stdout,
        stderr,
        pid: process.dwProcessId,
        assigned: true,
        reaped: false,
    };
    // Parent copies must close before any reads can reach EOF. Attribute storage
    // is destroyed before either referenced value array goes out of scope.
    drop(attributes);
    drop(stdin);
    drop(child_stdout);
    drop(child_stderr);
    Ok(guard)
}

struct PipeReader {
    // Borrowed raw handle; ChildGuard keeps it live until stop has completed,
    // including on error or unwind. PipeReader never closes it.
    handle: HANDLE,
    bytes: Vec<u8>,
    maximum: usize,
    eof: bool,
}

impl PipeReader {
    fn new(handle: HANDLE, maximum: usize) -> Self {
        Self {
            handle,
            bytes: Vec::new(),
            maximum,
            eof: false,
        }
    }

    fn drain_once(&mut self) -> Result<bool, ProcessFailure> {
        if self.eof {
            return Ok(false);
        }
        let mut available = 0;
        // SAFETY: no other reader owns this pipe. ReadFile consumes at most the
        // bytes observed by PeekNamedPipe, avoiding an unbounded blocking read.
        match unsafe { PeekNamedPipe(self.handle, None, 0, None, Some(&mut available), None) } {
            Err(error) if error.code() == HRESULT::from_win32(ERROR_BROKEN_PIPE.0) => {
                self.eof = true;
                return Ok(false);
            }
            Err(_) => return Err(failure(ProcessFailureReason::Pipe)),
            Ok(()) => {}
        }
        if available == 0 {
            return Ok(false);
        }
        if available as usize > self.maximum - self.bytes.len() {
            return Err(failure(ProcessFailureReason::OutputLimit));
        }
        let mut buffer = [0_u8; 8192];
        let take = (available as usize).min(buffer.len());
        let mut read = 0;
        unsafe {
            ReadFile(
                self.handle,
                Some(&mut buffer[..take]),
                Some(&mut read),
                None,
            )
        }
        .map_err(|_| failure(ProcessFailureReason::Pipe))?;
        if read == 0 || read as usize > take {
            return Err(failure(ProcessFailureReason::Pipe));
        }
        let needed = self.bytes.len() + read as usize;
        if needed > self.bytes.capacity() {
            let target = needed
                .max(self.bytes.capacity().saturating_mul(2))
                .min(self.maximum);
            self.bytes
                .try_reserve_exact(target - self.bytes.len())
                .map_err(|_| failure(ProcessFailureReason::OutputLimit))?;
            if self.bytes.capacity() > self.maximum {
                return Err(failure(ProcessFailureReason::OutputLimit));
            }
        }
        self.bytes.extend_from_slice(&buffer[..read as usize]);
        Ok(true)
    }
}

fn collect(
    child: &mut ChildGuard,
    limits: &ProcessLimits,
    deadline: Instant,
    cancel: &CancellationToken,
) -> Result<ProcessOutput, ProcessFailure> {
    let mut stdout = PipeReader::new(child.stdout.0, limits.stdout_bytes);
    let mut stderr = PipeReader::new(child.stderr.0, limits.stderr_bytes);
    let mut exit = None;
    loop {
        checkpoint(deadline, cancel)?;
        let notifications = child.job.notifications();
        checkpoint(deadline, cancel)?;
        let port_empty = notifications?;
        let out_read = stdout.drain_once();
        let err_read = stderr.drain_once();
        checkpoint(deadline, cancel)?;
        let moved = out_read? | err_read?;
        // SAFETY: the owned handle remains live until explicit stop and Drop.
        match unsafe { WaitForSingleObject(child.process.0, 0) } {
            WAIT_OBJECT_0 if exit.is_none() => {
                let mut code = 0;
                unsafe { GetExitCodeProcess(child.process.0, &mut code) }
                    .map_err(|_| failure(ProcessFailureReason::Wait))?;
                exit = Some(code);
                // The leader can exit while descendants still hold inherited
                // pipes. Stop the entire Job before attempting to confirm EOF.
                child.stop()?;
            }
            WAIT_OBJECT_0 | WAIT_TIMEOUT => {}
            _ => return Err(failure(ProcessFailureReason::Wait)),
        }
        if let Some(exit_code) = exit {
            if stdout.eof && stderr.eof && port_empty {
                // Completion messages can race with leader exit. Observe the
                // queue again after the Job's active-process count reached zero.
                if !child.job.notifications()? {
                    continue;
                }
                let peak_job_bytes = child.job.limits()?.PeakJobMemoryUsed;
                checkpoint(deadline, cancel)?;
                return Ok(ProcessOutput {
                    stdout: stdout.bytes,
                    stderr: stderr.bytes,
                    exit_code,
                    peak_job_bytes,
                    memory_limit_observed: false,
                });
            }
        }
        if !moved {
            std::thread::sleep(Duration::from_millis(5));
        }
    }
}

pub(super) fn run(
    spec: &ProcessSpec,
    limits: &ProcessLimits,
    deadline: Instant,
    cancel: &CancellationToken,
    #[cfg(test)] hook: Option<&dyn Fn(FaultPoint, u32) -> Result<(), ProcessFailure>>,
) -> Result<ProcessOutput, ProcessFailure> {
    checkpoint(deadline, cancel)?;
    if REAP_FAILED.load(Ordering::SeqCst) {
        return Err(failure(ProcessFailureReason::Wait));
    }
    let job = Job::new(limits.memory_bytes)?;
    checkpoint(deadline, cancel)?;
    let mut child = spawn(spec, job)?;
    macro_rules! boundary {
        ($point:ident) => {{
            #[cfg(test)]
            if let Some(hook) = hook {
                hook(FaultPoint::$point, child.pid)?;
            }
            checkpoint(deadline, cancel)?;
        }};
    }
    let outcome = (|| {
        boundary!(AfterCreate);
        boundary!(BeforeAssign);
        // The legacy Assign hook labels bracket verification of creation-time
        // membership. SAFETY: the suspended child and its private Job are live.
        let mut assigned = BOOL(0);
        unsafe { IsProcessInJob(child.process.0, child.job.handle.0, &mut assigned) }
            .map_err(|_| failure(ProcessFailureReason::Assign))?;
        if !assigned.as_bool() {
            // Only an explicit non-membership result selects direct process
            // termination; a query failure retains creation-time Job ownership.
            child.assigned = false;
            return Err(failure(ProcessFailureReason::Assign));
        }
        boundary!(AfterAssign);
        boundary!(BeforeResume);
        // SAFETY: the only primary-thread handle; previous suspend count must
        // be exactly one so assignment always precedes the first execution.
        if unsafe { ResumeThread(child.thread.0) } != 1 {
            return Err(failure(ProcessFailureReason::Resume));
        }
        let output = collect(&mut child, limits, deadline, cancel)?;
        boundary!(BeforeCleanup);
        Ok(output)
    })();
    let stopped = child.stop();
    checkpoint(deadline, cancel)?;
    match outcome {
        Err(error) => Err(error),
        Ok(output) => stopped.map(|()| output),
    }
}
