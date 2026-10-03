use crate::{error::err, ErrorCode, SafeError, REQUESTED_EFFORT};
use zeroize::Zeroizing;
#[derive(Clone, PartialEq, Eq)]
pub struct ApiHost {
    canonical: String,
    region: Region,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Region {
    #[default]
    Tokyo,
    Singapore,
    Beijing,
    HongKong,
    Frankfurt,
    Virginia,
}
impl Region {
    pub const ALL: [Self; 6] = [
        Self::Tokyo,
        Self::Singapore,
        Self::Beijing,
        Self::HongKong,
        Self::Frankfurt,
        Self::Virginia,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Self::Tokyo => "東京",
            Self::Singapore => "シンガポール",
            Self::Beijing => "北京",
            Self::HongKong => "香港",
            Self::Frankfurt => "フランクフルト",
            Self::Virginia => "バージニア",
        }
    }
    pub fn id(self) -> &'static str {
        match self {
            Self::Tokyo => "ap-northeast-1",
            Self::Singapore => "ap-southeast-1",
            Self::Beijing => "cn-beijing",
            Self::HongKong => "cn-hongkong",
            Self::Frankfurt => "eu-central-1",
            Self::Virginia => "us-east-1",
        }
    }
    pub fn example_url(self) -> String {
        format!(
            "https://{{WorkspaceId}}.{}.maas.aliyuncs.com/compatible-mode/v1",
            self.id()
        )
    }
}
impl ApiHost {
    pub fn parse(base: &str) -> Result<Self, SafeError> {
        let bad = || err(ErrorCode::InvalidHost);
        let raw = base.strip_prefix("https://").ok_or_else(bad)?;
        if raw.contains(['@', ':', '%', '?', '#']) {
            return Err(bad());
        }
        let (host, path) = raw.split_once('/').map_or((raw, ""), |(h, p)| (h, p));
        if !["", "compatible-mode/v1", "compatible-mode/v1/"].contains(&path) {
            return Err(bad());
        }
        let (region, label) = Region::ALL
            .into_iter()
            .find_map(|region| {
                host.strip_suffix(&format!(".{}.maas.aliyuncs.com", region.id()))
                    .map(|label| (region, label))
            })
            .ok_or_else(bad)?;
        if label.is_empty()
            || label.len() > 63
            || !label
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
            || label.starts_with('-')
            || label.ends_with('-')
        {
            return Err(bad());
        }
        Ok(Self {
            canonical: format!("https://{host}/compatible-mode/v1"),
            region,
        })
    }
    pub fn parse_for_region(base: &str, region: Region) -> Result<Self, SafeError> {
        let host = Self::parse(base)?;
        if host.region != region {
            return Err(err(ErrorCode::InvalidHost));
        }
        Ok(host)
    }
    pub fn region(&self) -> Region {
        self.region
    }
    pub fn as_str(&self) -> &str {
        &self.canonical
    }
}
#[derive(Clone, PartialEq, Eq)]
pub struct TokyoHost(ApiHost);
impl TokyoHost {
    pub fn parse(base: &str) -> Result<Self, SafeError> {
        ApiHost::parse_for_region(base, Region::Tokyo).map(Self)
    }
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}
impl From<TokyoHost> for ApiHost {
    fn from(host: TokyoHost) -> Self {
        host.0
    }
}
#[derive(Clone)]
pub struct ApiKey(Zeroizing<String>);
impl ApiKey {
    pub fn new(value: String) -> Result<Self, SafeError> {
        if value.trim().is_empty()
            || value.chars().any(|c| c.is_control())
            || reqwest::header::HeaderValue::from_str(&format!("Bearer {value}")).is_err()
        {
            return Err(err(ErrorCode::InvalidKey));
        }
        Ok(Self(Zeroizing::new(value)))
    }
    pub(crate) fn value(&self) -> &str {
        &self.0
    }
}
impl std::fmt::Debug for ApiKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ApiKey([redacted])")
    }
}
#[derive(Clone)]
pub struct Connection {
    host: ApiHost,
    key: ApiKey,
}
impl Connection {
    pub fn new(host: impl Into<ApiHost>, key: ApiKey) -> Self {
        Self {
            host: host.into(),
            key,
        }
    }
    pub fn host(&self) -> &ApiHost {
        &self.host
    }
    /// Retain the opaque key only for the same canonical destination.
    pub fn with_host(&self, host: ApiHost) -> Result<Self, SafeError> {
        if host != self.host {
            return Err(err(ErrorCode::InvalidKey));
        }
        Ok(Self::new(host, self.key.clone()))
    }
    pub fn effort(&self) -> &'static str {
        REQUESTED_EFFORT
    }
    pub(crate) fn key(&self) -> &ApiKey {
        &self.key
    }
}
impl std::fmt::Debug for Connection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Connection([redacted])")
    }
}
pub trait CredentialStore: Send + Sync {
    fn load(&self) -> Result<Option<Connection>, SafeError>;
    fn save(&self, value: &Connection) -> Result<(), SafeError>;
}
#[cfg(all(windows, feature = "windows-credentials"))]
pub struct WindowsCredentialStore {
    target: &'static str,
}
#[cfg(all(windows, feature = "windows-credentials"))]
impl WindowsCredentialStore {
    pub fn new() -> Self {
        Self {
            target: "WordWeave5.QwenAudio.Connection.v1",
        }
    }
    pub fn for_target(target: &'static str) -> Result<Self, SafeError> {
        if target.is_empty() || target.contains('\0') || target.encode_utf16().count() > 32767 {
            return Err(err(ErrorCode::InvalidState));
        }
        Ok(Self { target })
    }
    pub fn target(&self) -> &'static str {
        self.target
    }
}
#[cfg(all(windows, feature = "windows-credentials"))]
impl Default for WindowsCredentialStore {
    fn default() -> Self {
        Self::new()
    }
}
#[cfg(all(windows, feature = "windows-credentials"))]
impl CredentialStore for WindowsCredentialStore {
    fn load(&self) -> Result<Option<Connection>, SafeError> {
        use windows::{
            core::PCWSTR,
            Win32::{Foundation::ERROR_NOT_FOUND, Security::Credentials::*},
        };
        let target: Vec<u16> = self.target.encode_utf16().chain(Some(0)).collect();
        let mut ptr = std::ptr::null_mut();
        unsafe {
            if let Err(e) = CredReadW(PCWSTR(target.as_ptr()), CRED_TYPE_GENERIC, 0, &mut ptr) {
                if e.code() == ERROR_NOT_FOUND.to_hresult() {
                    return Ok(None);
                }
                return Err(err(ErrorCode::CredentialRead));
            }
            let record = &*ptr;
            let result = (|| {
                let mut bytes =
                    copy_and_wipe_blob(record.CredentialBlob, record.CredentialBlobSize as usize)?;
                let connection = decode_record(&bytes);
                bytes.fill(0);
                connection.map(Some)
            })();
            CredFree(ptr.cast());
            result
        }
    }
    fn save(&self, value: &Connection) -> Result<(), SafeError> {
        use windows::{core::PWSTR, Win32::Security::Credentials::*};
        let mut target: Vec<u16> = self.target.encode_utf16().chain(Some(0)).collect();
        #[derive(serde::Serialize)]
        struct Saved<'a> {
            version: u8,
            host: &'a str,
            key: &'a str,
            effort: &'a str,
        }
        let wire = Saved {
            version: 1,
            host: value.host().as_str(),
            key: value.key().value(),
            effort: REQUESTED_EFFORT,
        };
        let mut bytes =
            Zeroizing::new(serde_json::to_vec(&wire).map_err(|_| err(ErrorCode::CredentialWrite))?);
        if bytes.len() > 2560 {
            return Err(err(ErrorCode::CredentialWrite));
        }
        let record = CREDENTIALW {
            Type: CRED_TYPE_GENERIC,
            TargetName: PWSTR(target.as_mut_ptr()),
            CredentialBlobSize: bytes.len() as u32,
            CredentialBlob: bytes.as_mut_ptr(),
            Persist: CRED_PERSIST_LOCAL_MACHINE,
            ..Default::default()
        };
        unsafe { CredWriteW(&record, 0).map_err(|_| err(ErrorCode::CredentialWrite)) }
    }
}
/// The caller must provide a writable allocation of `len` bytes for a non-null
/// pointer whose size passes the local record bounds. Invalid metadata is never
/// dereferenced or written. Credential ownership remains with the caller.
#[cfg(all(windows, feature = "windows-credentials"))]
unsafe fn copy_and_wipe_blob(blob: *mut u8, len: usize) -> Result<Zeroizing<Vec<u8>>, SafeError> {
    if blob.is_null() || !(1..=2560).contains(&len) {
        return Err(err(ErrorCode::CredentialRead));
    }
    let copy = Zeroizing::new(std::slice::from_raw_parts(blob, len).to_vec());
    std::ptr::write_bytes(blob, 0, len);
    Ok(copy)
}
#[cfg(all(test, windows, feature = "windows-credentials"))]
#[path = "../tests/internal/credential_record.rs"]
mod credential_record_tests;
#[cfg(all(windows, feature = "windows-credentials"))]
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Record {
    version: u8,
    host: String,
    #[serde(deserialize_with = "key_record")]
    key: Zeroizing<String>,
    effort: String,
}
/// Pure decoding shared by the Windows adapter and offline compatibility tests.
/// This boundary has no store handle and never migrates or writes a record.
#[cfg(all(windows, feature = "windows-credentials"))]
fn decode_record(bytes: &[u8]) -> Result<Connection, SafeError> {
    let wire: Record = serde_json::from_slice(bytes).map_err(|_| err(ErrorCode::CredentialRead))?;
    if wire.version != 1 || wire.effort != REQUESTED_EFFORT {
        return Err(err(ErrorCode::CredentialRead));
    }
    Ok(Connection::new(
        ApiHost::parse(&wire.host).map_err(|_| err(ErrorCode::CredentialRead))?,
        ApiKey::new(wire.key.to_string()).map_err(|_| err(ErrorCode::CredentialRead))?,
    ))
}
#[cfg(all(windows, feature = "windows-credentials"))]
fn key_record<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Zeroizing<String>, D::Error> {
    Ok(Zeroizing::new(<String as serde::Deserialize>::deserialize(
        d,
    )?))
}
