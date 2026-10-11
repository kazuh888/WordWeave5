//! Independent Windows OS tests for I5b / AH-AC01/09. No real Praat or user data.
//! Build acoustic-test-child explicitly before this unit-test target.
use crate::{
    process::{
        self, FaultPoint, ProcessFailure, ProcessFailureReason, ProcessLimits, ProcessOutput,
        ProcessSpec,
    },
    AnalysisErrorKind,
};
use std::{
    cell::Cell,
    ffi::OsString,
    fs,
    io::{Read, Write},
    path::PathBuf,
    process::{Child, Command, Stdio},
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc, Condvar, Mutex,
    },
    thread,
    time::{Duration, Instant},
};
use tokio_util::sync::CancellationToken;
use windows::Win32::{
    Foundation::{CloseHandle, BOOL, FILETIME, HANDLE, WAIT_FAILED, WAIT_OBJECT_0, WAIT_TIMEOUT},
    Security::SECURITY_ATTRIBUTES,
    System::{
        JobObjects::IsProcessInJob,
        Threading::{
            CreateEventW, GetCurrentProcess, GetProcessHandleCount, GetProcessId, GetProcessTimes,
            OpenProcess, TerminateProcess, WaitForSingleObject, PROCESS_QUERY_LIMITED_INFORMATION,
            PROCESS_SYNCHRONIZE, PROCESS_TERMINATE,
        },
    },
};

#[allow(dead_code)]
#[path = "praat_child.rs"]
mod child_fixture;

#[derive(Clone, Copy, Debug)]
enum AckMode {
    Normal,
    Withheld,
    Mismatch,
}

#[derive(Default)]
struct PublisherGateState {
    arrived: bool,
    released: bool,
}
struct PublisherGate {
    state: Mutex<PublisherGateState>,
    changed: Condvar,
    deadline: Instant,
}
impl PublisherGate {
    fn new(deadline: Instant) -> Self {
        Self {
            state: Mutex::new(PublisherGateState::default()),
            changed: Condvar::new(),
            deadline,
        }
    }
    fn pause_after_publish(&self) -> Result<(), &'static str> {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        state.arrived = true;
        self.changed.notify_all();
        while !state.released {
            let remaining = self.deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                state.released = true;
                self.changed.notify_all();
                return Err("PublisherGateDeadline");
            }
            state = self
                .changed
                .wait_timeout(state, remaining)
                .unwrap_or_else(|e| e.into_inner())
                .0;
        }
        Ok(())
    }
    fn release_after_snapshot(&self) -> Result<(), &'static str> {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        while !state.arrived {
            let remaining = self.deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                state.released = true;
                self.changed.notify_all();
                return Err("PublisherGateDeadline");
            }
            state = self
                .changed
                .wait_timeout(state, remaining)
                .unwrap_or_else(|e| e.into_inner())
                .0;
        }
        state.released = true;
        self.changed.notify_all();
        Ok(())
    }
    fn release(&self) {
        self.state
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .released = true;
        self.changed.notify_all();
    }
}

#[derive(Clone, Debug)]
struct AckEvidence {
    generation: u32,
    root: Identity,
    child: Identity,
    root_live: Option<WaitObservation>,
    child_live: Option<WaitObservation>,
    published: Result<(), &'static str>,
    prepared_at: Option<Instant>,
    completed_at: Option<Instant>,
}

#[derive(Debug)]
struct TreeExitEvidence {
    ack: Option<AckEvidence>,
    notices: Result<(), &'static str>,
    verdict: Result<(), &'static str>,
}

// Serializes ONLY this suite so process-wide handle-count evidence is meaningful.
static SERIAL: Mutex<()> = Mutex::new(());
fn serial() -> std::sync::MutexGuard<'static, ()> {
    SERIAL.lock().unwrap_or_else(|e| e.into_inner())
}
fn executable() -> PathBuf {
    let path = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("acoustic-test-child.exe");
    assert!(
        path.is_file(),
        "fixture missing: build --bin acoustic-test-child first: {path:?}"
    );
    path
}
fn limits() -> ProcessLimits {
    ProcessLimits {
        stdout_bytes: 1_048_576,
        stderr_bytes: 65_536,
        memory_bytes: 536_870_912,
    }
}
fn failure(reason: ProcessFailureReason) -> ProcessFailure {
    ProcessFailure {
        kind: AnalysisErrorKind::ResourceLimit,
        reason,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Identity {
    pid: u32,
    created: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Role {
    Root,
    Descendant,
    ExplicitOwned,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Origin {
    AfterCreate,
    CancelObserver,
    Watchdog,
    ExplicitTrack,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct WaitObservation {
    value: u32,
    error: Option<i32>,
    before: Instant,
    after: Instant,
}
fn observe(raw: isize, milliseconds: u32) -> WaitObservation {
    let before = Instant::now();
    let value = unsafe { WaitForSingleObject(HANDLE(raw), milliseconds) };
    let error = (value == WAIT_FAILED)
        .then(|| std::io::Error::last_os_error().raw_os_error())
        .flatten();
    WaitObservation {
        value: value.0,
        error,
        before,
        after: Instant::now(),
    }
}
fn identity(handle: HANDLE) -> Result<Identity, &'static str> {
    let pid = unsafe { GetProcessId(handle) };
    if pid == 0 {
        return Err("GetProcessIdFailed");
    }
    let mut times = [FILETIME::default(); 4];
    let [created, exited, kernel, user] = &mut times;
    unsafe { GetProcessTimes(handle, created, exited, kernel, user) }
        .map_err(|_| "GetProcessTimesFailed")?;
    Ok(Identity {
        pid,
        created: ((created.dwHighDateTime as u64) << 32) | created.dwLowDateTime as u64,
    })
}

// Each successful open has one owner, including independently opened duplicates.
#[derive(Debug)]
struct OwnedProcessRecord {
    raw: isize,
    generation: u32,
    role: Role,
    origin: Origin,
    notified_pid: u32,
    actual: Identity,
    sequence: usize,
    first_wait: Option<WaitObservation>,
    closes: Arc<AtomicUsize>,
}
impl Drop for OwnedProcessRecord {
    fn drop(&mut self) {
        let result = unsafe { CloseHandle(HANDLE(self.raw)) };
        self.closes.fetch_add(1, Ordering::SeqCst);
        if let Err(error) = result {
            let _ = writeln!(
                std::io::stderr(),
                "D1 CloseFailed sequence={} error={error}",
                self.sequence
            );
        }
    }
}
#[derive(Clone, Copy, Debug)]
struct RecordView {
    raw: isize,
    generation: u32,
    role: Role,
    origin: Origin,
    notified_pid: u32,
    actual: Identity,
    sequence: usize,
}
impl OwnedProcessRecord {
    fn view(&self) -> RecordView {
        RecordView {
            raw: self.raw,
            generation: self.generation,
            role: self.role,
            origin: self.origin,
            notified_pid: self.notified_pid,
            actual: self.actual,
            sequence: self.sequence,
        }
    }
}
#[derive(Debug)]
struct CallEvidence {
    generation: u32,
    supervisor: String,
    reap_before: bool,
    reap_after: bool,
    descendant_ready: bool,
    poison_seen: bool,
    watchdog_rescued: bool,
    exit_code: Option<u32>,
    tree_exit_required: bool,
    tree_exit: Option<TreeExitEvidence>,
    observations: Vec<(RecordView, WaitObservation)>,
}
#[derive(Debug)]
struct RescueEvidence {
    record: RecordView,
    before: WaitObservation,
    terminated: Result<(), String>,
    after: WaitObservation,
}
#[derive(Default)]
struct Registry {
    records: Vec<OwnedProcessRecord>,
    generation: u32,
    root: Option<Identity>,
    sealed: bool,
    tree_generations: Vec<u32>,
    tree_exit_generations: Vec<u32>,
    acknowledgements: Vec<AckEvidence>,
    events: Vec<String>,
    calls: Vec<CallEvidence>,
    rescues: Vec<RescueEvidence>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
enum CleanupUnconfirmed {
    OwnedWait = 1,
    WatchdogPanic = 2,
    SupervisorReap = 4,
    TreeDescendant = 8,
}
struct Diagnostics {
    registry: Mutex<Registry>,
    poison: AtomicBool,
    capacity: AtomicBool,
    invalid: AtomicBool,
    closes: Arc<AtomicUsize>,
    terminations: AtomicUsize,
    open_attempts: AtomicUsize,
    cleanup_unconfirmed: AtomicUsize,
    publisher_gate: Mutex<Option<Arc<PublisherGate>>>,
    ack_completed: Condvar,
}
impl Diagnostics {
    fn new() -> Self {
        Self {
            registry: Mutex::new(Registry::default()),
            poison: AtomicBool::new(false),
            capacity: AtomicBool::new(false),
            invalid: AtomicBool::new(false),
            closes: Arc::new(AtomicUsize::new(0)),
            terminations: AtomicUsize::new(0),
            open_attempts: AtomicUsize::new(0),
            cleanup_unconfirmed: AtomicUsize::new(0),
            publisher_gate: Mutex::new(None),
            ack_completed: Condvar::new(),
        }
    }
    fn lock(&self) -> std::sync::MutexGuard<'_, Registry> {
        self.registry.lock().unwrap_or_else(|e| {
            self.poison.store(true, Ordering::SeqCst);
            e.into_inner()
        })
    }
    fn publisher_gate(&self) -> Option<Arc<PublisherGate>> {
        self.publisher_gate
            .lock()
            .unwrap_or_else(|e| {
                self.poison.store(true, Ordering::SeqCst);
                e.into_inner()
            })
            .clone()
    }
    fn event(&self, event: String) {
        let mut state = self.lock();
        if state.events.contains(&event) {
            return;
        }
        if state.events.len() == 256 {
            self.capacity.store(true, Ordering::SeqCst);
            return;
        }
        state.events.push(event);
    }
    fn begin(&self) -> Result<u32, &'static str> {
        let mut state = self.lock();
        if state.generation == 16 {
            self.capacity.store(true, Ordering::SeqCst);
            return Err("DiagnosticCapacityExceeded");
        }
        state.generation = state
            .generation
            .checked_add(1)
            .ok_or("GenerationOverflow")?;
        state.root = None;
        state.sealed = false;
        Ok(state.generation)
    }
    fn note_tree_created(&self, generation: u32) {
        let mut state = self.lock();
        if !state.tree_generations.contains(&generation) {
            if state.tree_generations.len() == 16 {
                self.capacity.store(true, Ordering::SeqCst);
                self.mark_cleanup(CleanupUnconfirmed::TreeDescendant);
            } else {
                state.tree_generations.push(generation);
            }
        }
    }
    fn mark_cleanup(&self, reason: CleanupUnconfirmed) {
        self.cleanup_unconfirmed
            .fetch_or(reason as usize, Ordering::SeqCst);
    }
    fn note_tree_exit_required(&self, generation: u32) {
        let mut state = self.lock();
        if !state.tree_exit_generations.contains(&generation) {
            if state.tree_exit_generations.len() == 16 {
                self.capacity.store(true, Ordering::SeqCst);
            } else {
                state.tree_exit_generations.push(generation);
            }
        }
    }
    fn publish_tree_exit_ack(
        &self,
        dir: &std::path::Path,
        generation: u32,
        root: Identity,
        child: Identity,
        mode: AckMode,
    ) -> Result<(), &'static str> {
        let mut evidence = AckEvidence {
            generation,
            root,
            child,
            root_live: None,
            child_live: None,
            published: Err("AckNotPrepared"),
            prepared_at: None,
            completed_at: None,
        };
        let preparation = (|| {
            let (root_record, child_record) = {
                let state = self.lock();
                Self::observer_admission(
                    &state,
                    Origin::Watchdog,
                    Some((generation, root, child)),
                )?;
                if !state.tree_exit_generations.contains(&generation) {
                    return Err("NotTreeExitGeneration");
                }
                let root_record = state
                    .records
                    .iter()
                    .find(|r| {
                        r.generation == generation && r.role == Role::Root && r.actual == root
                    })
                    .ok_or("AckRootNotRegistered")?
                    .view();
                let child_record = state
                    .records
                    .iter()
                    .find(|r| {
                        r.generation == generation
                            && r.role == Role::Descendant
                            && r.origin == Origin::Watchdog
                            && r.actual == child
                    })
                    .ok_or("AckChildNotRegistered")?
                    .view();
                (root_record, child_record)
            };
            evidence.root_live = Some(observe(root_record.raw, 0));
            evidence.child_live = Some(observe(child_record.raw, 0));
            if evidence.root_live.is_none_or(|w| w.value != WAIT_TIMEOUT.0)
                || evidence
                    .child_live
                    .is_none_or(|w| w.value != WAIT_TIMEOUT.0)
            {
                return Err("AckOwnedProcessNotLive");
            }
            let mut state = self.lock();
            Self::observer_admission(&state, Origin::Watchdog, Some((generation, root, child)))?;
            if state
                .acknowledgements
                .iter()
                .any(|ack| ack.generation == generation)
            {
                return Err("DuplicateAckGeneration");
            }
            if state.acknowledgements.len() == 16 {
                self.capacity.store(true, Ordering::SeqCst);
                return Err("DiagnosticCapacityExceeded");
            }
            evidence.prepared_at = Some(Instant::now());
            evidence.published = Err("AckPending");
            state.acknowledgements.push(evidence);
            Ok::<(), &'static str>(())
        })();
        if let Err(reason) = preparation {
            self.event(format!(
                "AckPreparationFailed generation={generation} {reason}"
            ));
            return Err(reason);
        }
        // Prepared is visible before publication, but no file I/O holds the lock.
        let result = (|| {
            if matches!(mode, AckMode::Withheld) {
                return Err("AckWithheld");
            }
            let wire_child = if matches!(mode, AckMode::Mismatch) {
                (child.pid, child.created + 1)
            } else {
                (child.pid, child.created)
            };
            child_fixture::atomic_notice(
                &child_fixture::tree_exit_path(dir, (root.pid, root.created), "ack"),
                &child_fixture::handshake_bytes(
                    "WWTREEEXITACK1",
                    (root.pid, root.created),
                    wire_child,
                ),
            )?;
            if let Some(gate) = self.publisher_gate() {
                gate.pause_after_publish()?;
            }
            Ok(())
        })();
        self.complete_ack(generation, root, child, result)
    }
    fn complete_ack(
        &self,
        generation: u32,
        root: Identity,
        child: Identity,
        result: Result<(), &'static str>,
    ) -> Result<(), &'static str> {
        let mut state = self.lock();
        let slot = state
            .acknowledgements
            .iter_mut()
            .find(|ack| {
                ack.generation == generation
                    && ack.root == root
                    && ack.child == child
                    && ack.prepared_at.is_some()
            })
            .ok_or("MissingPreparedAck")?;
        if slot.completed_at.is_some() {
            return Err("AckAlreadyComplete");
        }
        slot.published = result;
        slot.completed_at = Some(Instant::now());
        drop(state);
        self.ack_completed.notify_all();
        result
    }
    fn wait_ack_complete(
        &self,
        generation: u32,
        deadline: Instant,
    ) -> Result<AckEvidence, &'static str> {
        let mut state = self.lock();
        loop {
            if self.poison.load(Ordering::SeqCst) {
                return Err("AckRegistryPoison");
            }
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return Err("AckCompletionDeadline");
            }
            let slot = state
                .acknowledgements
                .iter()
                .find(|ack| ack.generation == generation && ack.prepared_at.is_some())
                .ok_or("MissingPreparedAck")?;
            if slot.completed_at.is_some() {
                return Ok(slot.clone());
            }
            state = match self.ack_completed.wait_timeout(state, remaining) {
                Ok((state, _)) => state,
                Err(error) => {
                    self.poison.store(true, Ordering::SeqCst);
                    drop(error.into_inner());
                    return Err("AckRegistryPoison");
                }
            };
        }
    }
    fn tree_exit_evidence(
        &self,
        dir: &std::path::Path,
        generation: u32,
        observations: &[(RecordView, WaitObservation)],
        exit_code: Option<u32>,
        deadline: Instant,
    ) -> TreeExitEvidence {
        let completion = self.wait_ack_complete(generation, deadline);
        let ack = completion.as_ref().ok().cloned();
        let notices = (|| {
            let ack = completion.as_ref().map_err(|reason| *reason)?;
            let root = (ack.root.pid, ack.root.created);
            let child = (ack.child.pid, ack.child.created);
            child_fixture::validate_handshake(
                &bounded_read(&child_fixture::tree_exit_path(dir, root, "ack"))?,
                "WWTREEEXITACK1",
                root,
                child,
            )?;
            child_fixture::validate_handshake(
                &bounded_read(&child_fixture::tree_exit_path(dir, root, "ready"))?,
                "WWTREEEXITREADY1",
                root,
                child,
            )
        })();
        let verdict = if observations
            .iter()
            .any(|(_, observed)| observed.value != WAIT_OBJECT_0.0)
        {
            Err("OwnedFirstWaitFailed")
        } else {
            match completion {
                Err(reason) => Err(reason),
                Ok(_) => {
                    tree_exit_oracle(generation, observations, ack.as_ref(), notices, exit_code)
                }
            }
        };
        TreeExitEvidence {
            ack,
            notices,
            verdict,
        }
    }
    fn record_cleanup_wait(&self, observed: u32) {
        if observed != WAIT_OBJECT_0.0 {
            self.invalid.store(true, Ordering::SeqCst);
            self.mark_cleanup(CleanupUnconfirmed::OwnedWait);
        }
    }
    fn check_tree_coverage(&self) {
        let state = self.lock();
        if state.tree_generations.iter().any(|generation| {
            !state
                .records
                .iter()
                .any(|record| record.generation == *generation && record.role == Role::Descendant)
        }) {
            self.mark_cleanup(CleanupUnconfirmed::TreeDescendant);
        }
    }
    fn cleanup_reasons(&self) -> Vec<CleanupUnconfirmed> {
        let bits = self.cleanup_unconfirmed.load(Ordering::SeqCst);
        [
            CleanupUnconfirmed::OwnedWait,
            CleanupUnconfirmed::WatchdogPanic,
            CleanupUnconfirmed::SupervisorReap,
            CleanupUnconfirmed::TreeDescendant,
        ]
        .into_iter()
        .filter(|reason| bits & *reason as usize != 0)
        .collect()
    }
    fn snapshot(&self) -> Vec<RecordView> {
        self.lock()
            .records
            .iter()
            .map(OwnedProcessRecord::view)
            .collect()
    }
    fn track(
        &self,
        pid: u32,
        origin: Origin,
        expected: Option<(u32, Identity, Identity)>,
    ) -> Result<HANDLE, &'static str> {
        let admission = Self::observer_admission(&self.lock(), origin, expected);
        if let Err(reason) = admission {
            self.event(format!("{origin:?} {reason}"));
            return Err(reason);
        }
        let record = self.open_owned(pid, origin)?;
        self.register_owned(record, expected)
    }
    fn observer_admission(
        state: &Registry,
        origin: Origin,
        expected: Option<(u32, Identity, Identity)>,
    ) -> Result<(), &'static str> {
        if !matches!(origin, Origin::Watchdog | Origin::CancelObserver) {
            return Ok(());
        }
        let Some((generation, root, _)) = expected else {
            return Err("MissingObserverIdentity");
        };
        if state.generation != generation || state.root != Some(root) {
            return Err("StaleGeneration");
        }
        if state.sealed {
            return Err("RegistrationSealed");
        }
        Ok(())
    }
    fn open_owned(&self, pid: u32, origin: Origin) -> Result<OwnedProcessRecord, &'static str> {
        self.open_attempts.fetch_add(1, Ordering::SeqCst);
        let handle = unsafe {
            OpenProcess(
                PROCESS_SYNCHRONIZE | PROCESS_TERMINATE | PROCESS_QUERY_LIMITED_INFORMATION,
                false,
                pid,
            )
        }
        .map_err(|_| "OpenOwnedProcessFailed")?;
        let actual = match identity(handle) {
            Ok(value) => value,
            Err(error) => {
                let _ = unsafe { CloseHandle(handle) };
                return Err(error);
            }
        };
        // An unverified handle is closed only; no termination is permitted.
        if actual.pid != pid {
            let _ = unsafe { CloseHandle(handle) };
            return Err("HandleIdentityMismatch");
        }
        Ok(OwnedProcessRecord {
            raw: handle.0,
            generation: 0,
            role: Role::ExplicitOwned,
            origin,
            notified_pid: pid,
            actual,
            sequence: 0,
            first_wait: None,
            closes: self.closes.clone(),
        })
    }
    fn register_owned(
        &self,
        mut record: OwnedProcessRecord,
        expected: Option<(u32, Identity, Identity)>,
    ) -> Result<HANDLE, &'static str> {
        let origin = record.origin;
        let pid = record.notified_pid;
        let handle = HANDLE(record.raw);
        if expected.is_some_and(|(_, _, child)| child != record.actual) {
            return Err("HandleIdentityMismatch");
        }
        // Admission is rechecked with insertion under one lock. Rejected record
        // drops outside the lock: exactly one CloseHandle, never TerminateProcess.
        let mut state = self.lock();
        if let Err(reason) = Self::observer_admission(&state, origin, expected) {
            drop(state);
            self.event(format!("{origin:?} {reason} opened_pid={pid}"));
            return Err(reason);
        }
        if state.records.len() == 16 {
            self.capacity.store(true, Ordering::SeqCst);
            drop(state);
            return Err("DiagnosticCapacityExceeded");
        }
        match origin {
            Origin::AfterCreate => {
                state.root = Some(record.actual);
                record.role = Role::Root;
                record.generation = state.generation;
            }
            Origin::CancelObserver | Origin::Watchdog => {
                record.role = Role::Descendant;
                record.generation = state.generation;
            }
            Origin::ExplicitTrack if state.root == Some(record.actual) => {
                record.role = Role::Root;
                record.generation = state.generation;
            }
            Origin::ExplicitTrack => {}
        }
        record.sequence = state.records.len() + 1;
        state.records.push(record);
        drop(state);
        self.event(format!("Registered {origin:?} pid={pid}"));
        Ok(handle)
    }
    fn descendant(&self, dir: &std::path::Path, origin: Origin) -> bool {
        let (generation, root) = {
            let state = self.lock();
            (state.generation, state.root)
        };
        let Some(root) = root else {
            return false;
        };
        let decoded = bounded_read(&dir.join("grandchild.identity")).and_then(|frame| {
            bounded_read(&dir.join("grandchild.pid"))
                .and_then(|pid| decode_notification(&frame, &pid, root))
        });
        match decoded
            .and_then(|child| self.track(child.pid, origin, Some((generation, root, child))))
        {
            Ok(_) => true,
            Err(reason) => {
                self.event(format!("generation={generation} {origin:?} {reason}"));
                false
            }
        }
    }
    fn first_waits(&self) -> Vec<(RecordView, WaitObservation)> {
        let snapshot: Vec<_> = {
            let mut state = self.lock();
            state.sealed = true;
            state.records.iter().map(OwnedProcessRecord::view).collect()
        };
        // No logging, identity queries or rescue between these zero-time observations.
        let observations: Vec<_> = snapshot
            .into_iter()
            .map(|record| (record, observe(record.raw, 0)))
            .collect();
        let mut state = self.lock();
        for (record, observed) in &observations {
            if let Some(owned) = state
                .records
                .iter_mut()
                .find(|owned| owned.sequence == record.sequence)
            {
                owned.first_wait.get_or_insert(*observed);
            }
        }
        observations
    }
    fn valid(&self) -> bool {
        !self.poison.load(Ordering::SeqCst)
            && !self.capacity.load(Ordering::SeqCst)
            && !self.invalid.load(Ordering::SeqCst)
            && self.lock().records.iter().all(|record| {
                record
                    .first_wait
                    .is_some_and(|w| w.value == WAIT_OBJECT_0.0)
            })
    }
    fn rescue(&self, record: RecordView, code: u32) {
        let before = observe(record.raw, 0);
        if before.value == WAIT_OBJECT_0.0 {
            return;
        }
        self.terminations.fetch_add(1, Ordering::SeqCst); // Latch before termination.
        let terminated =
            unsafe { TerminateProcess(HANDLE(record.raw), code) }.map_err(|e| e.to_string());
        let after = observe(record.raw, 2000);
        self.record_cleanup_wait(after.value);
        let evidence = RescueEvidence {
            record,
            before,
            terminated,
            after,
        };
        let _ = writeln!(std::io::stderr(), "D1 rescue {evidence:?}");
        let mut state = self.lock();
        if state.rescues.len() == 32 {
            self.capacity.store(true, Ordering::SeqCst);
        } else {
            state.rescues.push(evidence);
        }
    }
    fn report(&self) {
        let (events, calls, records) = {
            let state = self.lock();
            (
                format!("{:?}", state.events),
                format!("{:?}", state.calls),
                format!("{:?}", state.records),
            )
        };
        let _ = writeln!(std::io::stderr(), "D1 events={events} calls={calls} records={records} poison={} capacity={} invalid={} expected_wait=0", self.poison.load(Ordering::SeqCst), self.capacity.load(Ordering::SeqCst), self.invalid.load(Ordering::SeqCst));
    }
}
fn tree_exit_oracle(
    generation: u32,
    observations: &[(RecordView, WaitObservation)],
    ack: Option<&AckEvidence>,
    notices: Result<(), &'static str>,
    exit_code: Option<u32>,
) -> Result<(), &'static str> {
    let root = observations
        .iter()
        .find(|(r, _)| r.generation == generation && r.role == Role::Root)
        .ok_or("MissingRetainedRoot")?;
    let child = observations
        .iter()
        .find(|(r, _)| {
            r.generation == generation && r.role == Role::Descendant && r.origin == Origin::Watchdog
        })
        .ok_or("MissingRetainedDescendant")?;
    if observations
        .iter()
        .any(|(_, wait)| wait.value != WAIT_OBJECT_0.0)
    {
        return Err("OwnedFirstWaitFailed");
    }
    let ack = ack.ok_or("MissingAckEvidence")?;
    if ack.prepared_at.is_none() || ack.completed_at.is_none() {
        return Err("MissingPreparedComplete");
    }
    if ack.generation != generation || ack.root != root.0.actual || ack.child != child.0.actual {
        return Err("AckRecordedIdentityMismatch");
    }
    ack.published?;
    if ack.root_live.is_none_or(|w| w.value != WAIT_TIMEOUT.0)
        || ack.child_live.is_none_or(|w| w.value != WAIT_TIMEOUT.0)
    {
        return Err("MissingPreExitLiveEvidence");
    }
    notices?;
    if exit_code != Some(0) {
        return Err("TreeExitRootFailed");
    }
    Ok(())
}
fn bounded_read(path: &std::path::Path) -> Result<Vec<u8>, &'static str> {
    let file = fs::File::open(path).map_err(|_| "NotificationUnavailable")?;
    let mut bytes = Vec::new();
    file.take(257)
        .read_to_end(&mut bytes)
        .map_err(|_| "NotificationReadFailed")?;
    if bytes.len() > 256 {
        return Err("NotificationOversize");
    }
    Ok(bytes)
}
fn decode_notification(
    frame: &[u8],
    raw_pid: &[u8],
    expected_root: Identity,
) -> Result<Identity, &'static str> {
    if frame.len() > 256 || !frame.ends_with(b"\n") {
        return Err("IncompleteIdentity");
    }
    let text = std::str::from_utf8(&frame[..frame.len() - 1]).map_err(|_| "InvalidIdentity")?;
    let fields: Vec<_> = text.split('\t').collect();
    if fields.len() != 5 || fields[0] != "WWCHILD1" {
        return Err("InvalidIdentity");
    }
    let number = |value: &str| -> Result<u64, &'static str> {
        if value.is_empty() || !value.bytes().all(|b| b.is_ascii_digit()) || value.starts_with('0')
        {
            return Err("InvalidDecimal");
        }
        value.parse().map_err(|_| "InvalidDecimal")
    };
    let root = Identity {
        pid: u32::try_from(number(fields[1])?).map_err(|_| "InvalidPid")?,
        created: number(fields[2])?,
    };
    let child = Identity {
        pid: u32::try_from(number(fields[3])?).map_err(|_| "InvalidPid")?,
        created: number(fields[4])?,
    };
    if root != expected_root {
        return Err("RootIdentityMismatch");
    }
    if raw_pid != child.pid.to_string().as_bytes() {
        return Err("IncompletePid");
    }
    Ok(child)
}

struct Fixture {
    root: tempfile::TempDir,
    diagnostics: Arc<Diagnostics>,
    done: Arc<AtomicBool>,
    rescued: Arc<AtomicBool>,
    watchdog: Option<thread::JoinHandle<()>>,
}
impl Fixture {
    fn new() -> Self {
        Self::with_ack_mode(AckMode::Normal)
    }
    fn with_ack_mode(ack_mode: AckMode) -> Self {
        let root = tempfile::Builder::new()
            .prefix("ww-praat-process-日本語 ")
            .tempdir()
            .unwrap();
        let diagnostics = Arc::new(Diagnostics::new());
        let done = Arc::new(AtomicBool::new(false));
        let rescued = Arc::new(AtomicBool::new(false));
        let (h, d, r) = (diagnostics.clone(), done.clone(), rescued.clone());
        let descendant_dir = root.path().to_path_buf();
        let watchdog = thread::spawn(move || {
            let until = Instant::now() + Duration::from_secs(8);
            let mut descendant_generation = 0;
            while !d.load(Ordering::SeqCst) {
                let generation = h.lock().generation;
                if descendant_generation != generation
                    && h.descendant(&descendant_dir, Origin::Watchdog)
                {
                    descendant_generation = generation;
                    let identities = {
                        let state = h.lock();
                        if state.tree_exit_generations.contains(&generation) {
                            state.root.zip(
                                state
                                    .records
                                    .iter()
                                    .find(|record| {
                                        record.generation == generation
                                            && record.role == Role::Descendant
                                            && record.origin == Origin::Watchdog
                                    })
                                    .map(|record| record.actual),
                            )
                        } else {
                            None
                        }
                    };
                    if let Some((root, child)) = identities {
                        let _ = h.publish_tree_exit_ack(
                            &descendant_dir,
                            generation,
                            root,
                            child,
                            ack_mode,
                        );
                    }
                }
                if Instant::now() >= until {
                    r.store(true, Ordering::SeqCst);
                    for record in h.snapshot() {
                        h.rescue(record, 239);
                    }
                    return;
                }
                thread::sleep(Duration::from_millis(10));
            }
        });
        Self {
            root,
            diagnostics,
            done,
            rescued,
            watchdog: Some(watchdog),
        }
    }
    fn spec(&self, mode: &str, args: &[&str]) -> ProcessSpec {
        ProcessSpec {
            executable: executable(),
            cwd: self.root.path().to_path_buf(),
            args: std::iter::once(OsString::from(mode))
                .chain(std::iter::once(self.root.path().as_os_str().to_owned()))
                .chain(args.iter().map(OsString::from))
                .collect(),
        }
    }
    fn track(&self, pid: u32) -> HANDLE {
        self.diagnostics
            .track(pid, Origin::ExplicitTrack, None)
            .expect("open own child independently")
    }
    fn run(
        &self,
        spec: &ProcessSpec,
        cap: &ProcessLimits,
        deadline: Instant,
        cancel: &CancellationToken,
        hook: &dyn Fn(FaultPoint, u32) -> Result<(), ProcessFailure>,
    ) -> Result<ProcessOutput, ProcessFailure> {
        let generation = self
            .diagnostics
            .begin()
            .expect("DiagnosticCapacityExceeded");
        let reap_before = process::reap_unconfirmed();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            process::supervise_with_hook(spec, cap, deadline, cancel, &|point, pid| {
                if point == FaultPoint::AfterCreate {
                    if spec
                        .args
                        .first()
                        .is_some_and(|mode| mode == "tree" || mode == "tree-exit")
                    {
                        self.diagnostics.note_tree_created(generation);
                    }
                    if spec.args.first().is_some_and(|mode| mode == "tree-exit") {
                        self.diagnostics.note_tree_exit_required(generation);
                    }
                    self.diagnostics
                        .track(pid, Origin::AfterCreate, None)
                        .expect("track suspended owned root");
                }
                hook(point, pid)
            })
        }));
        let reap_after = process::reap_unconfirmed();
        let observations = self.diagnostics.first_waits();
        // These are return-time facts. Completion waiting cannot rewrite them.
        let poison_seen = self.diagnostics.poison.load(Ordering::SeqCst);
        let watchdog_rescued = self.rescued.load(Ordering::SeqCst);
        let supervisor = match &result {
            Ok(Ok(_)) => "Ok".into(),
            Ok(Err(error)) => format!("{error:?}"),
            Err(_) => "SupervisorPanic".into(),
        };
        if reap_before || reap_after {
            self.diagnostics
                .mark_cleanup(CleanupUnconfirmed::SupervisorReap);
        }
        let descendant_ready = observations.iter().any(|(r, _)| {
            r.generation == generation
                && r.role == Role::Descendant
                && r.origin == Origin::CancelObserver
        });
        let exit_code = match &result {
            Ok(Ok(output)) => Some(output.exit_code),
            _ => None,
        };
        let tree_exit_required = self
            .diagnostics
            .lock()
            .tree_exit_generations
            .contains(&generation);
        if tree_exit_required {
            if let Some(gate) = self.diagnostics.publisher_gate() {
                if gate.release_after_snapshot().is_err() {
                    self.diagnostics.invalid.store(true, Ordering::SeqCst);
                }
            }
        }
        let tree_exit = tree_exit_required.then(|| {
            self.diagnostics.tree_exit_evidence(
                &self.root.path(),
                generation,
                &observations,
                exit_code,
                deadline,
            )
        });
        let tree_exit_verdict = tree_exit.as_ref().map(|evidence| evidence.verdict);
        let evidence = CallEvidence {
            generation,
            supervisor,
            reap_before,
            reap_after,
            descendant_ready,
            poison_seen,
            watchdog_rescued,
            exit_code,
            tree_exit_required,
            tree_exit,
            observations,
        };
        let all_signaled = evidence
            .observations
            .iter()
            .all(|(_, w)| w.value == WAIT_OBJECT_0.0);
        self.diagnostics.lock().calls.push(evidence);
        self.diagnostics.report();
        let result = match result {
            Ok(value) => value,
            Err(panic) => {
                self.diagnostics.invalid.store(true, Ordering::SeqCst);
                std::panic::resume_unwind(panic);
            }
        };
        if let Some(verdict) = tree_exit_verdict {
            assert!(verdict.is_ok(), "tree-exit fixture invalid: {verdict:?}");
        }
        if spec.args.first().is_some_and(|mode| mode == "tree") {
            let state = self.diagnostics.lock();
            let has_root = state
                .records
                .iter()
                .any(|r| r.generation == generation && r.role == Role::Root);
            drop(state);
            assert!(
                has_root && descendant_ready,
                "tree fixture requires registered root and cancel observer descendant"
            );
        }
        assert!(
            !reap_before && !reap_after,
            "reap_unconfirmed is not a PASS"
        );
        assert!(
            self.diagnostics.valid(),
            "diagnostic failure or first wait failed"
        );
        assert!(
            !self.rescued.load(Ordering::SeqCst),
            "supervisor required the 8-second external safety kill"
        );
        assert!(
            all_signaled,
            "supervisor returned while an owned process remained alive"
        );
        result
    }
    fn normal(
        &self,
        mode: &str,
        args: &[&str],
        cap: &ProcessLimits,
    ) -> Result<ProcessOutput, ProcessFailure> {
        self.run(
            &self.spec(mode, args),
            cap,
            Instant::now() + Duration::from_secs(3),
            &CancellationToken::new(),
            &|_, _| Ok(()),
        )
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        self.done.store(true, Ordering::SeqCst);
        if let Some(gate) = self.diagnostics.publisher_gate() {
            gate.release();
        }
        if let Some(watchdog) = self.watchdog.take() {
            if watchdog.join().is_err() {
                self.diagnostics.invalid.store(true, Ordering::SeqCst);
                self.diagnostics.event("WatchdogPanic".into());
                self.diagnostics
                    .mark_cleanup(CleanupUnconfirmed::WatchdogPanic);
            }
        }
        if process::reap_unconfirmed() {
            self.diagnostics
                .mark_cleanup(CleanupUnconfirmed::SupervisorReap);
        }
        self.diagnostics.check_tree_coverage();
        let records = { std::mem::take(&mut self.diagnostics.lock().records) };
        for record in records {
            self.diagnostics.rescue(record.view(), 238);
            drop(record);
        }
        let reasons = self.diagnostics.cleanup_reasons();
        if !reasons.is_empty() {
            self.root.disable_cleanup(true);
            let _ = writeln!(
                std::io::stderr(),
                "D1 cleanup=unconfirmed retained_temp={:?} reasons={reasons:?}",
                self.root.path()
            );
        }
        self.diagnostics.report();
    }
}

#[test]
fn argv_round_trips_empty_unicode_spaces_quotes_and_trailing_backslashes() {
    let _lock = serial();
    let fixture = Fixture::new();
    let args = [
        "",
        "plain",
        "日本語 空白",
        "a\"b",
        "a b\\",
        "\\\\",
        "\"quoted\"",
        "末尾 \\",
    ];
    let output = fixture.normal("echo", &args, &limits()).unwrap();
    let expected: Vec<u8> = args
        .iter()
        .flat_map(|arg| {
            (arg.len() as u32)
                .to_le_bytes()
                .into_iter()
                .chain(arg.as_bytes().iter().copied())
        })
        .collect();
    assert_eq!(output.stdout, expected);
    assert!(output.stderr.is_empty());
    assert_eq!(output.exit_code, 0);
}

#[test]
fn public_supervise_entry_runs_the_same_bounded_child_contract() {
    let _lock = serial();
    let fixture = Fixture::new();
    let output = process::supervise(
        &fixture.spec("echo", &["entry"]),
        &limits(),
        Instant::now() + Duration::from_secs(3),
        &CancellationToken::new(),
    )
    .unwrap();
    assert_eq!(output.stdout, b"\x05\0\0\0entry");
    assert_eq!(output.exit_code, 0);
}

#[test]
fn executable_path_with_unicode_and_spaces_is_launched_directly() {
    let _lock = serial();
    let fixture = Fixture::new();
    let mut spec = fixture.spec("echo", &["direct"]);
    spec.executable = fixture.root.path().join("合成 子 process.exe");
    fs::copy(executable(), &spec.executable).unwrap();
    let output = fixture
        .run(
            &spec,
            &limits(),
            Instant::now() + Duration::from_secs(3),
            &CancellationToken::new(),
            &|_, _| Ok(()),
        )
        .unwrap();
    assert_eq!(output.stdout, b"\x06\0\0\0direct");
    assert_eq!(output.exit_code, 0);
}

#[test]
fn child_does_not_execute_until_assignment_and_resume_are_complete() {
    let _lock = serial();
    let fixture = Fixture::new();
    let seen = std::cell::RefCell::new(Vec::new());
    let output = fixture
        .run(
            &fixture.spec("echo", &[]),
            &limits(),
            Instant::now() + Duration::from_secs(3),
            &CancellationToken::new(),
            &|point, pid| {
                if point != FaultPoint::BeforeCleanup {
                    seen.borrow_mut().push(point);
                    thread::sleep(Duration::from_millis(40));
                    assert!(
                        !fixture.root.path().join("started").exists(),
                        "child executed at {point:?}"
                    );
                    if matches!(
                        point,
                        FaultPoint::AfterCreate
                            | FaultPoint::BeforeAssign
                            | FaultPoint::AfterAssign
                            | FaultPoint::BeforeResume
                    ) {
                        let handle = fixture.track(pid);
                        let mut in_job = BOOL(0);
                        unsafe { IsProcessInJob(handle, HANDLE::default(), &mut in_job) }.unwrap();
                        assert!(
                            in_job.as_bool(),
                            "child must belong to a Job from creation, at {point:?}"
                        );
                    }
                }
                Ok(())
            },
        )
        .unwrap();
    assert_eq!(output.exit_code, 0);
    assert_eq!(
        *seen.borrow(),
        [
            FaultPoint::AfterCreate,
            FaultPoint::BeforeAssign,
            FaultPoint::AfterAssign,
            FaultPoint::BeforeResume
        ]
    );
    assert!(fixture.root.path().join("started").exists());
}

#[test]
fn every_pre_resume_fault_reaps_suspended_child_without_running_user_code() {
    let _lock = serial();
    for stop in [
        FaultPoint::AfterCreate,
        FaultPoint::BeforeAssign,
        FaultPoint::AfterAssign,
        FaultPoint::BeforeResume,
    ] {
        let fixture = Fixture::new();
        let injected = failure(if stop == FaultPoint::BeforeResume {
            ProcessFailureReason::Resume
        } else {
            ProcessFailureReason::Assign
        });
        let hit = Cell::new(false);
        let error = fixture
            .run(
                &fixture.spec("hang", &[]),
                &limits(),
                Instant::now() + Duration::from_secs(3),
                &CancellationToken::new(),
                &|point, _| {
                    if point == stop {
                        hit.set(true);
                        Err(injected.clone())
                    } else {
                        Ok(())
                    }
                },
            )
            .unwrap_err();
        assert!(hit.get(), "fault hook not reached: {stop:?}");
        assert_eq!(error, injected);
        assert!(!fixture.root.path().join("started").exists());
    }
}

#[test]
fn unwind_with_assigned_live_child_reaps_owned_processes() {
    let _lock = serial();
    let fixture = Fixture::new();
    let hit = Cell::new(false);
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        fixture.run(
            &fixture.spec("echo", &[]),
            &limits(),
            Instant::now() + Duration::from_secs(3),
            &CancellationToken::new(),
            &|point, _| {
                if point == FaultPoint::AfterAssign {
                    hit.set(true);
                    panic!("intentional unwind while assigned child is alive");
                }
                Ok(())
            },
        )
    }));
    assert!(
        hit.get(),
        "a stub panic is not an exercised live-child hook"
    );
    assert!(result.is_err());
    for record in fixture.diagnostics.snapshot() {
        assert_eq!(
            unsafe { WaitForSingleObject(HANDLE(record.raw), 0) },
            WAIT_OBJECT_0
        );
    }
}

#[test]
fn synthetic_fixture_is_executable_and_generates_independent_expected_bytes() {
    let _lock = serial();
    let root = tempfile::tempdir().unwrap();
    let output = Command::new(executable())
        .arg("output")
        .arg(root.path())
        .args(["5", "3"])
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert!(output.status.success());
    assert_eq!(output.stdout, b"OOOOO");
    assert_eq!(output.stderr, b"EEE");
    assert!(root.path().join("started").is_file());
}

#[test]
fn cancellation_is_not_starved_by_simultaneous_pipe_floods() {
    let _lock = serial();
    let fixture = Fixture::new();
    let cancel = CancellationToken::new();
    let running = AtomicBool::new(false);
    let cap = ProcessLimits {
        stdout_bytes: 64 * 1024 * 1024,
        stderr_bytes: 64 * 1024 * 1024,
        ..limits()
    };
    let result = thread::scope(|scope| {
        scope.spawn(|| {
            let until = Instant::now() + Duration::from_secs(2);
            while Instant::now() < until {
                if fixture.root.path().join("started").exists() {
                    // Give both fixture writers an opportunity to fill the OS pipes.
                    thread::sleep(Duration::from_millis(50));
                    running.store(true, Ordering::SeqCst);
                    break;
                }
                thread::sleep(Duration::from_millis(5));
            }
            cancel.cancel();
        });
        fixture.run(
            &fixture.spec("flood", &[]),
            &cap,
            Instant::now() + Duration::from_secs(4),
            &cancel,
            &|_, _| Ok(()),
        )
    });
    assert!(
        running.load(Ordering::SeqCst),
        "fixture did not reach running state"
    );
    assert_eq!(result.unwrap_err().kind, AnalysisErrorKind::Cancelled);
}

#[test]
fn both_pipes_accept_below_and_exact_caps_without_truncation() {
    let _lock = serial();
    for (stdout, stderr) in [(32_767, 16_383), (32_768, 16_384)] {
        let fixture = Fixture::new();
        let cap = ProcessLimits {
            stdout_bytes: 32_768,
            stderr_bytes: 16_384,
            ..limits()
        };
        let output = fixture
            .normal("output", &[&stdout.to_string(), &stderr.to_string()], &cap)
            .unwrap();
        assert_eq!(output.stdout, vec![b'O'; stdout]);
        assert_eq!(output.stderr, vec![b'E'; stderr]);
        assert!(output.stdout.capacity() <= cap.stdout_bytes);
        assert!(output.stderr.capacity() <= cap.stderr_bytes);
    }
}

#[test]
fn one_byte_over_either_pipe_cap_is_resource_failure_even_with_exit_zero() {
    let _lock = serial();
    for (stdout, stderr) in [(32_769, 0), (0, 16_385)] {
        let fixture = Fixture::new();
        let cap = ProcessLimits {
            stdout_bytes: 32_768,
            stderr_bytes: 16_384,
            ..limits()
        };
        let error = fixture
            .normal("output", &[&stdout.to_string(), &stderr.to_string()], &cap)
            .unwrap_err();
        assert_eq!(error, failure(ProcessFailureReason::OutputLimit));
    }
}

#[test]
fn stderr_limit_is_observed_under_continuous_stdout_pressure() {
    let _lock = serial();
    let fixture = Fixture::new();
    let cap = ProcessLimits {
        stdout_bytes: 64 * 1024 * 1024,
        stderr_bytes: 1,
        ..limits()
    };
    let error = fixture.normal("flood", &[], &cap).unwrap_err();
    assert_eq!(error, failure(ProcessFailureReason::OutputLimit));
}

#[test]
fn pre_cancel_wins_over_expired_deadline_and_creates_no_process() {
    let _lock = serial();
    let fixture = Fixture::new();
    let cancel = CancellationToken::new();
    cancel.cancel();
    let error = fixture
        .run(
            &fixture.spec("hang", &[]),
            &limits(),
            Instant::now() - Duration::from_secs(1),
            &cancel,
            &|_, _| panic!("must reject before child creation"),
        )
        .unwrap_err();
    assert_eq!(error.kind, AnalysisErrorKind::Cancelled);
    assert!(!fixture.root.path().join("started").exists());
}

#[test]
fn expired_deadline_creates_no_process() {
    let _lock = serial();
    let fixture = Fixture::new();
    let error = fixture
        .run(
            &fixture.spec("hang", &[]),
            &limits(),
            Instant::now() - Duration::from_secs(1),
            &CancellationToken::new(),
            &|_, _| panic!("must reject before child creation"),
        )
        .unwrap_err();
    assert_eq!(error.kind, AnalysisErrorKind::TimedOut);
}

#[test]
fn cancellation_at_pre_resume_boundary_prevents_execution_and_allows_retry() {
    let _lock = serial();
    let fixture = Fixture::new();
    let cancel = CancellationToken::new();
    let error = fixture
        .run(
            &fixture.spec("hang", &[]),
            &limits(),
            Instant::now() + Duration::from_secs(3),
            &cancel,
            &|point, _| {
                if point == FaultPoint::BeforeResume {
                    cancel.cancel();
                }
                Ok(())
            },
        )
        .unwrap_err();
    assert_eq!(error.kind, AnalysisErrorKind::Cancelled);
    assert!(!fixture.root.path().join("started").exists());
    assert_eq!(
        fixture
            .normal("echo", &["retry"], &limits())
            .unwrap()
            .exit_code,
        0
    );
}

#[test]
fn running_timeout_stops_and_reaps_the_owned_process() {
    let _lock = serial();
    let fixture = Fixture::new();
    let began = Instant::now();
    let error = fixture
        .run(
            &fixture.spec("hang", &[]),
            &limits(),
            began + Duration::from_millis(350),
            &CancellationToken::new(),
            &|_, _| Ok(()),
        )
        .unwrap_err();
    assert_eq!(error.kind, AnalysisErrorKind::TimedOut);
    assert!(
        fixture.root.path().join("started").exists(),
        "must exercise a running child"
    );
    assert!(
        began.elapsed() < Duration::from_secs(3),
        "stop-to-reap fixture budget exceeded"
    );
}

struct Sibling(Child);
impl Drop for Sibling {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

#[test]
fn cancel_kills_child_and_descendant_preserves_sibling_and_existing_data() {
    let _lock = serial();
    let fixture = Fixture::new();
    let sibling_root = tempfile::tempdir().unwrap();
    let mut sibling = Sibling(
        Command::new(executable())
            .arg("hang")
            .arg(sibling_root.path())
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap(),
    );
    fs::write(
        fixture.root.path().join("preserved-original"),
        b"synthetic immutable bytes",
    )
    .unwrap();
    let cancel = CancellationToken::new();
    let descendant_ready = AtomicBool::new(false);
    let result = thread::scope(|scope| {
        scope.spawn(|| {
            let until = Instant::now() + Duration::from_secs(2);
            while Instant::now() < until {
                if fixture
                    .diagnostics
                    .descendant(fixture.root.path(), Origin::CancelObserver)
                {
                    descendant_ready.store(true, Ordering::SeqCst);
                    break;
                }
                thread::sleep(Duration::from_millis(5));
            }
            cancel.cancel();
        });
        fixture.run(
            &fixture.spec("tree", &[]),
            &limits(),
            Instant::now() + Duration::from_secs(4),
            &cancel,
            &|_, _| Ok(()),
        )
    });
    assert!(
        descendant_ready.load(Ordering::SeqCst),
        "must observe actual descendant before cancellation"
    );
    assert_eq!(result.unwrap_err().kind, AnalysisErrorKind::Cancelled);
    assert!(
        sibling.0.try_wait().unwrap().is_none(),
        "unrelated sibling was terminated"
    );
    assert_eq!(
        fs::read(fixture.root.path().join("preserved-original")).unwrap(),
        b"synthetic immutable bytes"
    );
}

#[test]
fn child_exit_with_descendant_holding_pipes_does_not_hang_eof_collection() {
    let _lock = serial();
    let fixture = Fixture::new();
    let output = fixture.normal("tree-exit", &[], &limits()).unwrap();
    assert_eq!(output.exit_code, 0);
    let root = fixture.diagnostics.lock().root.unwrap();
    assert!(
        fixture
            .root
            .path()
            .join(format!("tree-exit-{}-{}.ready", root.pid, root.created))
            .is_file(),
        "A1 requires identity-bound ready before accepting root success"
    );
    let records = fixture.diagnostics.snapshot();
    assert!(records.iter().any(|record| record.role == Role::Root));
    assert!(
        records
            .iter()
            .any(|record| record.role == Role::Descendant && record.origin == Origin::Watchdog),
        "missing retained descendant cannot be optional success"
    );
    let diagnostic = fixture.diagnostics.clone();
    let temp = fixture.root.path().to_path_buf();
    drop(fixture);
    assert!(diagnostic.cleanup_reasons().is_empty());
    assert_eq!(diagnostic.terminations.load(Ordering::SeqCst), 0);
    assert!(
        !temp.exists(),
        "confirmed tree-exit cleanup must remove its own temp"
    );
}

#[test]
fn inheritable_parent_event_is_excluded_from_child_handle_list() {
    let _lock = serial();
    let fixture = Fixture::new();
    let attributes = SECURITY_ATTRIBUTES {
        nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
        lpSecurityDescriptor: std::ptr::null_mut(),
        bInheritHandle: BOOL(1),
    };
    let event = unsafe { CreateEventW(Some(&attributes), true, false, None) }.unwrap();
    struct Event(HANDLE);
    impl Drop for Event {
        fn drop(&mut self) {
            unsafe {
                let _ = CloseHandle(self.0);
            }
        }
    }
    let event = Event(event);
    let output = fixture
        .normal("event", &[&event.0 .0.to_string()], &limits())
        .unwrap();
    assert_eq!(output.stdout, b"blocked");
    assert_eq!(unsafe { WaitForSingleObject(event.0, 0) }, WAIT_TIMEOUT);
}

#[test]
fn memory_commit_over_private_cap_is_denied_and_never_succeeds_as_analysis() {
    let _lock = serial();
    let fixture = Fixture::new();
    let cap = ProcessLimits {
        memory_bytes: 32 * 1024 * 1024,
        ..limits()
    };
    let result = fixture.normal("memory", &[], &cap);
    assert!(
        fixture.root.path().join("started").exists(),
        "cap must permit fixture startup"
    );
    assert!(
        fixture.root.path().join("allocation-attempted").exists(),
        "must exercise actual allocation, not startup failure"
    );
    assert!(!fixture.root.path().join("allocation-allowed").exists());
    match result {
        // The memory notification may reach the supervisor before the denied
        // allocator returns to fixture code. Both outcomes are in the contract.
        Err(error) => assert_eq!(error, failure(ProcessFailureReason::MemoryLimit)),
        Ok(output) => {
            assert!(fixture.root.path().join("allocation-denied").exists());
            assert_eq!(
                crate::praat_protocol::classify_output(&output)
                    .unwrap_err()
                    .kind,
                AnalysisErrorKind::ResourceLimit
            );
        }
    }
}

#[test]
fn nonzero_exit_is_preserved_for_the_adapter_classifier() {
    let _lock = serial();
    let fixture = Fixture::new();
    let output = fixture.normal("exit", &["37"], &limits()).unwrap();
    assert_eq!(output.exit_code, 37);
    assert_eq!(
        crate::praat_protocol::classify_output(&output).unwrap_err(),
        failure(ProcessFailureReason::UnclassifiedAbnormalExit)
    );
}

#[test]
fn missing_executable_and_embedded_nul_fail_without_running_child() {
    let _lock = serial();
    let fixture = Fixture::new();
    let mut spec = fixture.spec("echo", &[]);
    spec.executable = fixture.root.path().join("absent.exe");
    let error = fixture
        .run(
            &spec,
            &limits(),
            Instant::now() + Duration::from_secs(3),
            &CancellationToken::new(),
            &|_, _| panic!("no child expected"),
        )
        .unwrap_err();
    assert_eq!(error.kind, AnalysisErrorKind::BackendUnavailable);
    let mut spec = fixture.spec("echo", &[]);
    spec.args.push(OsString::from("nul\0argument"));
    assert!(fixture
        .run(
            &spec,
            &limits(),
            Instant::now() + Duration::from_secs(3),
            &CancellationToken::new(),
            &|_, _| panic!("NUL must reject before creation")
        )
        .is_err());
    assert!(!fixture.root.path().join("started").exists());
}

fn handle_count() -> u32 {
    let mut count = 0;
    unsafe { GetProcessHandleCount(GetCurrentProcess(), &mut count) }.unwrap();
    count
}

#[test]
fn repeated_success_and_injected_assignment_failure_do_not_leak_handles() {
    let _lock = serial();
    // Warm up fixture/runtime before measuring the process handle ledger.
    {
        let f = Fixture::new();
        f.normal("echo", &[], &limits()).unwrap();
    }
    let before = handle_count();
    for _ in 0..12 {
        let f = Fixture::new();
        f.normal("echo", &[], &limits()).unwrap();
        let error = f
            .run(
                &f.spec("hang", &[]),
                &limits(),
                Instant::now() + Duration::from_secs(3),
                &CancellationToken::new(),
                &|point, _| {
                    if point == FaultPoint::BeforeAssign {
                        Err(failure(ProcessFailureReason::Assign))
                    } else {
                        Ok(())
                    }
                },
            )
            .unwrap_err();
        assert_eq!(error, failure(ProcessFailureReason::Assign));
    }
    let after = handle_count();
    assert!(
        after <= before + 2,
        "handle growth across 24 calls: {before} -> {after}"
    );
}

const M1_SUPERVISOR_ROOT: &str = "WW_ACOUSTIC_M1_SUPERVISOR_ROOT";

fn d1_live_child(dir: &std::path::Path) -> Sibling {
    use std::os::windows::process::CommandExt;
    Sibling(
        Command::new(executable())
            .arg("hang")
            .arg(dir)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .creation_flags(windows::Win32::System::Threading::CREATE_NO_WINDOW.0)
            .spawn()
            .unwrap(),
    )
}

#[test]
fn a1_r2_completion_absence_deadline_and_poison_are_failures() {
    let diagnostics = Diagnostics::new();
    let now = Instant::now();
    assert_eq!(
        diagnostics
            .wait_ack_complete(1, now + Duration::from_secs(1))
            .unwrap_err(),
        "MissingPreparedAck"
    );
    let root = Identity {
        pid: 11,
        created: 111,
    };
    let child = Identity {
        pid: 22,
        created: 222,
    };
    diagnostics.lock().acknowledgements.push(AckEvidence {
        generation: 1,
        root,
        child,
        root_live: None,
        child_live: None,
        published: Err("AckPending"),
        prepared_at: Some(now),
        completed_at: None,
    });
    let expired = Instant::now();
    assert_eq!(
        diagnostics.wait_ack_complete(1, expired).unwrap_err(),
        "AckCompletionDeadline"
    );
    assert!(diagnostics.lock().acknowledgements[0]
        .completed_at
        .is_none());
    let finite_deadline = Instant::now() + Duration::from_millis(20);
    assert_eq!(
        diagnostics
            .wait_ack_complete(1, finite_deadline)
            .unwrap_err(),
        "AckCompletionDeadline"
    );
    assert!(Instant::now() >= finite_deadline);
    diagnostics.complete_ack(1, root, child, Ok(())).unwrap();
    assert_eq!(
        diagnostics.wait_ack_complete(1, expired).unwrap_err(),
        "AckCompletionDeadline",
        "a late completion cannot restart the same deadline"
    );
    let poisoned = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _guard = diagnostics.registry.lock().unwrap();
        panic!("intentional ACK registry poison");
    }));
    assert!(poisoned.is_err());
    assert_eq!(
        diagnostics
            .wait_ack_complete(1, Instant::now() + Duration::from_secs(1))
            .unwrap_err(),
        "AckRegistryPoison"
    );
}

#[test]
fn a1_r2_old_completion_cannot_fill_or_overwrite_another_generation() {
    let diagnostics = Diagnostics::new();
    let root = Identity {
        pid: 11,
        created: 111,
    };
    let child = Identity {
        pid: 22,
        created: 222,
    };
    for generation in 1..=2 {
        assert_eq!(diagnostics.begin(), Ok(generation));
        diagnostics.lock().acknowledgements.push(AckEvidence {
            generation,
            root,
            child,
            root_live: None,
            child_live: None,
            published: Err("AckPending"),
            prepared_at: Some(Instant::now()),
            completed_at: None,
        });
    }
    diagnostics.complete_ack(1, root, child, Ok(())).unwrap();
    assert!(diagnostics.lock().acknowledgements[1]
        .completed_at
        .is_none());
    assert_eq!(
        diagnostics.complete_ack(3, root, child, Ok(())),
        Err("MissingPreparedAck")
    );
    assert_eq!(
        diagnostics.complete_ack(2, root, child, Err("HandshakeRenameFailed")),
        Err("HandshakeRenameFailed")
    );
    assert_eq!(
        diagnostics.complete_ack(2, root, child, Ok(())),
        Err("AckAlreadyComplete")
    );
    assert_eq!(
        diagnostics
            .wait_ack_complete(2, Instant::now() + Duration::from_secs(1))
            .unwrap()
            .published,
        Err("HandshakeRenameFailed")
    );
    assert_eq!(
        diagnostics
            .wait_ack_complete(1, Instant::now() + Duration::from_secs(1))
            .unwrap()
            .published,
        Ok(())
    );
}

#[test]
fn a1_r2_publisher_gate_releases_on_deadline_and_fixture_unwind() {
    let expired = PublisherGate::new(Instant::now());
    assert_eq!(expired.pause_after_publish(), Err("PublisherGateDeadline"));
    assert!(expired.state.lock().unwrap().released);
    let absent = PublisherGate::new(Instant::now());
    assert_eq!(
        absent.release_after_snapshot(),
        Err("PublisherGateDeadline")
    );
    assert!(absent.state.lock().unwrap().released);
    let gate = Arc::new(PublisherGate::new(Instant::now() + Duration::from_secs(3)));
    let panicked = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let fixture = Fixture::new();
        *fixture.diagnostics.publisher_gate.lock().unwrap() = Some(gate.clone());
        panic!("intentional fixture unwind before publisher release");
    }));
    assert!(panicked.is_err());
    assert!(gate.state.lock().unwrap().released);
    assert_eq!(gate.pause_after_publish(), Ok(()));
}

#[test]
fn a1_r2_publication_io_failure_completes_as_failure_and_keeps_first_timeout() {
    let _lock = serial();
    let fixture = Fixture::new();
    let root_child = d1_live_child(fixture.root.path());
    let descendant_child = d1_live_child(fixture.root.path());
    let generation = fixture.diagnostics.begin().unwrap();
    fixture.diagnostics.note_tree_created(generation);
    fixture.diagnostics.note_tree_exit_required(generation);
    fixture
        .diagnostics
        .track(root_child.0.id(), Origin::AfterCreate, None)
        .unwrap();
    let root = fixture.diagnostics.lock().root.unwrap();
    use std::os::windows::io::AsRawHandle;
    let child = identity(HANDLE(descendant_child.0.as_raw_handle() as isize)).unwrap();
    fixture
        .diagnostics
        .track(child.pid, Origin::Watchdog, Some((generation, root, child)))
        .unwrap();
    fs::create_dir(child_fixture::tree_exit_path(
        fixture.root.path(),
        (root.pid, root.created),
        "ack",
    ))
    .unwrap();
    assert_eq!(
        fixture.diagnostics.publish_tree_exit_ack(
            fixture.root.path(),
            generation,
            root,
            child,
            AckMode::Normal
        ),
        Err("HandshakeAlreadyExists")
    );
    let ack = fixture
        .diagnostics
        .wait_ack_complete(generation, Instant::now() + Duration::from_secs(1))
        .unwrap();
    assert_eq!(ack.published, Err("HandshakeAlreadyExists"));
    assert!(ack.prepared_at.is_some() && ack.completed_at.is_some());
    let first = fixture.diagnostics.first_waits();
    assert!(first.iter().all(|(_, wait)| wait.value == WAIT_TIMEOUT.0));
    for (record, _) in &first {
        fixture.diagnostics.rescue(*record, 238);
    }
    assert!(fixture
        .diagnostics
        .first_waits()
        .iter()
        .all(|(_, wait)| wait.value == WAIT_OBJECT_0.0));
    assert!(fixture
        .diagnostics
        .lock()
        .records
        .iter()
        .all(|record| record
            .first_wait
            .is_some_and(|wait| wait.value == WAIT_TIMEOUT.0)));
    let after = fixture.diagnostics.tree_exit_evidence(
        fixture.root.path(),
        generation,
        &first,
        Some(0),
        Instant::now() + Duration::from_secs(1),
    );
    assert_eq!(after.verdict, Err("OwnedFirstWaitFailed"));
    assert!(!fixture.diagnostics.valid());
}

#[test]
fn a1_r2_published_ack_finalize_delay_succeeds_after_first_snapshot() {
    let _lock = serial();
    let fixture = Fixture::new();
    let deadline = Instant::now() + Duration::from_secs(3);
    let gate = Arc::new(PublisherGate::new(deadline));
    *fixture.diagnostics.publisher_gate.lock().unwrap() = Some(gate.clone());
    let output = fixture
        .run(
            &fixture.spec("tree-exit", &[]),
            &limits(),
            deadline,
            &CancellationToken::new(),
            &|_, _| Ok(()),
        )
        .unwrap();
    assert_eq!(output.exit_code, 0);
    assert!(gate.state.lock().unwrap().arrived);
    assert!(gate.state.lock().unwrap().released);
    {
        let state = fixture.diagnostics.lock();
        let call = &state.calls[0];
        assert!(call
            .observations
            .iter()
            .all(|(_, wait)| wait.value == WAIT_OBJECT_0.0));
        assert_eq!(call.tree_exit.as_ref().unwrap().verdict, Ok(()));
        let ack = call.tree_exit.as_ref().unwrap().ack.as_ref().unwrap();
        let first_start = call
            .observations
            .iter()
            .map(|(_, wait)| wait.before)
            .min()
            .unwrap();
        let first_end = call
            .observations
            .iter()
            .map(|(_, wait)| wait.after)
            .max()
            .unwrap();
        assert!(ack.prepared_at.unwrap() <= first_start);
        assert!(
            ack.completed_at.unwrap() >= first_end,
            "the seam must delay Complete until all first observations are latched"
        );
        assert!(!call.reap_before && !call.reap_after && !call.watchdog_rescued);
    }
    assert!(!fixture.watchdog.as_ref().unwrap().is_finished());
    fixture.normal("echo", &[], &limits()).unwrap();
    assert_eq!(fixture.diagnostics.lock().generation, 2);
    let diagnostics = fixture.diagnostics.clone();
    let path = fixture.root.path().to_path_buf();
    drop(fixture);
    assert!(diagnostics.cleanup_reasons().is_empty());
    assert_eq!(diagnostics.terminations.load(Ordering::SeqCst), 0);
    assert!(!path.exists());
}

#[test]
fn a1_handwritten_ack_and_ready_require_exact_identity_and_complete_bytes() {
    let _lock = serial();
    let root = (1234, 5678);
    let child = (9876, 5432);
    let dir = tempfile::tempdir().unwrap();
    for (header, valid) in [
        (
            "WWTREEEXITACK1",
            b"WWTREEEXITACK1\t1234\t5678\t9876\t5432\n".as_slice(),
        ),
        (
            "WWTREEEXITREADY1",
            b"WWTREEEXITREADY1\t1234\t5678\t9876\t5432\n".as_slice(),
        ),
    ] {
        assert_eq!(
            child_fixture::validate_handshake(valid, header, root, child),
            Ok(())
        );
        for bad in [
            b"".as_slice(),
            &valid[..valid.len() - 1],
            &valid[..10],
            &[b'x'; 257],
        ] {
            assert!(child_fixture::validate_handshake(bad, header, root, child).is_err());
        }
        let mut extra = valid.to_vec();
        extra.push(b'\n');
        assert!(child_fixture::validate_handshake(&extra, header, root, child).is_err());
        for (wrong_root, wrong_child) in [
            ((1235, 5678), child),
            ((1234, 5679), child),
            (root, (9877, 5432)),
            (root, (9876, 5433)),
        ] {
            assert!(
                child_fixture::validate_handshake(valid, header, wrong_root, wrong_child).is_err()
            );
        }
    }
    // Exercise the real child consumer against completed, malformed on-disk files.
    let ack = child_fixture::tree_exit_path(dir.path(), root, "ack");
    for bad in [
        b"WWTREEEXITACK1\t1234\t5678\t9876\t5433\n".as_slice(),
        b"WWTREEEXITACK1\t1234",
        b"WWTREEEXITACK1\t1234\t5678\t9876\t5432\nextra",
        &[b'x'; 257],
    ] {
        fs::write(&ack, bad).unwrap();
        let error = child_fixture::wait_tree_exit_ack(
            dir.path(),
            root,
            child,
            Instant::now() + Duration::from_secs(2),
        )
        .unwrap_err();
        assert_ne!(
            error, "AckDeadline",
            "complete malformed ACK must be rejected without waiting for deadline"
        );
    }
    fs::write(&ack, b"WWTREEEXITACK1\t1234\t5678\t9876\t5432\n").unwrap();
    assert_eq!(
        child_fixture::wait_tree_exit_ack(
            dir.path(),
            root,
            child,
            Instant::now() + Duration::from_secs(2)
        ),
        Ok(())
    );
}

#[test]
fn a1_missing_expired_and_late_ack_cannot_restart_the_deadline() {
    let _lock = serial();
    let dir = tempfile::tempdir().unwrap();
    let deadline = Instant::now();
    assert_eq!(
        child_fixture::wait_tree_exit_ack(dir.path(), (1234, 5678), (9876, 5432), deadline),
        Err("AckDeadline")
    );
    fs::write(
        child_fixture::tree_exit_path(dir.path(), (1234, 5678), "ack"),
        b"WWTREEEXITACK1\t1234\t5678\t9876\t5432\n",
    )
    .unwrap();
    assert_eq!(
        child_fixture::wait_tree_exit_ack(dir.path(), (1234, 5678), (9876, 5432), deadline),
        Err("AckDeadline")
    );
}

#[test]
fn a1_ack_publisher_requires_registered_live_current_unsealed_handles() {
    let _lock = serial();
    let fixture = Fixture::new();
    let root_child = d1_live_child(fixture.root.path());
    let descendant_child = d1_live_child(fixture.root.path());
    let generation = fixture.diagnostics.begin().unwrap();
    fixture.diagnostics.note_tree_created(generation);
    fixture.diagnostics.note_tree_exit_required(generation);
    fixture
        .diagnostics
        .track(root_child.0.id(), Origin::AfterCreate, None)
        .unwrap();
    let root = fixture.diagnostics.lock().root.unwrap();
    use std::os::windows::io::AsRawHandle;
    let descendant = identity(HANDLE(descendant_child.0.as_raw_handle() as isize)).unwrap();
    assert_eq!(
        fixture.diagnostics.publish_tree_exit_ack(
            fixture.root.path(),
            generation,
            root,
            descendant,
            AckMode::Normal
        ),
        Err("AckChildNotRegistered")
    );
    let path = child_fixture::tree_exit_path(fixture.root.path(), (root.pid, root.created), "ack");
    assert!(!path.exists());
    fixture
        .diagnostics
        .track(
            descendant.pid,
            Origin::Watchdog,
            Some((generation, root, descendant)),
        )
        .unwrap();
    assert_eq!(
        fixture.diagnostics.publish_tree_exit_ack(
            fixture.root.path(),
            generation,
            root,
            descendant,
            AckMode::Normal
        ),
        Ok(())
    );
    let published = fs::read(&path).unwrap();
    assert_eq!(
        published,
        format!(
            "WWTREEEXITACK1\t{}\t{}\t{}\t{}\n",
            root.pid, root.created, descendant.pid, descendant.created
        )
        .as_bytes()
    );
    assert_eq!(
        fixture.diagnostics.publish_tree_exit_ack(
            fixture.root.path(),
            generation,
            root,
            descendant,
            AckMode::Normal
        ),
        Err("DuplicateAckGeneration")
    );
    assert_eq!(fs::read(&path).unwrap(), published);
    let first = fixture.diagnostics.first_waits();
    assert_eq!(
        fixture.diagnostics.publish_tree_exit_ack(
            fixture.root.path(),
            generation,
            root,
            descendant,
            AckMode::Normal
        ),
        Err("RegistrationSealed")
    );
    assert_eq!(fs::read(&path).unwrap(), published);
    let next = fixture.diagnostics.begin().unwrap();
    fixture
        .diagnostics
        .track(root.pid, Origin::AfterCreate, None)
        .unwrap();
    assert_eq!(next, generation + 1);
    assert_eq!(
        fixture.diagnostics.publish_tree_exit_ack(
            fixture.root.path(),
            generation,
            root,
            descendant,
            AckMode::Normal
        ),
        Err("StaleGeneration")
    );
    assert_eq!(
        fixture.diagnostics.lock().records[0].first_wait,
        Some(first[0].1)
    );
    assert_eq!(fixture.diagnostics.terminations.load(Ordering::SeqCst), 0);
}

#[test]
fn a1_child_cannot_publish_ready_after_its_descendant_exits() {
    let _lock = serial();
    let dir = tempfile::tempdir().unwrap();
    let mut child = Command::new(executable())
        .arg("echo")
        .arg(dir.path())
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    child.wait().unwrap();
    use std::os::windows::io::AsRawHandle;
    let root = identity(unsafe { GetCurrentProcess() }).unwrap();
    let descendant = identity(HANDLE(child.as_raw_handle() as isize)).unwrap();
    fs::write(
        child_fixture::tree_exit_path(dir.path(), (root.pid, root.created), "ack"),
        format!(
            "WWTREEEXITACK1\t{}\t{}\t{}\t{}\n",
            root.pid, root.created, descendant.pid, descendant.created
        ),
    )
    .unwrap();
    assert_eq!(
        child_fixture::complete_tree_exit(
            dir.path(),
            &child,
            Instant::now() + Duration::from_secs(2)
        ),
        Err("DescendantNotLiveAtReady")
    );
    assert!(!child_fixture::tree_exit_path(dir.path(), (root.pid, root.created), "ready").exists());
}

#[test]
fn a1_missing_d1_descendant_evidence_is_rejected_even_with_late_complete_notices() {
    let _lock = serial();
    // Hand-written missing-record shape from D1 tree-exit; raw=0 is never used
    // for an OS call. This is an oracle fixture, not another process launch.
    let root = Identity {
        pid: 34468,
        created: 134358573102582092,
    };
    let child = Identity {
        pid: 11712,
        created: 134358573102809610,
    };
    let now = Instant::now();
    let observed = WaitObservation {
        value: WAIT_OBJECT_0.0,
        error: None,
        before: now,
        after: now,
    };
    let root_record = RecordView {
        raw: 0,
        generation: 1,
        role: Role::Root,
        origin: Origin::AfterCreate,
        notified_pid: root.pid,
        actual: root,
        sequence: 1,
    };
    let live = WaitObservation {
        value: WAIT_TIMEOUT.0,
        ..observed
    };
    let ack = AckEvidence {
        generation: 1,
        root,
        child,
        root_live: Some(live),
        child_live: Some(live),
        published: Ok(()),
        prepared_at: Some(now),
        completed_at: Some(now),
    };
    let missing = vec![(root_record, observed)];
    assert_eq!(
        tree_exit_oracle(1, &missing, None, Err("MissingAckEvidence"), Some(0)),
        Err("MissingRetainedDescendant")
    );
    assert_eq!(
        tree_exit_oracle(1, &missing, Some(&ack), Ok(()), Some(0)),
        Err("MissingRetainedDescendant")
    );
    let child_record = RecordView {
        raw: 0,
        role: Role::Descendant,
        origin: Origin::Watchdog,
        notified_pid: child.pid,
        actual: child,
        sequence: 2,
        ..root_record
    };
    let first_failed = vec![(root_record, observed), (child_record, live)];
    assert_eq!(
        tree_exit_oracle(1, &first_failed, Some(&ack), Ok(()), Some(0)),
        Err("OwnedFirstWaitFailed")
    );
    let complete = vec![(root_record, observed), (child_record, observed)];
    assert_eq!(
        tree_exit_oracle(1, &complete, Some(&ack), Ok(()), Some(0)),
        Ok(())
    );
    let prior_failure = AckEvidence {
        published: Err("AckWithheld"),
        ..ack
    };
    assert_eq!(
        tree_exit_oracle(1, &complete, Some(&prior_failure), Ok(()), Some(0)),
        Err("AckWithheld")
    );
    eprintln!("A1 deterministic missing-evidence oracle: FAIL MissingRetainedDescendant; late notices cannot change it");
}

#[test]
fn a1_withheld_and_mismatched_ack_fail_without_ready_or_cleanup_loss() {
    let _lock = serial();
    for mode in [AckMode::Withheld, AckMode::Mismatch] {
        let fixture = Fixture::with_ack_mode(mode);
        let started = Instant::now();
        let rejected = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            fixture.normal("tree-exit", &[], &limits())
        }));
        assert!(
            rejected.is_err(),
            "invalid ACK must not create a successful fixture: {mode:?}"
        );
        assert!(
            started.elapsed() < Duration::from_secs(3),
            "fixed 3-second budget exceeded"
        );
        {
            let state = fixture.diagnostics.lock();
            let call = state.calls.last().unwrap();
            assert_eq!(call.exit_code, Some(241));
            assert!(call.tree_exit_required);
            assert!(call.tree_exit.as_ref().unwrap().verdict.is_err());
            assert!(
                call.observations
                    .iter()
                    .all(|(_, w)| w.value == WAIT_OBJECT_0.0),
                "identified first wait failure requires investigation"
            );
            assert!(!call.reap_before && !call.reap_after && !call.watchdog_rescued);
            let root = state.root.unwrap();
            assert!(!child_fixture::tree_exit_path(
                fixture.root.path(),
                (root.pid, root.created),
                "ready"
            )
            .exists());
        }
        let diagnostics = fixture.diagnostics.clone();
        let path = fixture.root.path().to_path_buf();
        if matches!(mode, AckMode::Withheld) {
            let (generation, observations, root, child, exit_code) = {
                let state = diagnostics.lock();
                let call = &state.calls[0];
                let ack = &state.acknowledgements[0];
                (
                    call.generation,
                    call.observations.clone(),
                    ack.root,
                    ack.child,
                    call.exit_code,
                )
            };
            // This ACK is truly later than the observed failed root exit.
            child_fixture::atomic_notice(
                &child_fixture::tree_exit_path(&path, (root.pid, root.created), "ack"),
                &child_fixture::handshake_bytes(
                    "WWTREEEXITACK1",
                    (root.pid, root.created),
                    (child.pid, child.created),
                ),
            )
            .unwrap();
            let late = diagnostics.tree_exit_evidence(
                &path,
                generation,
                &observations,
                exit_code,
                started + Duration::from_secs(3),
            );
            assert_eq!(late.verdict, Err("AckWithheld"));
            assert!(
                !child_fixture::tree_exit_path(&path, (root.pid, root.created), "ready").exists()
            );
        }
        drop(fixture);
        assert!(diagnostics.cleanup_reasons().is_empty());
        assert_eq!(diagnostics.terminations.load(Ordering::SeqCst), 0);
        assert!(!path.exists());
    }
}

#[test]
fn d1_r_open_inflight_at_seal_is_closed_once_without_register_or_terminate() {
    let _lock = serial();
    let fixture = Fixture::new();
    let mut child = d1_live_child(fixture.root.path());
    let generation = fixture.diagnostics.begin().unwrap();
    fixture
        .diagnostics
        .track(child.0.id(), Origin::AfterCreate, None)
        .unwrap();
    let actual = fixture.diagnostics.lock().root.unwrap();
    let pending = fixture
        .diagnostics
        .open_owned(child.0.id(), Origin::Watchdog)
        .unwrap();
    let first = fixture.diagnostics.first_waits(); // deterministic seal boundary
    assert_eq!(
        fixture
            .diagnostics
            .register_owned(pending, Some((generation, actual, actual)))
            .unwrap_err(),
        "RegistrationSealed"
    );
    assert_eq!(fixture.diagnostics.snapshot().len(), 1);
    assert_eq!(
        fixture.diagnostics.lock().records[0].first_wait,
        Some(first[0].1)
    );
    assert_eq!(fixture.diagnostics.closes.load(Ordering::SeqCst), 1);
    assert_eq!(fixture.diagnostics.terminations.load(Ordering::SeqCst), 0);
    assert!(child.0.try_wait().unwrap().is_none());
    let diagnostics = fixture.diagnostics.clone();
    drop(fixture);
    assert_eq!(diagnostics.closes.load(Ordering::SeqCst), 2);
    assert!(child.0.try_wait().unwrap().is_some());
}

#[test]
fn d1_r_new_generation_rejects_old_inflight_and_accepts_new_root() {
    let _lock = serial();
    let fixture = Fixture::new();
    let mut child = d1_live_child(fixture.root.path());
    let old_generation = fixture.diagnostics.begin().unwrap();
    fixture
        .diagnostics
        .track(child.0.id(), Origin::AfterCreate, None)
        .unwrap();
    let actual = fixture.diagnostics.lock().root.unwrap();
    let pending = fixture
        .diagnostics
        .open_owned(child.0.id(), Origin::Watchdog)
        .unwrap();
    let first = fixture.diagnostics.first_waits();
    let next_generation = fixture.diagnostics.begin().unwrap();
    assert_eq!(next_generation, old_generation + 1);
    fixture
        .diagnostics
        .track(child.0.id(), Origin::AfterCreate, None)
        .unwrap();
    assert!(!fixture.diagnostics.lock().sealed);
    assert_eq!(
        fixture
            .diagnostics
            .register_owned(pending, Some((old_generation, actual, actual)))
            .unwrap_err(),
        "StaleGeneration"
    );
    let snapshot = fixture.diagnostics.snapshot();
    assert_eq!(snapshot.len(), 2);
    assert_eq!(snapshot[1].generation, next_generation);
    assert_eq!(
        fixture.diagnostics.lock().records[0].first_wait,
        Some(first[0].1)
    );
    assert_eq!(fixture.diagnostics.terminations.load(Ordering::SeqCst), 0);
    let diagnostics = fixture.diagnostics.clone();
    drop(fixture);
    assert!(child.0.try_wait().unwrap().is_some());
    assert_eq!(diagnostics.closes.load(Ordering::SeqCst), 3);
}

#[test]
fn d1_r_cleanup_failure_observations_retain_temp_at_policy_boundary() {
    let _lock = serial();
    // Inject only the unsafe OS-failure observation / coverage boundary.
    // No production reap flag, permissions or real process is altered.
    for case in 0..4 {
        let fixture = Fixture::new();
        let path = fixture.root.path().to_path_buf();
        fs::write(path.join("retained-evidence"), b"owned synthetic evidence").unwrap();
        let expected = match case {
            0 => {
                fixture.diagnostics.record_cleanup_wait(WAIT_TIMEOUT.0);
                CleanupUnconfirmed::OwnedWait
            }
            1 => {
                fixture.diagnostics.record_cleanup_wait(WAIT_FAILED.0);
                CleanupUnconfirmed::OwnedWait
            }
            2 => {
                fixture
                    .diagnostics
                    .mark_cleanup(CleanupUnconfirmed::SupervisorReap);
                CleanupUnconfirmed::SupervisorReap
            }
            _ => {
                let generation = fixture.diagnostics.begin().unwrap();
                fixture.diagnostics.note_tree_created(generation);
                CleanupUnconfirmed::TreeDescendant
            }
        };
        let diagnostics = fixture.diagnostics.clone();
        drop(fixture);
        let retained = path.is_dir();
        let bytes = fs::read(path.join("retained-evidence"));
        if retained {
            fs::remove_dir_all(&path).unwrap();
        } // only this test's known temp
        assert!(retained, "case {case}: unconfirmed cleanup was discarded");
        assert_eq!(bytes.unwrap(), b"owned synthetic evidence");
        assert!(diagnostics.cleanup_reasons().contains(&expected));
        assert_eq!(diagnostics.open_attempts.load(Ordering::SeqCst), 0);
        assert_eq!(diagnostics.terminations.load(Ordering::SeqCst), 0);
    }
}

#[test]
fn d1_r_watchdog_panic_retains_owned_temp_and_bytes() {
    let _lock = serial();
    let mut fixture = Fixture::new();
    fixture.done.store(true, Ordering::SeqCst);
    fixture.watchdog.take().unwrap().join().unwrap();
    let path = fixture.root.path().to_path_buf();
    fs::write(
        path.join("preserved-diagnostic"),
        b"synthetic diagnostic bytes",
    )
    .unwrap();
    fixture.watchdog = Some(thread::spawn(|| panic!("intentional D1 watchdog panic")));
    drop(fixture);
    let retained = path.is_dir();
    let bytes = fs::read(path.join("preserved-diagnostic"));
    // This exact directory was created by this test; no process was spawned.
    if retained {
        fs::remove_dir_all(&path).unwrap();
    }
    assert!(
        retained,
        "unconfirmed watchdog cleanup must retain the isolated temp"
    );
    assert_eq!(bytes.unwrap(), b"synthetic diagnostic bytes");
}

#[test]
fn d1_r_sealed_observer_makes_no_new_open_and_preserves_snapshot() {
    let _lock = serial();
    let fixture = Fixture::new();
    let child = d1_live_child(fixture.root.path());
    let generation = fixture.diagnostics.begin().unwrap();
    fixture
        .diagnostics
        .track(child.0.id(), Origin::AfterCreate, None)
        .unwrap();
    let actual = fixture.diagnostics.lock().root.unwrap();
    let first = fixture.diagnostics.first_waits();
    let opens = fixture.diagnostics.open_attempts.load(Ordering::SeqCst);
    let result = fixture.diagnostics.track(
        actual.pid,
        Origin::Watchdog,
        Some((generation, actual, actual)),
    );
    assert_eq!(result.unwrap_err(), "RegistrationSealed");
    assert_eq!(
        fixture.diagnostics.open_attempts.load(Ordering::SeqCst),
        opens
    );
    assert_eq!(fixture.diagnostics.snapshot().len(), first.len());
    assert_eq!(
        fixture.diagnostics.lock().records[0].first_wait,
        Some(first[0].1)
    );
    assert_eq!(fixture.diagnostics.terminations.load(Ordering::SeqCst), 0);
}

#[test]
fn d1_a_complete_notification_requires_all_bytes_and_current_root() {
    let _lock = serial();
    let root = Identity {
        pid: 1234,
        created: 5678,
    };
    let frame = b"WWCHILD1\t1234\t5678\t9876\t5432\n";
    assert_eq!(
        decode_notification(frame, b"9876", root),
        Ok(Identity {
            pid: 9876,
            created: 5432
        })
    );
    for raw in [
        b"9".as_slice(),
        b"",
        b"9876\n",
        b"98760",
        b"09876",
        b"9876\0",
    ] {
        assert!(
            decode_notification(frame, raw, root).is_err(),
            "partial/extra PID accepted: {raw:?}"
        );
    }
    let invalid_frames: Vec<&[u8]> = vec![
        b"",
        b"WWCHILD1\t1234",
        b"WWCHILD1\t1234\t5678\t9876\t5432",
        b"WWCHILD1\t1234\t5678\t9876\t5432\nextra",
        b"WWCHILD1\t1234\t5678\t9876\t5432\t0\n",
        b"WWCHILD1\t1234\t5678\t0\t5432\n",
        b"WWCHILD1\t1234\t5678\t4294967296\t5432\n",
        b"WWCHILD1\t1234\t5678\t9876\t0\n",
        b"WWCHILD1\t1234\t5678\t9876\t+5432\n",
    ];
    let mut fixture = Fixture::new();
    // Hand-written invalid identities never reach a background OpenProcess.
    fixture.done.store(true, Ordering::SeqCst);
    fixture.watchdog.take().unwrap().join().unwrap();
    fixture.diagnostics.begin().unwrap();
    fixture.diagnostics.lock().root = Some(root);
    fs::write(fixture.root.path().join("grandchild.identity"), frame).unwrap();
    for raw in [
        b"9".as_slice(),
        b"",
        b"9876\n",
        b"98760",
        b"09876",
        b"9876\0",
    ] {
        fs::write(fixture.root.path().join("grandchild.pid"), raw).unwrap();
        assert!(!fixture
            .diagnostics
            .descendant(fixture.root.path(), Origin::CancelObserver));
    }
    for invalid in invalid_frames {
        fs::write(fixture.root.path().join("grandchild.identity"), invalid).unwrap();
        fs::write(fixture.root.path().join("grandchild.pid"), b"9876").unwrap();
        assert!(!fixture
            .diagnostics
            .descendant(fixture.root.path(), Origin::CancelObserver));
    }
    fs::write(fixture.root.path().join("grandchild.identity"), frame).unwrap();
    for wrong in [
        Identity { pid: 1235, ..root },
        Identity {
            created: 5679,
            ..root
        },
    ] {
        fixture.diagnostics.lock().root = Some(wrong);
        assert!(!fixture
            .diagnostics
            .descendant(fixture.root.path(), Origin::CancelObserver));
    }
    fs::write(
        fixture.root.path().join("grandchild.identity"),
        vec![b'1'; 257],
    )
    .unwrap();
    assert_eq!(
        bounded_read(&fixture.root.path().join("grandchild.identity")),
        Err("NotificationOversize")
    );
    assert!(fixture.diagnostics.snapshot().is_empty());
    assert_eq!(fixture.diagnostics.open_attempts.load(Ordering::SeqCst), 0);
    assert_eq!(fixture.diagnostics.terminations.load(Ordering::SeqCst), 0);
}

#[test]
fn d1_a_atomic_publisher_registers_owned_tree_with_matching_identity() {
    let _lock = serial();
    let mut fixture = Fixture::new();
    let generation = fixture.diagnostics.begin().unwrap();
    fixture.diagnostics.note_tree_created(generation);
    let child_stderr = fixture.root.path().join("publisher.stderr");
    let mut root = Sibling(
        Command::new(executable())
            .arg("tree")
            .arg(fixture.root.path())
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(fs::File::create(&child_stderr).unwrap())
            .spawn()
            .unwrap(),
    );
    fixture
        .diagnostics
        .track(root.0.id(), Origin::AfterCreate, None)
        .unwrap();
    let until = Instant::now() + Duration::from_secs(2);
    let ready = loop {
        if fixture
            .diagnostics
            .descendant(fixture.root.path(), Origin::CancelObserver)
        {
            break true;
        }
        if Instant::now() >= until {
            break false;
        }
        thread::sleep(Duration::from_millis(5));
    };
    assert!(
        ready,
        "synthetic publisher did not complete identity/PID notification: exit={:?} stderr={}",
        root.0.try_wait().unwrap(),
        fs::read_to_string(&child_stderr).unwrap()
    );
    // Freeze registration before counting independent handles and final closes.
    fixture.done.store(true, Ordering::SeqCst);
    fixture.watchdog.take().unwrap().join().unwrap();
    assert!(!fixture.rescued.load(Ordering::SeqCst));
    let records = fixture.diagnostics.snapshot();
    let descendant = records
        .iter()
        .find(|r| r.origin == Origin::CancelObserver)
        .unwrap();
    assert_eq!(descendant.generation, generation);
    assert_eq!(descendant.role, Role::Descendant);
    assert_eq!(descendant.notified_pid, descendant.actual.pid);
    let expected_root = fixture.diagnostics.lock().root.unwrap();
    assert_eq!(
        decode_notification(
            &bounded_read(&fixture.root.path().join("grandchild.identity")).unwrap(),
            &bounded_read(&fixture.root.path().join("grandchild.pid")).unwrap(),
            expected_root
        ),
        Ok(descendant.actual)
    );
    let diagnostic = fixture.diagnostics.clone();
    drop(fixture);
    assert!(root.0.try_wait().unwrap().is_some());
    assert!(diagnostic
        .lock()
        .rescues
        .iter()
        .all(|r| r.after.value == WAIT_OBJECT_0.0));
    assert_eq!(diagnostic.closes.load(Ordering::SeqCst), records.len());
}

#[test]
fn d1_b_creation_mismatch_closes_unknown_handle_without_termination() {
    let _lock = serial();
    let fixture = Fixture::new();
    let mut child = d1_live_child(fixture.root.path());
    use std::os::windows::io::AsRawHandle;
    let actual = identity(HANDLE(child.0.as_raw_handle() as isize)).unwrap();
    let generation = fixture.diagnostics.begin().unwrap();
    fixture.diagnostics.lock().root = Some(actual);
    let wrong = Identity {
        created: actual.created + 1,
        ..actual
    };
    let before = handle_count();
    assert_eq!(
        fixture
            .diagnostics
            .track(
                actual.pid,
                Origin::CancelObserver,
                Some((generation, actual, wrong))
            )
            .unwrap_err(),
        "HandleIdentityMismatch"
    );
    assert_eq!(handle_count(), before, "rejected handle leaked");
    assert!(fixture.diagnostics.snapshot().is_empty());
    assert_eq!(fixture.diagnostics.terminations.load(Ordering::SeqCst), 0);
    assert!(
        child.0.try_wait().unwrap().is_none(),
        "mismatched handle must be close-only"
    );
}

#[test]
fn d1_c_registry_poison_drop_is_nonpanicking_and_reaps_in_outer_process() {
    const ENV: &str = "WW_CANCEL_D1_POISON_INNER";
    const TEST: &str =
        "process_unit::d1_c_registry_poison_drop_is_nonpanicking_and_reaps_in_outer_process";
    if std::env::var_os(ENV).is_some() {
        let fixture = Fixture::new();
        let mut child = d1_live_child(fixture.root.path());
        fixture.track(child.0.id());
        let diagnostic = fixture.diagnostics.clone();
        let root = fixture.root.path().to_owned();
        let poison = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _guard = diagnostic.registry.lock().unwrap();
            panic!("intentional D1 registry poison");
        }));
        assert!(poison.is_err());
        let dropped = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| drop(fixture)));
        assert!(dropped.is_ok(), "Drop repanicked after registry poison");
        assert!(diagnostic.poison.load(Ordering::SeqCst));
        assert!(
            !diagnostic.valid(),
            "poison cannot be a successful diagnostic"
        );
        assert!(child.0.try_wait().unwrap().is_some());
        assert_eq!(diagnostic.closes.load(Ordering::SeqCst), 1);
        assert_eq!(diagnostic.terminations.load(Ordering::SeqCst), 1);
        assert!(!root.exists());
        eprintln!("D1-C cleanup=confirmed poison=true close_count=1 abort=false");
        return;
    }
    let _lock = serial();
    use std::os::windows::process::CommandExt;
    let temp = tempfile::tempdir().unwrap();
    let stderr = temp.path().join("poison.stderr");
    let mut outer = Sibling(
        Command::new(std::env::current_exe().unwrap())
            .args(["--exact", TEST, "--nocapture"])
            .env(ENV, "1")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(fs::File::create(&stderr).unwrap())
            .creation_flags(windows::Win32::System::Threading::CREATE_NO_WINDOW.0)
            .spawn()
            .unwrap(),
    );
    let until = Instant::now() + Duration::from_secs(6);
    let status = loop {
        if let Some(status) = outer.0.try_wait().unwrap() {
            break Some(status);
        }
        if Instant::now() >= until {
            break None;
        }
        thread::sleep(Duration::from_millis(10));
    };
    let report = fs::read_to_string(&stderr).unwrap();
    assert!(
        status.is_some_and(|s| s.success()),
        "D1-C outer failed/abort/timeout: {status:?} {report}"
    );
    assert!(
        report.contains("D1-C cleanup=confirmed poison=true close_count=1 abort=false"),
        "missing cleanup evidence: {report}"
    );
    eprintln!("D1-C outer exit=0, inner cleanup marker verified");
}

#[test]
fn d1_d_first_live_timeout_remains_failure_after_successful_rescue() {
    let _lock = serial();
    let fixture = Fixture::new();
    let mut child = d1_live_child(fixture.root.path());
    fixture.track(child.0.id());
    let first = fixture.diagnostics.first_waits();
    assert_eq!(first.len(), 1);
    assert_eq!(
        first[0].1.value, WAIT_TIMEOUT.0,
        "must observe an actual live child"
    );
    assert!(!fixture.diagnostics.valid());
    fixture.diagnostics.rescue(first[0].0, 238);
    let second = fixture.diagnostics.first_waits();
    assert_eq!(second[0].1.value, WAIT_OBJECT_0.0);
    assert_eq!(
        fixture.diagnostics.lock().records[0].first_wait,
        Some(first[0].1)
    );
    assert!(
        !fixture.diagnostics.valid(),
        "rescue must not convert first failure to PASS"
    );
    assert!(child.0.try_wait().unwrap().is_some());
    let diagnostic = fixture.diagnostics.clone();
    drop(fixture);
    assert_eq!(diagnostic.closes.load(Ordering::SeqCst), 1);
}

#[test]
fn d1_e_two_runs_duplicate_tracks_preserve_first_observation_and_close_once() {
    let _lock = serial();
    let fixture = Fixture::new();
    fixture
        .run(
            &fixture.spec("echo", &[]),
            &limits(),
            Instant::now() + Duration::from_secs(3),
            &CancellationToken::new(),
            &|point, pid| {
                if point == FaultPoint::AfterCreate {
                    for _ in 0..4 {
                        fixture.track(pid);
                    }
                }
                Ok(())
            },
        )
        .unwrap();
    let (old_root, first) = {
        let state = fixture.diagnostics.lock();
        (
            state.root.unwrap(),
            state
                .records
                .iter()
                .map(|r| r.first_wait)
                .collect::<Vec<_>>(),
        )
    };
    assert_eq!(first.len(), 5);
    fixture.normal("echo", &[], &limits()).unwrap();
    {
        let state = fixture.diagnostics.lock();
        assert_eq!(state.generation, 2);
        assert_eq!(state.records.len(), 6);
        assert!(state.records[..5]
            .iter()
            .all(|r| r.generation == 1 && r.role == Role::Root));
        assert_eq!(state.records[5].generation, 2);
        assert_eq!(
            state.records[..5]
                .iter()
                .map(|r| r.first_wait)
                .collect::<Vec<_>>(),
            first
        );
        assert!(state.calls.iter().all(|c| !c.descendant_ready)); // root-only is valid.
    }
    fs::write(
        fixture.root.path().join("grandchild.identity"),
        format!(
            "WWCHILD1\t{}\t{}\t9876\t5432\n",
            old_root.pid, old_root.created
        ),
    )
    .unwrap();
    fs::write(fixture.root.path().join("grandchild.pid"), b"9876").unwrap();
    assert!(!fixture
        .diagnostics
        .descendant(fixture.root.path(), Origin::CancelObserver));
    assert_eq!(fixture.diagnostics.snapshot().len(), 6);
    let diagnostic = fixture.diagnostics.clone();
    drop(fixture);
    assert_eq!(diagnostic.closes.load(Ordering::SeqCst), 6);
    assert_eq!(diagnostic.terminations.load(Ordering::SeqCst), 0);
}

#[test]
fn d1_e_record_capacity_fails_without_evicting_owned_handles() {
    let _lock = serial();
    let fixture = Fixture::new();
    let mut child = d1_live_child(fixture.root.path());
    for _ in 0..16 {
        fixture.track(child.0.id());
    }
    assert_eq!(
        fixture
            .diagnostics
            .track(child.0.id(), Origin::ExplicitTrack, None)
            .unwrap_err(),
        "DiagnosticCapacityExceeded"
    );
    assert_eq!(fixture.diagnostics.snapshot().len(), 16);
    assert!(!fixture.diagnostics.valid());
    let diagnostic = fixture.diagnostics.clone();
    drop(fixture);
    assert!(child.0.try_wait().unwrap().is_some());
    assert_eq!(diagnostic.closes.load(Ordering::SeqCst), 17);
}

#[test]
fn d1_e_generation_capacity_fails_and_still_reaps_owned_child() {
    let _lock = serial();
    let fixture = Fixture::new();
    let mut child = d1_live_child(fixture.root.path());
    fixture.track(child.0.id());
    for expected in 1..=16 {
        assert_eq!(fixture.diagnostics.begin(), Ok(expected));
    }
    assert_eq!(
        fixture.diagnostics.begin(),
        Err("DiagnosticCapacityExceeded")
    );
    assert_eq!(fixture.diagnostics.lock().generation, 16);
    assert!(!fixture.diagnostics.valid());
    let diagnostic = fixture.diagnostics.clone();
    drop(fixture);
    assert!(child.0.try_wait().unwrap().is_some());
    assert_eq!(diagnostic.closes.load(Ordering::SeqCst), 1);
}

#[test]
fn d1_e_event_capacity_fails_without_discarding_first_evidence() {
    let _lock = serial();
    let fixture = Fixture::new();
    let mut child = d1_live_child(fixture.root.path());
    fixture.track(child.0.id()); // One registration event already exists.
    let initial = fixture.diagnostics.lock().events[0].clone();
    for index in 1..256 {
        fixture
            .diagnostics
            .event(format!("synthetic transition {index}"));
    }
    assert!(!fixture.diagnostics.capacity.load(Ordering::SeqCst));
    fixture.diagnostics.event("overflow transition".into());
    assert!(!fixture.diagnostics.valid());
    assert_eq!(fixture.diagnostics.lock().events.len(), 256);
    assert_eq!(fixture.diagnostics.lock().events[0], initial);
    let diagnostic = fixture.diagnostics.clone();
    drop(fixture);
    assert!(child.0.try_wait().unwrap().is_some());
    assert_eq!(diagnostic.closes.load(Ordering::SeqCst), 1);
}

const M1_TEST: &str =
    "process_unit::m1_forced_supervisor_exit_at_after_create_reaps_child_and_preserves_sibling";

// Invoked only by the exact regression test in a separate copy of this test EXE.
// The normal test runner remains an independent observer of both OS processes.
fn m1_supervisor_at_barrier(root: PathBuf) {
    let mut in_job = BOOL(0);
    unsafe { IsProcessInJob(GetCurrentProcess(), HANDLE::default(), &mut in_job) }.unwrap();
    assert!(
        !in_job.as_bool(),
        "fixture prerequisite: supervisor must not be rescued by an outer Job"
    );
    let fixture = Fixture::new(); // Existing 8-second owned-handle watchdog.
    let spec = ProcessSpec {
        executable: executable(),
        args: vec!["hang".into(), root.as_os_str().to_owned()],
        cwd: root.clone(),
    };
    let result = fixture.run(
        &spec,
        &limits(),
        Instant::now() + Duration::from_secs(7),
        &CancellationToken::new(),
        &|point, pid| {
            if point == FaultPoint::AfterCreate {
                // Publish the complete PID atomically; the observer opens its
                // own handle before killing this supervisor. Never resume.
                fs::write(root.join("child.pid.tmp"), pid.to_string()).unwrap();
                fs::rename(root.join("child.pid.tmp"), root.join("child.pid")).unwrap();
                thread::sleep(Duration::from_secs(7));
                // If the observer fails to kill us, return through normal
                // cleanup before the independent 8-second watchdog expires.
                return Err(failure(ProcessFailureReason::Wait));
            }
            Ok(())
        },
    );
    panic!("fixture barrier expired instead of forced supervisor termination: {result:?}");
}

#[test]
fn m1_forced_supervisor_exit_at_after_create_reaps_child_and_preserves_sibling() {
    if let Some(root) = std::env::var_os(M1_SUPERVISOR_ROOT) {
        m1_supervisor_at_barrier(PathBuf::from(root));
        return;
    }
    use std::os::windows::process::CommandExt;
    use windows::Win32::System::Threading::{CREATE_BREAKAWAY_FROM_JOB, CREATE_NO_WINDOW};

    let _lock = serial();
    let fixture = Fixture::new();
    let sibling_root = tempfile::tempdir().unwrap();
    let mut sibling = Sibling(
        Command::new(executable())
            .arg("hang")
            .arg(sibling_root.path())
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .creation_flags(CREATE_NO_WINDOW.0)
            .spawn()
            .unwrap(),
    );
    let stderr_path = fixture.root.path().join("supervisor.stderr");
    let mut supervisor = Sibling(
        Command::new(std::env::current_exe().unwrap())
            .args(["--exact", M1_TEST, "--nocapture"])
            .env(M1_SUPERVISOR_ROOT, fixture.root.path())
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(fs::File::create(&stderr_path).unwrap())
            .creation_flags(CREATE_NO_WINDOW.0 | CREATE_BREAKAWAY_FROM_JOB.0)
            .spawn()
            .expect("fixture prerequisite: launch a supervisor outside any outer Job"),
    );
    let notice = fixture.root.path().join("child.pid");
    let deadline = Instant::now() + Duration::from_secs(2);
    let pid = loop {
        match fs::read_to_string(&notice) {
            Ok(text) => break Some(text.parse::<u32>().expect("complete owned-child PID")),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => panic!("read isolated barrier notice: {error}"),
        }
        if supervisor.0.try_wait().unwrap().is_some() || Instant::now() >= deadline {
            break None;
        }
        thread::sleep(Duration::from_millis(5));
    };
    let Some(pid) = pid else {
        // Do not abruptly kill a helper before its PID handoff. Give its
        // bounded hook/watchdog time to reap a child we cannot yet observe.
        let cleanup_deadline = Instant::now() + Duration::from_secs(9);
        while supervisor.0.try_wait().unwrap().is_none() && Instant::now() < cleanup_deadline {
            thread::sleep(Duration::from_millis(10));
        }
        let _ = supervisor.0.kill();
        let _ = supervisor.0.wait();
        panic!(
            "fixture did not reach AfterCreate (not behavior RED): {}",
            fs::read_to_string(&stderr_path).unwrap()
        );
    };
    let child = fixture.track(pid);
    let parent = fixture.track(supervisor.0.id());
    let mut parent_in_job = BOOL(0);
    unsafe { IsProcessInJob(parent, HANDLE::default(), &mut parent_in_job) }.unwrap();
    assert!(
        !parent_in_job.as_bool(),
        "independent observer found an outer Job: invalid fixture"
    );
    let mut child_in_job = BOOL(0);
    unsafe { IsProcessInJob(child, HANDLE::default(), &mut child_in_job) }.unwrap();
    assert_eq!(unsafe { WaitForSingleObject(child, 0) }, WAIT_TIMEOUT);
    assert!(supervisor.0.try_wait().unwrap().is_none());
    assert!(sibling.0.try_wait().unwrap().is_none());
    assert!(!fixture.root.path().join("started").exists());

    supervisor
        .0
        .kill()
        .expect("force-terminate only the owned supervisor");
    supervisor.0.wait().expect("reap the forced supervisor");
    // Freeze every oracle BEFORE recovery cleanup. A later rescue can never
    // turn WAIT_TIMEOUT into a passing observation of kill-on-close.
    let child_observed = unsafe { WaitForSingleObject(child, 5000) };
    let no_execution = !fixture.root.path().join("started").exists();
    let sibling_alive = sibling.0.try_wait().unwrap().is_none();
    let watchdog_rescued = fixture.rescued.load(Ordering::SeqCst);
    if child_observed != WAIT_OBJECT_0 {
        unsafe { TerminateProcess(child, 237) }.expect("reap only the leaked owned child");
    }
    let child_cleanup = unsafe { WaitForSingleObject(child, 2000) };
    sibling
        .0
        .kill()
        .expect("cleanup only the owned unrelated sibling");
    sibling
        .0
        .wait()
        .expect("reap the owned sibling after observing survival");
    eprintln!(
        "M1 evidence: supervisor_in_job={} child_in_job_at_create={} child_wait_before_cleanup={child_observed:?} sentinel_absent={no_execution} sibling_alive_before_cleanup={sibling_alive} watchdog_rescued={watchdog_rescued} child_cleanup={child_cleanup:?}",
        parent_in_job.as_bool(), child_in_job.as_bool()
    );
    assert_eq!(child_cleanup, WAIT_OBJECT_0, "owned child cleanup failed");
    assert!(!watchdog_rescued, "8-second watchdog rescue is not a PASS");
    assert!(no_execution, "suspended child unexpectedly executed");
    assert!(sibling_alive, "unrelated sibling was terminated");
    assert_eq!(
        child_observed, WAIT_OBJECT_0,
        "child survived forced supervisor termination at AfterCreate"
    );
    assert!(child_in_job.as_bool(), "child was not assigned at creation");
}
