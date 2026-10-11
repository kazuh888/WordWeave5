//! Explicit, pinned external Praat installation. No download or fallback engine.

use std::{
    path::PathBuf,
    sync::atomic::{AtomicBool, Ordering},
    time::Instant,
};
use tokio_util::sync::CancellationToken;

use crate::{
    AnalysisError, AnalysisErrorKind, EngineIdentity, MonoPcm, PitchBackend, PitchProfile,
    PitchTrack,
};

pub struct PraatConfig {
    pub executable: PathBuf,
    pub work_root: PathBuf,
}

pub struct PraatBackend {
    #[cfg(windows)]
    inner: platform::Backend,
}

#[cfg(all(test, windows))]
pub(crate) use platform::DigestWriter;

#[cfg(all(test, windows))]
#[derive(Clone, Copy, Debug)]
pub(crate) enum WavTestPoint {
    WavWriterClosed,
    WavPinAttempted { acquired: bool },
    WavHashChunk,
    WavVerified,
    BeforeAnalysisRun,
}

#[cfg(all(test, windows))]
type WavHook<'a> = dyn Fn(WavTestPoint, &std::path::Path) -> Result<(), AnalysisError> + 'a;

static CLEANUP_WARNING: AtomicBool = AtomicBool::new(false);
fn error(kind: AnalysisErrorKind) -> AnalysisError {
    AnalysisError { kind }
}
fn checkpoint(deadline: Instant, cancel: &CancellationToken) -> Result<(), AnalysisError> {
    if cancel.is_cancelled() {
        Err(error(AnalysisErrorKind::Cancelled))
    } else if Instant::now() >= deadline {
        Err(error(AnalysisErrorKind::TimedOut))
    } else {
        Ok(())
    }
}

impl PraatBackend {
    #[cfg(all(test, windows))]
    pub(crate) fn extract_with_hook(
        &self,
        pcm: MonoPcm<'_>,
        profile: &PitchProfile,
        deadline: Instant,
        cancel: &CancellationToken,
        hook: &WavHook<'_>,
    ) -> Result<PitchTrack, AnalysisError> {
        checkpoint(deadline, cancel)?;
        self.inner
            .extract(pcm, profile, deadline, cancel, Some(hook))
    }

    pub fn new(
        config: PraatConfig,
        deadline: Instant,
        cancel: &CancellationToken,
    ) -> Result<Self, AnalysisError> {
        checkpoint(deadline, cancel)?;
        #[cfg(windows)]
        {
            platform::Backend::new(config, deadline, cancel).map(|inner| Self { inner })
        }
        #[cfg(not(windows))]
        {
            let _ = config;
            Err(error(AnalysisErrorKind::BackendUnavailable))
        }
    }
}

impl PitchBackend for PraatBackend {
    fn identity(&self) -> Option<EngineIdentity> {
        #[cfg(windows)]
        {
            Some(self.inner.identity.clone())
        }
        #[cfg(not(windows))]
        {
            None
        }
    }

    fn extract(
        &self,
        pcm: MonoPcm<'_>,
        profile: &PitchProfile,
        deadline: Instant,
        cancel: &CancellationToken,
    ) -> Result<PitchTrack, AnalysisError> {
        checkpoint(deadline, cancel)?;
        #[cfg(windows)]
        {
            self.inner.extract(
                pcm,
                profile,
                deadline,
                cancel,
                #[cfg(test)]
                None,
            )
        }
        #[cfg(not(windows))]
        {
            let _ = (pcm, profile);
            Err(error(AnalysisErrorKind::BackendUnavailable))
        }
    }
}

pub fn take_praat_cleanup_warning() -> bool {
    CLEANUP_WARNING.swap(false, Ordering::AcqRel)
}

#[cfg(windows)]
mod platform {
    use super::*;
    use crate::{
        praat_protocol::{classify_output, parse_pitch, write_mono_wav},
        process::{self, ProcessLimits, ProcessOutput, ProcessSpec},
        AnalysisProfile,
    };
    use sha2::{Digest, Sha256};
    use std::{
        ffi::OsString,
        fs::{File, OpenOptions},
        io::{Read, Seek, SeekFrom, Write},
        os::windows::{fs::OpenOptionsExt, io::AsRawHandle},
        path::{Component, Path},
    };
    use tempfile::TempDir;
    use windows::Win32::{
        Foundation::HANDLE,
        Storage::FileSystem::{
            GetFileInformationByHandle, BY_HANDLE_FILE_INFORMATION, FILE_ATTRIBUTE_DIRECTORY,
            FILE_ATTRIBUTE_REPARSE_POINT, FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT,
            FILE_SHARE_READ, FILE_SHARE_WRITE,
        },
    };

    const EXE_SIZE: u64 = 96_780_624;
    const EXE_HASH: &str = "22c8994fd9767a2a5c1f03f21b35e41bb5ac716c628965e5d437033eeda8fff4";
    const SCRIPT_HASH: &str = "3616a00415149e0ef5791f5ea497cbf523468695faa44fb059d911b6bca0876e";
    const SCRIPT: &[u8] = include_bytes!("../scripts/pitch.praat");
    const WORK_CAP: usize = 64 * 1024 * 1024;
    static LEASE: AtomicBool = AtomicBool::new(false);

    struct Lease;
    impl Lease {
        fn claim() -> Result<Self, AnalysisError> {
            LEASE
                .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
                .map(|_| Self)
                .map_err(|_| error(AnalysisErrorKind::ResourceLimit))
        }
    }
    impl Drop for Lease {
        fn drop(&mut self) {
            if !process::reap_unconfirmed() {
                LEASE.store(false, Ordering::Release);
            }
        }
    }
    fn unavailable<T>(_: T) -> AnalysisError {
        error(AnalysisErrorKind::BackendUnavailable)
    }
    fn failed<T>(_: T) -> AnalysisError {
        error(AnalysisErrorKind::BackendFailed)
    }

    fn info(file: &File) -> Result<BY_HANDLE_FILE_INFORMATION, AnalysisError> {
        let mut info = BY_HANDLE_FILE_INFORMATION::default();
        // SAFETY: File owns a live handle; info is a writable output value.
        unsafe { GetFileInformationByHandle(HANDLE(file.as_raw_handle() as isize), &mut info) }
            .map_err(unavailable)?;
        if info.dwFileAttributes & FILE_ATTRIBUTE_REPARSE_POINT.0 != 0 {
            return Err(unavailable(()));
        }
        Ok(info)
    }
    fn open_pin(path: &Path, directory: bool) -> Result<File, AnalysisError> {
        let flags = FILE_FLAG_OPEN_REPARSE_POINT.0
            | if directory {
                FILE_FLAG_BACKUP_SEMANTICS.0
            } else {
                0
            };
        let share = FILE_SHARE_READ.0 | if directory { FILE_SHARE_WRITE.0 } else { 0 };
        let file = OpenOptions::new()
            .read(true)
            .share_mode(share)
            .custom_flags(flags)
            .open(path)
            .map_err(unavailable)?;
        if (info(&file)?.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY.0 != 0) != directory {
            return Err(unavailable(()));
        }
        Ok(file)
    }
    /// Ancestor locks last one call, not the lifetime of the backend.
    fn pin_directories(
        path: &Path,
        deadline: Instant,
        cancel: &CancellationToken,
    ) -> Result<Vec<File>, AnalysisError> {
        if !path.is_absolute() {
            return Err(unavailable(()));
        }
        let mut current = PathBuf::new();
        let mut pins = Vec::new();
        for part in path.components() {
            checkpoint(deadline, cancel)?;
            match part {
                Component::Prefix(_) => current.push(part.as_os_str()),
                Component::RootDir | Component::Normal(_) => {
                    current.push(part.as_os_str());
                    pins.push(open_pin(&current, true)?);
                }
                _ => return Err(unavailable(())),
            }
        }
        Ok(pins)
    }
    fn same_file(left: &File, right: &File) -> Result<bool, AnalysisError> {
        let a = info(left)?;
        let b = info(right)?;
        Ok(a.dwVolumeSerialNumber == b.dwVolumeSerialNumber
            && a.nFileIndexHigh == b.nFileIndexHigh
            && a.nFileIndexLow == b.nFileIndexLow)
    }
    fn hash(
        file: &File,
        size: u64,
        expected: &str,
        deadline: Instant,
        cancel: &CancellationToken,
        #[cfg(test)] chunk_hook: Option<&dyn Fn() -> Result<(), AnalysisError>>,
    ) -> Result<(), AnalysisError> {
        checkpoint(deadline, cancel)?;
        if file.metadata().map_err(unavailable)?.len() != size {
            return Err(unavailable(()));
        }
        let mut reader = file;
        reader.seek(SeekFrom::Start(0)).map_err(unavailable)?;
        let mut scratch = [0_u8; 64 * 1024];
        let mut hasher = Sha256::new();
        let mut total = 0_u64;
        loop {
            checkpoint(deadline, cancel)?;
            let read = reader.read(&mut scratch).map_err(unavailable)?;
            if read == 0 {
                break;
            }
            #[cfg(test)]
            let hook_result = chunk_hook.map_or(Ok(()), |hook| hook());
            checkpoint(deadline, cancel)?;
            #[cfg(test)]
            hook_result?;
            total = total
                .checked_add(read as u64)
                .ok_or_else(|| unavailable(()))?;
            if total > size {
                return Err(unavailable(()));
            }
            hasher.update(&scratch[..read]);
        }
        if total != size || format!("{:x}", hasher.finalize()) != expected {
            return Err(unavailable(()));
        }
        checkpoint(deadline, cancel)
    }

    fn wav_error(mut failure: AnalysisError) -> AnalysisError {
        if failure.kind == AnalysisErrorKind::BackendUnavailable {
            failure.kind = AnalysisErrorKind::BackendFailed;
        }
        failure
    }

    fn pin_wav(
        path: &Path,
        size: usize,
        digest: [u8; 32],
        deadline: Instant,
        cancel: &CancellationToken,
        #[cfg(test)] hook: Option<&WavHook<'_>>,
    ) -> Result<File, AnalysisError> {
        checkpoint(deadline, cancel)?;
        let pin = open_pin(path, false).map_err(wav_error);
        // The release hook must observe even a failed acquisition before any
        // early return, so a competing test writer cannot obstruct cleanup.
        #[cfg(test)]
        let hook_result = hook.map_or(Ok(()), |hook| {
            hook(
                WavTestPoint::WavPinAttempted {
                    acquired: pin.is_ok(),
                },
                path,
            )
        });
        checkpoint(deadline, cancel)?;
        let pin = pin?;
        #[cfg(test)]
        hook_result?;

        let digest: sha2::digest::Output<Sha256> = digest.into();
        #[cfg(test)]
        let chunk_hook = || hook.map_or(Ok(()), |hook| hook(WavTestPoint::WavHashChunk, path));
        hash(
            &pin,
            size as u64,
            &format!("{digest:x}"),
            deadline,
            cancel,
            #[cfg(test)]
            Some(&chunk_hook),
        )
        .map_err(wav_error)?;
        // Return this verified handle, never a reopened path.
        Ok(pin)
    }

    struct Work {
        directory: Option<TempDir>,
        pins: Vec<File>,
        _root_pins: Vec<File>,
    }
    impl Work {
        fn new(
            root: &Path,
            deadline: Instant,
            cancel: &CancellationToken,
        ) -> Result<Self, AnalysisError> {
            let root_pins = pin_directories(root, deadline, cancel)?;
            let directory = tempfile::Builder::new()
                .prefix("wordweave-praat-")
                .tempdir_in(root)
                .map_err(failed)?;
            let mut work = Self {
                directory: Some(directory),
                pins: Vec::new(),
                _root_pins: root_pins,
            };
            // Version checks also own a pinned cwd until the child is reaped.
            work.pins.push(open_pin(work.path(), true)?);
            Ok(work)
        }
        fn path(&self) -> &Path {
            self.directory
                .as_ref()
                .expect("owned work directory")
                .path()
        }
        fn cleanup(&mut self) -> bool {
            let Some(directory) = self.directory.take() else {
                return true;
            };
            if process::reap_unconfirmed() {
                // Keep read pins and ancestry alive while child lifetime is unknown.
                std::mem::forget(std::mem::take(&mut self.pins));
                std::mem::forget(std::mem::take(&mut self._root_pins));
                let _ = directory.keep();
                CLEANUP_WARNING.store(true, Ordering::Release);
                return false;
            }
            self.pins.clear();
            if directory.close().is_err() {
                CLEANUP_WARNING.store(true, Ordering::Release);
                return false;
            }
            true
        }
        fn finish<T>(
            mut self,
            result: Result<T, AnalysisError>,
            deadline: Instant,
            cancel: &CancellationToken,
        ) -> Result<T, AnalysisError> {
            let clean = self.cleanup();
            checkpoint(deadline, cancel)?;
            match result {
                Err(e)
                    if matches!(
                        e.kind,
                        AnalysisErrorKind::Cancelled
                            | AnalysisErrorKind::TimedOut
                            | AnalysisErrorKind::ResourceLimit
                    ) =>
                {
                    Err(e)
                }
                _ if !clean => Err(error(if process::reap_unconfirmed() {
                    AnalysisErrorKind::ResourceLimit
                } else {
                    AnalysisErrorKind::BackendFailed
                })),
                other => other,
            }
        }
    }
    impl Drop for Work {
        fn drop(&mut self) {
            self.cleanup();
        }
    }

    /// Refuse writes before crossing the checked work estimate.
    struct BoundedWriter {
        file: File,
        bytes: usize,
        cap: usize,
    }
    impl Write for BoundedWriter {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            if bytes.len() > self.cap.saturating_sub(self.bytes) {
                return Err(std::io::Error::other("work limit"));
            }
            let written = self.file.write(bytes)?;
            self.bytes += written;
            Ok(written)
        }
        fn flush(&mut self) -> std::io::Result<()> {
            self.file.flush()
        }
    }

    /// Hash only bytes accepted by the inner writer, including short writes.
    pub(crate) struct DigestWriter<W: Write> {
        inner: W,
        digest: Sha256,
    }
    impl<W: Write> DigestWriter<W> {
        pub(crate) fn new(inner: W) -> Self {
            Self {
                inner,
                digest: Sha256::new(),
            }
        }
        pub(crate) fn finish(self) -> [u8; 32] {
            let Self { inner, digest } = self;
            drop(inner);
            digest.finalize().into()
        }
    }
    impl<W: Write> Write for DigestWriter<W> {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            let written = self.inner.write(bytes)?;
            self.digest.update(&bytes[..written]);
            Ok(written)
        }
        fn flush(&mut self) -> std::io::Result<()> {
            self.inner.flush()
        }
    }

    pub(super) struct Backend {
        config: PraatConfig,
        executable: File,
        pub(super) identity: EngineIdentity,
    }
    impl Backend {
        pub(super) fn new(
            config: PraatConfig,
            deadline: Instant,
            cancel: &CancellationToken,
        ) -> Result<Self, AnalysisError> {
            let _lease = Lease::claim()?;
            if !config.executable.is_absolute() || !config.work_root.is_absolute() {
                return Err(unavailable(()));
            }
            let _parents = pin_directories(
                config.executable.parent().ok_or_else(|| unavailable(()))?,
                deadline,
                cancel,
            )?;
            let executable = open_pin(&config.executable, false)?;
            hash(
                &executable,
                EXE_SIZE,
                EXE_HASH,
                deadline,
                cancel,
                #[cfg(test)]
                None,
            )?;
            if format!("{:x}", Sha256::digest(SCRIPT)) != SCRIPT_HASH {
                return Err(unavailable(()));
            }
            let backend = Self {
                config,
                executable,
                identity: EngineIdentity {
                    name: "Praat".into(),
                    version: "7.0.02".into(),
                    binary_sha256: Some(EXE_HASH.into()),
                    script_sha256: Some(SCRIPT_HASH.into()),
                },
            };
            let work = Work::new(&backend.config.work_root, deadline, cancel)?;
            let result = (|| {
                let output = backend.run(
                    vec!["--utf8".into(), "--version".into()],
                    work.path(),
                    deadline,
                    cancel,
                )?;
                if output.stdout != b"Praat 7.0.02 (August 26 2026)\n"
                    && output.stdout != b"Praat 7.0.02 (August 26 2026)\r\n"
                {
                    return Err(unavailable(()));
                }
                Ok(())
            })();
            work.finish(result, deadline, cancel)?;
            Ok(backend)
        }
        fn run(
            &self,
            args: Vec<OsString>,
            cwd: &Path,
            deadline: Instant,
            cancel: &CancellationToken,
        ) -> Result<ProcessOutput, AnalysisError> {
            let _parents = pin_directories(
                self.config
                    .executable
                    .parent()
                    .ok_or_else(|| unavailable(()))?,
                deadline,
                cancel,
            )?;
            let current = open_pin(&self.config.executable, false)?;
            if !same_file(&self.executable, &current)? {
                return Err(unavailable(()));
            }
            hash(
                &current,
                EXE_SIZE,
                EXE_HASH,
                deadline,
                cancel,
                #[cfg(test)]
                None,
            )?;
            checkpoint(deadline, cancel)?;
            let result = process::supervise(
                &ProcessSpec {
                    executable: self.config.executable.clone(),
                    args,
                    cwd: cwd.to_owned(),
                },
                &ProcessLimits {
                    stdout_bytes: 1_048_576,
                    stderr_bytes: 65_536,
                    memory_bytes: 536_870_912,
                },
                deadline,
                cancel,
            );
            checkpoint(deadline, cancel)?;
            let output = result.map_err(|failure| error(failure.kind))?;
            classify_output(&output).map_err(|failure| error(failure.kind))?;
            Ok(output)
        }
        pub(super) fn extract(
            &self,
            pcm: MonoPcm<'_>,
            profile: &PitchProfile,
            deadline: Instant,
            cancel: &CancellationToken,
            #[cfg(test)] hook: Option<&WavHook<'_>>,
        ) -> Result<PitchTrack, AnalysisError> {
            let _lease = Lease::claim()?;
            if *profile != AnalysisProfile::v1().pitch {
                return Err(error(AnalysisErrorKind::UnsupportedProfile));
            }
            if pcm.samples.is_empty()
                || !(8000..=48000).contains(&pcm.sample_rate)
                || pcm.samples.len() > pcm.sample_rate as usize * 60
                || pcm.samples.len() as u64 * 1000 < pcm.sample_rate as u64 * 60
            {
                return Err(error(AnalysisErrorKind::InvalidInput));
            }
            for chunk in pcm.samples.chunks(8192) {
                checkpoint(deadline, cancel)?;
                if chunk
                    .iter()
                    .any(|sample| !sample.is_finite() || !(-1.0..=1.0).contains(sample))
                {
                    return Err(error(AnalysisErrorKind::InvalidInput));
                }
            }
            let wav_bytes = pcm
                .samples
                .len()
                .checked_mul(8)
                .and_then(|n| n.checked_add(58))
                .ok_or_else(|| error(AnalysisErrorKind::ResourceLimit))?;
            if wav_bytes
                .checked_add(SCRIPT.len())
                .is_none_or(|n| n > WORK_CAP)
            {
                return Err(error(AnalysisErrorKind::ResourceLimit));
            }
            let mut work = Work::new(&self.config.work_root, deadline, cancel)?;
            macro_rules! wav_boundary {
                ($point:expr, $path:expr) => {{
                    #[cfg(test)]
                    let hook_result = hook.map_or(Ok(()), |hook| hook($point, $path));
                    checkpoint(deadline, cancel)?;
                    #[cfg(test)]
                    hook_result?;
                }};
            }
            let result = (|| {
                let script_path = work.path().join("pitch.praat");
                let wav_path = work.path().join("mono.wav");
                {
                    let file = OpenOptions::new()
                        .write(true)
                        .create_new(true)
                        .open(&script_path)
                        .map_err(failed)?;
                    let mut out = BoundedWriter {
                        file,
                        bytes: 0,
                        cap: SCRIPT.len(),
                    };
                    out.write_all(SCRIPT).map_err(failed)?;
                    out.flush().map_err(failed)?;
                }
                let wav_digest = {
                    let file = OpenOptions::new()
                        .write(true)
                        .create_new(true)
                        .share_mode(FILE_SHARE_READ.0)
                        .open(&wav_path)
                        .map_err(failed)?;
                    let mut out = DigestWriter::new(BoundedWriter {
                        file,
                        bytes: 0,
                        cap: wav_bytes,
                    });
                    write_mono_wav(&mut out, pcm, deadline, cancel)?;
                    // write_mono_wav flushes successfully before returning;
                    // finish closes the writer before exposing its digest.
                    out.finish()
                };
                wav_boundary!(WavTestPoint::WavWriterClosed, &wav_path);
                let wav_pin = pin_wav(
                    &wav_path,
                    wav_bytes,
                    wav_digest,
                    deadline,
                    cancel,
                    #[cfg(test)]
                    hook,
                )?;
                work.pins.push(wav_pin);
                wav_boundary!(WavTestPoint::WavVerified, &wav_path);
                let script_pin = open_pin(&script_path, false)?;
                hash(
                    &script_pin,
                    SCRIPT.len() as u64,
                    SCRIPT_HASH,
                    deadline,
                    cancel,
                    #[cfg(test)]
                    None,
                )?;
                work.pins.push(script_pin);
                wav_boundary!(WavTestPoint::BeforeAnalysisRun, &wav_path);
                let output = self.run(
                    vec![
                        "--run".into(),
                        "--no-pref-files".into(),
                        "--no-plugins".into(),
                        "--utf8".into(),
                        script_path.into_os_string(),
                        wav_path.into_os_string(),
                    ],
                    work.path(),
                    deadline,
                    cancel,
                )?;
                let track = parse_pitch(
                    &output.stdout,
                    pcm.samples.len() as f64 / f64::from(pcm.sample_rate),
                    profile,
                )?;
                checkpoint(deadline, cancel)?;
                Ok(track)
            })();
            work.finish(result, deadline, cancel)
        }
    }
}
