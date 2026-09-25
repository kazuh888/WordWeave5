//! Durable, content-addressed originals. Metadata contains references, never media bytes.

use serde::{Deserialize, Serialize};
use std::{
    fs::{self, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};

pub const MAX_ASSET_BYTES: usize = 12 * 1024 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssetKind {
    AudioWav,
    ImagePng,
    ImageBmp,
    ImageGif,
    ImageJpeg,
    InkJson,
    FileBlob,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetRef {
    pub id: String,
    pub kind: AssetKind,
    pub bytes: u64,
}
impl AssetRef {
    pub fn validate(&self) -> Result<(), String> {
        if self.id.len() != 64
            || !self
                .id
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            || self.bytes == 0
            || self.bytes > MAX_ASSET_BYTES as u64
        {
            return Err("媒体の参照が不正です。".into());
        }
        Ok(())
    }
    fn filename(&self) -> String {
        format!(
            "{}.{}",
            self.id,
            match self.kind {
                AssetKind::AudioWav => "wav",
                AssetKind::ImagePng => "png",
                AssetKind::ImageBmp => "bmp",
                AssetKind::ImageGif => "gif",
                AssetKind::ImageJpeg => "jpg",
                AssetKind::InkJson => "json",
                AssetKind::FileBlob => "bin",
            }
        )
    }
}

fn check_format(kind: AssetKind, bytes: &[u8]) -> Result<(), String> {
    let valid = !bytes.is_empty()
        && bytes.len() <= MAX_ASSET_BYTES
        && match kind {
            AssetKind::ImagePng => bytes.starts_with(b"\x89PNG\r\n\x1a\n"),
            AssetKind::ImageBmp => bytes.starts_with(b"BM"),
            AssetKind::ImageGif => bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a"),
            AssetKind::ImageJpeg => bytes.starts_with(b"\xFF\xD8\xFF"),
            AssetKind::AudioWav => bytes.starts_with(b"RIFF") && bytes.get(8..12) == Some(b"WAVE"),
            AssetKind::InkJson => {
                serde_json::from_slice::<serde_json::Value>(bytes).is_ok_and(|v| v.is_object())
            }
            AssetKind::FileBlob => true,
        };
    if valid {
        Ok(())
    } else {
        Err("媒体の形式が不正、または12MiBを超えています。".into())
    }
}

fn no_link(path: &Path) -> Result<(), String> {
    let meta = fs::symlink_metadata(path).map_err(|e| e.to_string())?;
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if meta.file_attributes() & 0x400 != 0 {
            return Err("媒体の保存先にリンクは使用できません。".into());
        }
    }
    if meta.file_type().is_symlink() {
        return Err("媒体の保存先にリンクは使用できません。".into());
    }
    Ok(())
}

/// Windows CNG SHA-256; no credentials, network or custom cryptography.
#[cfg(windows)]
pub fn sha256(bytes: &[u8]) -> Result<String, String> {
    use std::ffi::c_void;
    #[link(name = "bcrypt")]
    extern "system" {
        fn BCryptOpenAlgorithmProvider(
            handle: *mut *mut c_void,
            algorithm: *const u16,
            provider: *const u16,
            flags: u32,
        ) -> i32;
        fn BCryptHash(
            handle: *mut c_void,
            secret: *const u8,
            secret_len: u32,
            input: *const u8,
            input_len: u32,
            output: *mut u8,
            output_len: u32,
        ) -> i32;
        fn BCryptCloseAlgorithmProvider(handle: *mut c_void, flags: u32) -> i32;
    }
    let len = u32::try_from(bytes.len()).map_err(|_| "ハッシュ対象が大きすぎます。")?;
    let name: Vec<u16> = "SHA256\0".encode_utf16().collect();
    let mut handle = std::ptr::null_mut();
    let mut output = [0u8; 32];
    // CNG does not mutate the input. All buffers remain alive for the synchronous call.
    unsafe {
        if BCryptOpenAlgorithmProvider(&mut handle, name.as_ptr(), std::ptr::null(), 0) < 0 {
            return Err("SHA-256を初期化できません。".into());
        }
        let status = BCryptHash(
            handle,
            std::ptr::null(),
            0,
            bytes.as_ptr(),
            len,
            output.as_mut_ptr(),
            32,
        );
        BCryptCloseAlgorithmProvider(handle, 0);
        if status < 0 {
            return Err("SHA-256を計算できません。".into());
        }
    }
    Ok(output.iter().map(|b| format!("{b:02x}")).collect())
}
#[cfg(not(windows))]
pub fn sha256(_bytes: &[u8]) -> Result<String, String> {
    Err("媒体保存はWindows専用です。".into())
}

pub struct AssetStore {
    root: PathBuf,
}

impl AssetStore {
    /// Includes detached originals: removing an attachment is not file deletion.
    pub fn inventory(&self) -> Result<Vec<AssetRef>, String> {
        if !self.root.try_exists().map_err(|e| e.to_string())? {
            return Ok(Vec::new());
        }
        no_link(&self.root)?;
        let mut refs = Vec::new();
        for entry in fs::read_dir(&self.root).map_err(|e| e.to_string())? {
            let path = entry.map_err(|e| e.to_string())?.path();
            if path
                .file_name()
                .and_then(|s| s.to_str())
                .is_some_and(|s| s.starts_with(".tmp-"))
            {
                continue;
            }
            no_link(&path)?;
            let kind = match path.extension().and_then(|s| s.to_str()) {
                Some("wav") => AssetKind::AudioWav,
                Some("png") => AssetKind::ImagePng,
                Some("bmp") => AssetKind::ImageBmp,
                Some("gif") => AssetKind::ImageGif,
                Some("jpg") => AssetKind::ImageJpeg,
                Some("json") => AssetKind::InkJson,
                Some("bin") => AssetKind::FileBlob,
                _ => return Err("媒体フォルダーに未対応ファイルがあります。".into()),
            };
            let reference = AssetRef {
                id: path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .into(),
                kind,
                bytes: fs::metadata(&path).map_err(|e| e.to_string())?.len(),
            };
            self.read(&reference)?;
            refs.push(reference);
        }
        Ok(refs)
    }
    pub fn new(root: PathBuf) -> Self {
        Self {
            root: root.join("assets"),
        }
    }

    pub fn put(&self, kind: AssetKind, bytes: &[u8]) -> Result<AssetRef, String> {
        check_format(kind, bytes)?;
        let reference = AssetRef {
            id: sha256(bytes)?,
            kind,
            bytes: bytes.len() as u64,
        };
        fs::create_dir_all(&self.root).map_err(|e| e.to_string())?;
        no_link(&self.root)?;
        let path = self.root.join(reference.filename());
        if path.try_exists().map_err(|e| e.to_string())? {
            self.read(&reference)?;
            return Ok(reference);
        }
        let temp = self.root.join(format!(
            ".tmp-{}-{}",
            std::process::id(),
            chrono::Utc::now().timestamp_nanos_opt().unwrap()
        ));
        let result = (|| {
            let mut f = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&temp)
                .map_err(|e| e.to_string())?;
            f.write_all(bytes)
                .and_then(|_| f.sync_all())
                .map_err(|e| e.to_string())?;
            drop(f);
            // A hard link publishes without replacing any previously existing original.
            if let Err(error) = fs::hard_link(&temp, &path) {
                if error.kind() != std::io::ErrorKind::AlreadyExists {
                    return Err(error.to_string());
                }
            }
            self.read(&reference)?;
            Ok(reference)
        })();
        let _ = fs::remove_file(temp);
        result
    }

    pub fn read(&self, reference: &AssetRef) -> Result<Vec<u8>, String> {
        reference.validate()?;
        no_link(&self.root)?;
        let path = self.root.join(reference.filename());
        no_link(&path)?;
        let f = fs::File::open(path).map_err(|e| e.to_string())?;
        if f.metadata().map_err(|e| e.to_string())?.len() != reference.bytes {
            return Err("媒体のサイズが一致しません。".into());
        }
        let mut bytes = Vec::new();
        f.take(MAX_ASSET_BYTES as u64 + 1)
            .read_to_end(&mut bytes)
            .map_err(|e| e.to_string())?;
        check_format(reference.kind, &bytes)?;
        if bytes.len() as u64 != reference.bytes || sha256(&bytes)? != reference.id {
            return Err("媒体の破損を検出しました。原本を上書きしません。".into());
        }
        Ok(bytes)
    }

    pub fn export_to(&self, dest: &Path, refs: &[AssetRef]) -> Result<(), String> {
        for reference in refs {
            self.read(reference)?;
        }
        fs::create_dir(dest).map_err(|e| e.to_string())?;
        let target = Self::new(dest.to_owned());
        for reference in refs {
            target.put(reference.kind, &self.read(reference)?)?;
        }
        // Manifest is the completion marker; an interrupted export has no manifest.
        crate::store::atomic_write(
            &dest.join("manifest.json"),
            &serde_json::to_vec_pretty(&serde_json::json!({"format_version":1,"assets":refs}))
                .map_err(|e| e.to_string())?,
        )
    }
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;
    use std::{
        fs,
        sync::atomic::{AtomicU64, Ordering},
    };

    static NEXT: AtomicU64 = AtomicU64::new(0);
    const PNG: &[u8] = b"\x89PNG\r\n\x1a\noriginal image bytes";
    const WAV: &[u8] = b"RIFF\x04\0\0\0WAVE";
    const INK: &[u8] = br#"{"strokes":[[[0.1,0.2],[0.3,0.4]]]}"#;

    struct Fixture(PathBuf);
    impl Fixture {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "wordweave-assets-{}-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&path).unwrap();
            Self(path)
        }
        fn store(&self) -> AssetStore {
            AssetStore::new(self.0.clone())
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn preserves_original_bytes_and_deduplicates_identical_content() {
        let f = Fixture::new();
        for (kind, bytes) in [
            (AssetKind::ImagePng, PNG),
            (AssetKind::AudioWav, WAV),
            (AssetKind::InkJson, INK),
        ] {
            let reference = f.store().put(kind, bytes).unwrap();
            assert_eq!(reference.bytes, bytes.len() as u64);
            assert_eq!(reference.id.len(), 64);
            assert_eq!(f.store().read(&reference).unwrap(), bytes);
            assert_eq!(f.store().put(kind, bytes).unwrap(), reference);
        }
        assert_eq!(fs::read_dir(f.0.join("assets")).unwrap().count(), 3);
    }

    #[test]
    fn file_blob_roundtrips_as_content_addressed_bin_and_appears_in_inventory() {
        let f = Fixture::new();
        let original = b"\0\xffarbitrary\r\nbytes";
        let reference = f.store().put(AssetKind::FileBlob, original).unwrap();
        assert_eq!(reference.kind, AssetKind::FileBlob);
        assert_eq!(reference.bytes, original.len() as u64);
        assert_eq!(reference.id, sha256(original).unwrap());
        assert!(f
            .0
            .join("assets")
            .join(format!("{}.bin", reference.id))
            .exists());
        assert_eq!(f.store().read(&reference).unwrap(), original);
        assert_eq!(
            f.store().put(AssetKind::FileBlob, original).unwrap(),
            reference
        );
        assert_eq!(f.store().inventory().unwrap(), vec![reference.clone()]);

        let destination = f.0.join("blob-backup");
        f.store()
            .export_to(&destination, &[reference.clone()])
            .unwrap();
        assert_eq!(
            AssetStore::new(destination).read(&reference).unwrap(),
            original
        );

        let path = f.0.join("assets").join(format!("{}.bin", reference.id));
        let mut corrupted = original.to_vec();
        corrupted[0] ^= 1;
        fs::write(&path, &corrupted).unwrap();
        assert!(f.store().read(&reference).is_err());
        assert!(f.store().put(AssetKind::FileBlob, original).is_err());
        assert_eq!(fs::read(path).unwrap(), corrupted);
    }

    #[test]
    fn rejects_wrong_format_and_oversize_without_creating_originals() {
        let f = Fixture::new();
        assert!(f.store().put(AssetKind::ImagePng, b"not png").is_err());
        assert!(f.store().put(AssetKind::AudioWav, b"RIFF1234AVI ").is_err());
        assert!(f.store().put(AssetKind::InkJson, b"not json").is_err());
        assert!(f.store().put(AssetKind::InkJson, b"[]").is_err());
        assert!(f.store().put(AssetKind::FileBlob, b"").is_err());
        assert!(f
            .store()
            .put(AssetKind::ImagePng, &vec![0; MAX_ASSET_BYTES + 1])
            .is_err());
        assert!(f
            .store()
            .put(AssetKind::FileBlob, &vec![0; MAX_ASSET_BYTES + 1])
            .is_err());
        assert!(!f.0.join("assets").exists());
    }

    #[test]
    fn references_cannot_escape_asset_directory() {
        let f = Fixture::new();
        for id in [
            "../outside",
            "C:\\private",
            "abc:stream",
            &"A".repeat(64),
            &"a".repeat(63),
        ] {
            let reference = AssetRef {
                id: id.into(),
                kind: AssetKind::ImagePng,
                bytes: 24,
            };
            assert!(f.store().read(&reference).is_err());
        }
    }

    #[test]
    fn rejects_changed_size_and_content_instead_of_overwriting() {
        let f = Fixture::new();
        let mut reference = f.store().put(AssetKind::ImagePng, PNG).unwrap();
        reference.bytes += 1;
        assert!(f.store().read(&reference).is_err());
        reference.bytes -= 1;
        let path = f.0.join("assets").join(format!("{}.png", reference.id));
        let mut corrupted = PNG.to_vec();
        *corrupted.last_mut().unwrap() ^= 1;
        fs::write(&path, &corrupted).unwrap();
        assert!(f.store().read(&reference).is_err());
        assert!(f.store().put(AssetKind::ImagePng, PNG).is_err());
        assert_eq!(fs::read(&path).unwrap(), corrupted);
    }

    #[test]
    fn exports_manifest_and_originals_without_overwriting_existing_destination() {
        let f = Fixture::new();
        let reference = f.store().put(AssetKind::ImagePng, PNG).unwrap();
        let dest = f.0.join("backup");
        f.store().export_to(&dest, &[reference.clone()]).unwrap();
        let manifest: serde_json::Value =
            serde_json::from_slice(&fs::read(dest.join("manifest.json")).unwrap()).unwrap();
        assert_eq!(manifest["format_version"], 1);
        assert_eq!(manifest["assets"][0]["id"], reference.id);
        assert_eq!(AssetStore::new(dest.clone()).read(&reference).unwrap(), PNG);
        assert!(f.store().export_to(&dest, &[reference]).is_err());
        assert!(dest.join("manifest.json").exists());
    }

    #[test]
    fn missing_original_aborts_export_before_publishing_backup() {
        let f = Fixture::new();
        let reference = AssetRef {
            id: "a".repeat(64),
            kind: AssetKind::ImagePng,
            bytes: 24,
        };
        let dest = f.0.join("backup");
        assert!(f.store().export_to(&dest, &[reference]).is_err());
        assert!(!dest.exists());
    }
}
