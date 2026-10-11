//! Isolated synthetic process fixture, never a real Praat/audio/network client.
use std::{
    env, fs,
    io::{self, Write},
    path::PathBuf,
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

fn main() {
    let args: Vec<_> = env::args_os().skip(1).collect();
    let mode = args[0].to_str().expect("fixture mode");
    let root = PathBuf::from(&args[1]);
    assert!(
        root.is_absolute() && root.is_dir(),
        "isolated fixture root required"
    );
    if mode != "grandchild" {
        fs::write(root.join("started"), b"synthetic child started").unwrap();
    }
    match mode {
        "echo" => {
            for arg in &args[2..] {
                let text = arg.to_str().unwrap().as_bytes();
                io::stdout()
                    .write_all(&(text.len() as u32).to_le_bytes())
                    .unwrap();
                io::stdout().write_all(text).unwrap();
            }
        }
        "output" => {
            let out: usize = args[2].to_str().unwrap().parse().unwrap();
            let err: usize = args[3].to_str().unwrap().parse().unwrap();
            // Concurrent writers reveal a supervisor which drains one pipe to EOF first.
            let writer = thread::spawn(move || emit(io::stderr(), err, b'E'));
            emit(io::stdout(), out, b'O');
            writer.join().unwrap();
        }
        "flood" => {
            thread::spawn(|| loop {
                emit(io::stderr(), 8192, b'E');
            });
            loop {
                emit(io::stdout(), 8192, b'O');
            }
        }
        "hang" => bounded_sleep(),
        "exit" => std::process::exit(args[2].to_str().unwrap().parse().unwrap()),
        "tree" | "tree-exit" => {
            let mut child = Command::new(env::current_exe().unwrap())
                .arg("grandchild")
                .arg(&root)
                .stdin(Stdio::null())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .spawn()
                .unwrap();
            #[cfg(windows)]
            publish_identity(&root, &child);
            fs::write(root.join("grandchild.pid"), child.id().to_string()).unwrap();
            if mode == "tree" {
                bounded_sleep();
                let _ = child.kill();
                let _ = child.wait();
            } else {
                #[cfg(windows)]
                if let Err(reason) =
                    complete_tree_exit(&root, &child, Instant::now() + Duration::from_secs(2))
                {
                    eprintln!("A1 tree-exit fixture failure: {reason}");
                    std::process::exit(241);
                }
            }
            // tree-exit intentionally leaves a descendant holding both inherited pipes.
        }
        "grandchild" => bounded_sleep(),
        "memory" => {
            fs::write(root.join("allocation-attempted"), b"attempting 128 MiB").unwrap();
            let mut allocation = Vec::<u8>::new();
            if allocation.try_reserve_exact(128 * 1024 * 1024).is_err() {
                fs::write(root.join("allocation-denied"), b"128 MiB denied").unwrap();
                eprint!("Error: Out of memory: synthetic allocation denied");
            } else {
                allocation.resize(128 * 1024 * 1024, 0x5a);
                fs::write(root.join("allocation-allowed"), b"128 MiB allocated").unwrap();
                std::hint::black_box(&allocation);
            }
        }
        #[cfg(windows)]
        "event" => {
            use windows::Win32::{Foundation::HANDLE, System::Threading::SetEvent};
            let handle: isize = args[2].to_str().unwrap().parse().unwrap();
            // A parent-owned inheritable event must NOT be in the child's handle list.
            let inherited = unsafe { SetEvent(HANDLE(handle)) }.is_ok();
            print!("{}", if inherited { "inherited" } else { "blocked" });
        }
        _ => panic!("unknown synthetic fixture mode"),
    }
}

pub(crate) fn tree_exit_path(dir: &std::path::Path, root: (u32, u64), extension: &str) -> PathBuf {
    dir.join(format!("tree-exit-{}-{}.{}", root.0, root.1, extension))
}

pub(crate) fn handshake_bytes(header: &str, root: (u32, u64), child: (u32, u64)) -> Vec<u8> {
    format!(
        "{header}\t{}\t{}\t{}\t{}\n",
        root.0, root.1, child.0, child.1
    )
    .into_bytes()
}

pub(crate) fn validate_handshake(
    bytes: &[u8],
    header: &str,
    root: (u32, u64),
    child: (u32, u64),
) -> Result<(), &'static str> {
    if bytes.len() > 256 {
        return Err("HandshakeOversize");
    }
    if bytes != handshake_bytes(header, root, child) {
        return Err("HandshakeIdentityMismatch");
    }
    Ok(())
}

pub(crate) fn atomic_notice(path: &std::path::Path, bytes: &[u8]) -> Result<(), &'static str> {
    if bytes.len() > 256 {
        return Err("HandshakeOversize");
    }
    if path.exists() {
        return Err("HandshakeAlreadyExists");
    }
    let temporary = path.with_extension(format!(
        "{}.tmp",
        path.extension()
            .and_then(|s| s.to_str())
            .ok_or("HandshakePath")?
    ));
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .map_err(|_| "HandshakeCreateFailed")?;
    file.write_all(bytes).map_err(|_| "HandshakeWriteFailed")?;
    file.flush().map_err(|_| "HandshakeFlushFailed")?;
    drop(file);
    fs::rename(temporary, path).map_err(|_| "HandshakeRenameFailed")
}

pub(crate) fn wait_tree_exit_ack(
    dir: &std::path::Path,
    root: (u32, u64),
    child: (u32, u64),
    deadline: Instant,
) -> Result<(), &'static str> {
    use std::io::Read;
    let path = tree_exit_path(dir, root, "ack");
    loop {
        if Instant::now() >= deadline {
            return Err("AckDeadline");
        }
        match fs::File::open(&path) {
            Ok(file) => {
                let mut bytes = Vec::new();
                file.take(257)
                    .read_to_end(&mut bytes)
                    .map_err(|_| "AckReadFailed")?;
                if Instant::now() >= deadline {
                    return Err("AckDeadline");
                }
                return validate_handshake(&bytes, "WWTREEEXITACK1", root, child);
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(_) => return Err("AckReadFailed"),
        }
        thread::sleep(Duration::from_millis(5));
    }
}

#[cfg(windows)]
pub(crate) fn complete_tree_exit(
    dir: &std::path::Path,
    child: &std::process::Child,
    deadline: Instant,
) -> Result<(), &'static str> {
    use std::os::windows::io::AsRawHandle;
    use windows::Win32::{
        Foundation::{HANDLE, WAIT_TIMEOUT},
        System::Threading::{GetCurrentProcess, WaitForSingleObject},
    };
    let root = process_identity(unsafe { GetCurrentProcess() })?;
    let child_handle = HANDLE(child.as_raw_handle() as isize);
    let descendant = process_identity(child_handle)?;
    wait_tree_exit_ack(dir, root, descendant, deadline)?;
    if unsafe { WaitForSingleObject(child_handle, 0) } != WAIT_TIMEOUT {
        return Err("DescendantNotLiveAtReady");
    }
    atomic_notice(
        &tree_exit_path(dir, root, "ready"),
        &handshake_bytes("WWTREEEXITREADY1", root, descendant),
    )
}

#[cfg(windows)]
fn process_identity(
    handle: windows::Win32::Foundation::HANDLE,
) -> Result<(u32, u64), &'static str> {
    use windows::Win32::{
        Foundation::FILETIME,
        System::Threading::{GetProcessId, GetProcessTimes},
    };
    let mut times = [FILETIME::default(); 4];
    let [created, exited, kernel, user] = &mut times;
    unsafe { GetProcessTimes(handle, created, exited, kernel, user) }
        .map_err(|_| "IdentityTimesFailed")?;
    let pid = unsafe { GetProcessId(handle) };
    if pid == 0 {
        return Err("IdentityPidFailed");
    }
    Ok((
        pid,
        ((created.dwHighDateTime as u64) << 32) | created.dwLowDateTime as u64,
    ))
}

#[cfg(windows)]
fn publish_identity(root: &std::path::Path, child: &std::process::Child) {
    use std::os::windows::io::AsRawHandle;
    use windows::Win32::{Foundation::HANDLE, System::Threading::GetCurrentProcess};
    let (root_pid, root_created) = process_identity(unsafe { GetCurrentProcess() }).unwrap();
    let (child_pid, child_created) =
        process_identity(HANDLE(child.as_raw_handle() as isize)).unwrap();
    let frame = format!("WWCHILD1\t{root_pid}\t{root_created}\t{child_pid}\t{child_created}\n");
    assert!(frame.len() <= 256);
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(root.join("grandchild.identity.tmp"))
        .unwrap();
    file.write_all(frame.as_bytes()).unwrap();
    file.flush().unwrap();
    drop(file);
    fs::rename(
        root.join("grandchild.identity.tmp"),
        root.join("grandchild.identity"),
    )
    .unwrap();
}

fn emit(mut stream: impl Write, count: usize, value: u8) {
    let chunk = [value; 8192];
    let mut remaining = count;
    while remaining != 0 {
        let take = remaining.min(chunk.len());
        if stream.write_all(&chunk[..take]).is_err() {
            return;
        }
        remaining -= take;
    }
    let _ = stream.flush();
}

fn bounded_sleep() {
    // Second safety net if both supervisor and test watchdog disappear.
    thread::sleep(Duration::from_secs(15));
}
