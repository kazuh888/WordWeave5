use crate::{audio::check_cancel, error::err, read_audio, AudioInput, ErrorCode, SafeError};
use std::{
    path::PathBuf,
    sync::{
        mpsc::{self, Receiver, TryRecvError},
        Mutex, OnceLock,
    },
    thread::{self, JoinHandle},
};
use tokio_util::sync::CancellationToken;

#[derive(Default)]
struct Registry {
    closed: bool,
    workers: Vec<(CancellationToken, JoinHandle<()>)>,
}
fn registry() -> &'static Mutex<Registry> {
    static REGISTRY: OnceLock<Mutex<Registry>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(Registry::default()))
}

/// A one-shot, nonblocking input load. Drop only requests cancellation; shutdown reaps workers.
pub struct AudioLoadJob {
    cancel: CancellationToken,
    receiver: Option<Receiver<Result<AudioInput, SafeError>>>,
}
impl AudioLoadJob {
    pub fn start(path: PathBuf) -> Result<Self, SafeError> {
        Self::start_with(registry(), move |cancel| read_audio(&path, cancel))
    }
    fn start_with(
        registry: &Mutex<Registry>,
        load: impl FnOnce(&CancellationToken) -> Result<AudioInput, SafeError> + Send + 'static,
    ) -> Result<Self, SafeError> {
        let mut registry = registry.lock().unwrap_or_else(|p| p.into_inner());
        if registry.closed {
            return Err(err(ErrorCode::InvalidState));
        }
        let mut i = 0;
        while i < registry.workers.len() {
            if registry.workers[i].1.is_finished() {
                let (_, worker) = registry.workers.swap_remove(i);
                let _ = worker.join();
            } else {
                i += 1;
            }
        }
        let cancel = CancellationToken::new();
        let worker_cancel = cancel.clone();
        let (sender, receiver) = mpsc::channel();
        let worker = thread::Builder::new()
            .name("audio-load".into())
            .spawn(move || {
                let result = check_cancel(&worker_cancel).and_then(|_| load(&worker_cancel));
                // load returns only after child/pipe/temp cleanup. Even a raced success is invalidated.
                let result = check_cancel(&worker_cancel).and(result);
                let _ = sender.send(result);
            })
            .map_err(|_| err(ErrorCode::AudioIo))?;
        registry.workers.push((cancel.clone(), worker));
        Ok(Self {
            cancel,
            receiver: Some(receiver),
        })
    }
    pub fn try_take(&mut self) -> Option<Result<AudioInput, SafeError>> {
        let receiver = self.receiver.as_ref()?;
        let result = match receiver.try_recv() {
            Ok(result) => result,
            Err(TryRecvError::Empty) => return None,
            Err(TryRecvError::Disconnected) => Err(err(ErrorCode::AudioDecodeFailed)),
        };
        self.receiver = None;
        Some(check_cancel(&self.cancel).and(result))
    }
    pub fn cancel(&self) {
        self.cancel.cancel();
    }
}
impl Drop for AudioLoadJob {
    fn drop(&mut self) {
        self.cancel();
    }
}

/// Process exit only: prevents new loads, cancels all workers and joins outside the lock.
pub fn shutdown_audio_jobs() {
    shutdown(registry());
}
fn shutdown(registry: &Mutex<Registry>) {
    let workers = {
        let mut registry = registry.lock().unwrap_or_else(|p| p.into_inner());
        registry.closed = true;
        for (cancel, _) in &registry.workers {
            cancel.cancel();
        }
        std::mem::take(&mut registry.workers)
    };
    for (_, worker) in workers {
        let _ = worker.join();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cancellation_discards_ready_result_and_shutdown_closes_registry() {
        let registry = Mutex::new(Registry::default());
        let (entered, ready) = mpsc::channel();
        let (release, wait) = mpsc::channel();
        let mut job = AudioLoadJob::start_with(&registry, move |_| {
            entered.send(()).unwrap();
            wait.recv().unwrap();
            AudioInput::parse(include_bytes!("../tests/fixtures/formats/tone.wav").to_vec())
        })
        .unwrap();
        ready.recv().unwrap();
        assert!(job.try_take().is_none());
        job.cancel();
        release.send(()).unwrap();
        shutdown(&registry);
        assert_eq!(
            job.try_take().unwrap().unwrap_err().code(),
            ErrorCode::Cancelled
        );
        assert!(job.try_take().is_none());
        assert!(AudioLoadJob::start_with(&registry, |_| unreachable!()).is_err());
        shutdown(&registry);
    }
    #[test]
    fn panic_is_terminal_instead_of_pending() {
        let registry = Mutex::new(Registry::default());
        let mut job =
            AudioLoadJob::start_with(&registry, |_| panic!("synthetic worker panic")).unwrap();
        // Join without cancellation to observe the disconnected channel.
        let (_, handle) = registry.lock().unwrap().workers.pop().unwrap();
        assert!(handle.join().is_err());
        assert_eq!(
            job.try_take().unwrap().unwrap_err().code(),
            ErrorCode::AudioDecodeFailed
        );
    }
    #[test]
    fn ready_success_is_discarded_after_cancel() {
        let registry = Mutex::new(Registry::default());
        let mut job = AudioLoadJob::start_with(&registry, |_| {
            AudioInput::parse(include_bytes!("../tests/fixtures/formats/tone.wav").to_vec())
        })
        .unwrap();
        let (_, handle) = registry.lock().unwrap().workers.pop().unwrap();
        handle.join().unwrap();
        job.cancel();
        assert_eq!(
            job.try_take().unwrap().unwrap_err().code(),
            ErrorCode::Cancelled
        );
        assert!(job.try_take().is_none());
    }
}
