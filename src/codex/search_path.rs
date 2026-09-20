//! Search sources are ordered explicitly; never change the process environment.
use std::{ffi::OsStr, path::PathBuf};

pub(super) fn directories() -> Vec<PathBuf> {
    let process = std::env::var_os("PATH");
    #[cfg(windows)]
    {
        use windows::{core::w, Win32::System::Registry::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE}};
        let machine = registry_path(HKEY_LOCAL_MACHINE, w!(r"SYSTEM\CurrentControlSet\Control\Session Manager\Environment"));
        let user = registry_path(HKEY_CURRENT_USER, w!("Environment"));
        combine(process.as_deref(), machine.as_deref(), user.as_deref())
    }
    #[cfg(not(windows))]
    combine(process.as_deref(), None, None)
}

pub(super) fn combine(process: Option<&OsStr>, machine: Option<&OsStr>, user: Option<&OsStr>) -> Vec<PathBuf> {
    let mut dirs = Vec::<PathBuf>::new();
    for path in [process, machine, user].into_iter().flatten() {
        for dir in std::env::split_paths(path).filter(|dir| dir.is_absolute()) {
            let duplicate = dirs.iter().any(|existing| {
                if cfg!(windows) {
                    existing.as_os_str().eq_ignore_ascii_case(dir.as_os_str())
                } else {
                    existing == &dir
                }
            });
            if !duplicate { dirs.push(dir); }
        }
    }
    dirs
}

#[cfg(windows)]
fn registry_path(key: windows::Win32::System::Registry::HKEY, subkey: windows::core::PCWSTR) -> Option<std::ffi::OsString> {
    use std::os::windows::ffi::OsStringExt;
    use windows::{core::w, Win32::System::Registry::{RegGetValueW, RRF_RT_REG_SZ, RRF_RT_REG_EXPAND_SZ}};
    // Bound the read to Windows' environment-string limit. RegGetValueW expands
    // REG_EXPAND_SZ (e.g. %USERPROFILE%) and rejects unrelated registry types.
    let mut buffer = vec![0u16; 32_768];
    let mut bytes = (buffer.len() * 2) as u32;
    // SAFETY: predefined read-only root, static NUL-terminated names, and a live
    // aligned buffer whose byte capacity is passed to the Windows API.
    let status = unsafe {
        RegGetValueW(key, subkey, w!("Path"), RRF_RT_REG_SZ | RRF_RT_REG_EXPAND_SZ,
            None, Some(buffer.as_mut_ptr().cast()), Some(&mut bytes))
    };
    if status.is_err() || bytes % 2 != 0 || bytes as usize > buffer.len() * 2 { return None; }
    let value = &buffer[..bytes as usize / 2];
    let end = value.iter().position(|&c| c == 0)?;
    Some(std::ffi::OsString::from_wide(&value[..end]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn machine_path_is_available_without_process_path() {
        let dir = std::env::temp_dir().join("wordweave-machine");
        let value = std::env::join_paths([&dir]).unwrap();
        assert_eq!(combine(None, Some(&value), None), vec![dir]);
    }

    #[test]
    fn user_path_is_available_without_process_path() {
        let dir = std::env::temp_dir().join("wordweave ユーザー tools");
        let value = std::env::join_paths([&dir]).unwrap();
        assert_eq!(combine(None, None, Some(&value)), vec![dir]);
    }

    #[test]
    fn process_then_machine_then_user_preserves_order_and_removes_duplicates() {
        let root = std::env::temp_dir();
        let a = root.join("first");
        let b = root.join("second");
        let c = root.join("third");
        let process = std::env::join_paths([&a]).unwrap();
        let machine = std::env::join_paths([&a, &b]).unwrap();
        let user = std::env::join_paths([&b, &c]).unwrap();
        assert_eq!(combine(Some(&process), Some(&machine), Some(&user)), vec![a, b, c]);
    }

    #[test]
    fn empty_and_relative_entries_do_not_search_the_working_directory() {
        let absolute = std::env::temp_dir().join("absolute");
        let value = std::env::join_paths([std::path::Path::new(""), std::path::Path::new("relative"), &absolute]).unwrap();
        assert_eq!(combine(Some(&value), None, None), vec![absolute]);
    }

    #[cfg(windows)]
    #[test]
    fn windows_duplicates_are_case_insensitive() {
        assert_eq!(combine(Some(OsStr::new(r"C:\Tools")), Some(OsStr::new(r"c:\tools")), None), vec![PathBuf::from(r"C:\Tools")]);
    }

    #[cfg(windows)]
    #[test]
    fn absent_registry_path_is_a_nonfatal_missing_source() {
        use windows::{core::w, Win32::System::Registry::HKEY_CURRENT_USER};
        assert!(registry_path(HKEY_CURRENT_USER, w!("WordWeave5-Missing-Path-Test-8e247793")).is_none());
    }
}
