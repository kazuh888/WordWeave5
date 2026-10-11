//! Strict intent files, retained-byte accounting, and recovery API values.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::Read,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use super::closure::{contract_error, related, ReadSession, DOCUMENT_CAP};
use super::*;

pub(super) const INTENT_CAP: usize = 512 * 1024;
const ENTRY_CAP: usize = 100_000;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecoveryView {
    pub candidates: Vec<RecoveryCandidate>,
    pub next_after: Option<OperationId>,
    pub diagnostics: Vec<HistoryError>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecoveryCandidate {
    pub operation: OperationId,
    pub request: Sha256,
    pub state: RecoveryState,
    pub missing: Vec<BlobRef>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecoveryState {
    AlreadyCommitted,
    OriginalMissing,
    AssociationReady,
    PayloadMissing,
    PublishedUncommitted,
    Conflict,
    Damaged,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RecoveryIntent {
    pub schema_version: u32,
    pub operation: OperationId,
    pub request: Sha256,
    pub mutation_json: String,
    #[serde(deserialize_with = "Option::deserialize")]
    pub base: Option<RootRef>,
    pub planned_root: RootRef,
    pub planned_objects: Vec<BlobRef>,
}

impl RecoveryIntent {
    pub fn encode(&self) -> Result<Vec<u8>, HistoryError> {
        self.validate()?;
        let bytes =
            serde_json::to_vec(self).map_err(|_| error(HistoryErrorKind::InvalidDocument))?;
        if bytes.len() > INTENT_CAP {
            return Err(error(HistoryErrorKind::LimitExceeded));
        }
        Ok(bytes)
    }

    fn validate(&self) -> Result<(), HistoryError> {
        if self.schema_version != 1 {
            return Err(error(HistoryErrorKind::UnsupportedVersion));
        }
        self.operation.validate().map_err(contract_error)?;
        self.request.validate().map_err(contract_error)?;
        if self.mutation_json.len() > INTENT_CAP || self.planned_objects.len() > 8 {
            return Err(error(HistoryErrorKind::LimitExceeded));
        }
        related(hash_bytes(self.mutation_json.as_bytes())? == self.request)?;
        let mutation: HistoryMutation = serde_json::from_str(&self.mutation_json)
            .map_err(|_| error(HistoryErrorKind::InvalidDocument))?;
        mutation.validate()?;
        if let Some(base) = &self.base {
            base.validate().map_err(contract_error)?;
        }
        self.planned_root.validate().map_err(contract_error)?;
        let mut hashes = BTreeSet::new();
        for reference in &self.planned_objects {
            reference.validate().map_err(contract_error)?;
            related(hashes.insert(&reference.sha256))?;
        }
        related(self.planned_objects.last() == Some(self.planned_root.as_blob()))?;
        for reference in mutation.payload_refs() {
            related(self.planned_objects.contains(reference))?;
        }
        Ok(())
    }
}

pub(super) fn io_error(err: std::io::Error) -> HistoryError {
    error(if err.kind() == std::io::ErrorKind::NotFound {
        HistoryErrorKind::MissingObject
    } else {
        HistoryErrorKind::Io
    })
}

pub(super) fn metadata(path: &Path, directory: bool) -> Result<Option<fs::Metadata>, HistoryError> {
    let meta = match fs::symlink_metadata(path) {
        Ok(meta) => meta,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(err) => return Err(io_error(err)),
    };
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if meta.file_attributes() & 0x400 != 0 {
            return Err(error(HistoryErrorKind::UnsafePath));
        }
    }
    if meta.file_type().is_symlink()
        || (directory && !meta.is_dir())
        || (!directory && !meta.is_file())
    {
        return Err(error(HistoryErrorKind::UnsafePath));
    }
    Ok(Some(meta))
}

pub(super) fn paths(storage: &Storage) -> Result<(), HistoryError> {
    let absolute = std::path::absolute(&storage.dir).map_err(io_error)?;
    let ancestors: Vec<_> = absolute.ancestors().collect();
    for ancestor in ancestors.into_iter().rev() {
        if metadata(ancestor, true)?.is_none() {
            return Err(error(HistoryErrorKind::MissingObject));
        }
    }
    let history = storage.dir.join("audio-history");
    if metadata(&history, true)?.is_some() {
        for directory in ["assets", "recovery", "work"] {
            metadata(&history.join(directory), true)?;
        }
    }
    Ok(())
}

pub(super) fn guard(storage: &Storage) -> Result<(), HistoryError> {
    paths(storage)?;
    if metadata(&storage.dir.join("pending-material-commit.json"), false)?.is_some() {
        return Err(error(HistoryErrorKind::PendingMaterial));
    }
    Ok(())
}

pub(super) fn intent_path(storage: &Storage, operation: &OperationId) -> PathBuf {
    storage
        .dir
        .join("audio-history/recovery")
        .join(format!("{}.json", operation.as_str()))
}

fn read_intent_file(
    path: &Path,
    operation: &OperationId,
) -> Result<Option<(RecoveryIntent, Vec<u8>)>, HistoryError> {
    let Some(meta) = metadata(path, false)? else {
        return Ok(None);
    };
    if meta.len() > INTENT_CAP as u64 {
        return Err(error(HistoryErrorKind::LimitExceeded));
    }
    let file = fs::File::open(path).map_err(io_error)?;
    let mut bytes = Vec::new();
    file.take(INTENT_CAP as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(io_error)?;
    if bytes.len() > INTENT_CAP {
        return Err(error(HistoryErrorKind::LimitExceeded));
    }
    if bytes.len() as u64 != meta.len() {
        return Err(error(HistoryErrorKind::DamagedObject));
    }
    let intent: RecoveryIntent =
        serde_json::from_slice(&bytes).map_err(|_| error(HistoryErrorKind::InvalidDocument))?;
    intent.validate()?;
    related(&intent.operation == operation)?;
    Ok(Some((intent, bytes)))
}

pub(super) fn read_intent(
    storage: &Storage,
    operation: &OperationId,
) -> Result<Option<(RecoveryIntent, Vec<u8>)>, HistoryError> {
    operation.validate().map_err(contract_error)?;
    paths(storage)?;
    read_intent_file(&intent_path(storage, operation), operation)
}

pub(super) fn save_intent(
    storage: &Storage,
    intent: &RecoveryIntent,
    bytes: &[u8],
) -> Result<(), HistoryError> {
    guard(storage)?;
    if read_intent(storage, &intent.operation)?.is_some() {
        return Err(error(HistoryErrorKind::Busy));
    }
    for directory in [
        storage.dir.join("audio-history"),
        storage.dir.join("audio-history/recovery"),
    ] {
        guard(storage)?;
        if metadata(&directory, true)?.is_none() {
            fs::create_dir(&directory).map_err(io_error)?;
        }
        metadata(&directory, true)?.ok_or_else(|| error(HistoryErrorKind::MissingObject))?;
    }
    guard(storage)?;
    let write = crate::store::atomic_write(&intent_path(storage, &intent.operation), bytes);
    // A reported write failure can follow a successful rename. Only exact
    // readback establishes the durable intent; never publish based on Ok alone.
    match read_intent(storage, &intent.operation)? {
        Some((actual, actual_bytes)) if actual == *intent && actual_bytes == bytes => Ok(()),
        _ => Err(error(if write.is_err() {
            HistoryErrorKind::Io
        } else {
            HistoryErrorKind::DamagedObject
        })),
    }
}

/// Enumerate names without letting one damaged intent hide healthy candidates.
pub(super) fn operations(
    storage: &Storage,
) -> Result<(Vec<OperationId>, Vec<HistoryError>), HistoryError> {
    paths(storage)?;
    let directory = storage.dir.join("audio-history/recovery");
    if metadata(&directory, true)?.is_none() {
        return Ok((vec![], vec![]));
    }
    let mut operations = Vec::new();
    let mut diagnostics = Vec::new();
    for (count, entry) in fs::read_dir(&directory).map_err(io_error)?.enumerate() {
        if count >= ENTRY_CAP {
            return Err(error(HistoryErrorKind::LimitExceeded));
        }
        let entry = entry.map_err(io_error)?;
        let name = entry.file_name();
        let operation = name
            .to_str()
            .and_then(|name| name.strip_suffix(".json"))
            .and_then(|name| OperationId::parse(name).ok());
        if let Some(operation) = operation {
            operations.push(operation);
        } else {
            diagnostics.push(error(HistoryErrorKind::InvalidDocument));
        }
    }
    operations.sort();
    Ok((operations, diagnostics))
}

/// The caller has checked the entire replacement and quota before this point.
pub(super) fn replace_intent(
    storage: &Storage,
    old: &RecoveryIntent,
    old_bytes: &[u8],
    next: &RecoveryIntent,
    next_bytes: &[u8],
) -> Result<(), HistoryError> {
    guard(storage)?;
    let stored = read_intent(storage, &old.operation)?;
    super::mutation::conflict(
        stored
            .as_ref()
            .is_some_and(|(intent, bytes)| intent == old && bytes == old_bytes),
    )?;
    if old == next {
        // Keep the original exact intent encoding on a non-rebased retry.
        return Ok(());
    }
    let write = crate::store::atomic_write(&intent_path(storage, &old.operation), next_bytes);
    match read_intent(storage, &old.operation)? {
        Some((actual, bytes)) if actual == *next && bytes == next_bytes => Ok(()),
        _ => Err(error(if write.is_err() {
            HistoryErrorKind::Io
        } else {
            HistoryErrorKind::DamagedObject
        })),
    }
}

pub(super) fn remove_intent(
    storage: &Storage,
    intent: &RecoveryIntent,
    bytes: &[u8],
) -> Result<(), HistoryError> {
    guard(storage)?;
    let stored = read_intent(storage, &intent.operation)?;
    super::mutation::conflict(
        stored
            .as_ref()
            .is_some_and(|(actual, actual_bytes)| actual == intent && actual_bytes == bytes),
    )?;
    fs::remove_file(intent_path(storage, &intent.operation)).map_err(io_error)
}

pub(super) struct Inventory {
    pub ledger: QuotaLedger,
    pub objects: BTreeMap<Sha256, BlobRef>,
}

pub(super) fn add(left: u64, right: u64) -> Result<u64, HistoryError> {
    left.checked_add(right)
        .ok_or_else(|| error(HistoryErrorKind::LimitExceeded))
}

pub(super) fn inventory(storage: &Storage, quota: HistoryQuota) -> Result<Inventory, HistoryError> {
    paths(storage)?;
    let history = storage.dir.join("audio-history");
    let mut ledger = QuotaLedger {
        object_bytes: 0,
        recovery_bytes: 0,
        residual_bytes: 0,
        retained_bytes: 0,
        limit_bytes: quota.limit_bytes,
        warning: false,
    };
    let mut objects = BTreeMap::new();
    if metadata(&history, true)?.is_none() {
        return Ok(Inventory { ledger, objects });
    }
    let reader = HistoryReader::with_limits(
        storage,
        HistoryLimits {
            max_nodes: ENTRY_CAP,
            max_total_bytes: u64::MAX,
        },
    )?;
    let mut reads = ReadSession::new(&reader);
    let mut directories = vec![history.clone()];
    let mut entries = 0;
    let mut unknown = false;
    while let Some(directory) = directories.pop() {
        metadata(&directory, true)?.ok_or_else(|| error(HistoryErrorKind::MissingObject))?;
        for entry in fs::read_dir(&directory).map_err(io_error)? {
            let entry = entry.map_err(io_error)?;
            entries += 1;
            if entries > ENTRY_CAP {
                return Err(error(HistoryErrorKind::LimitExceeded));
            }
            let path = entry.path();
            let kind = entry.file_type().map_err(io_error)?;
            let name = entry.file_name();
            let name = name
                .to_str()
                .ok_or_else(|| error(HistoryErrorKind::InvalidDocument))?;
            if kind.is_dir() {
                metadata(&path, true)?;
                if directory == history && matches!(name, "assets" | "recovery" | "work")
                    || directory.starts_with(history.join("work"))
                {
                    directories.push(path);
                } else {
                    return Err(error(HistoryErrorKind::UnsafePath));
                }
                continue;
            }
            let meta =
                metadata(&path, false)?.ok_or_else(|| error(HistoryErrorKind::MissingObject))?;
            if directory == history.join("assets") && name.ends_with(".bin") {
                let hash = Sha256::parse(&name[..name.len() - 4]).map_err(contract_error)?;
                let reference = BlobRef {
                    sha256: hash.clone(),
                    bytes: meta.len(),
                };
                reads.bytes(&reference, DOCUMENT_CAP)?;
                ledger.object_bytes = add(ledger.object_bytes, meta.len())?;
                objects.insert(hash, reference);
            } else if directory == history.join("recovery") && name.ends_with(".json") {
                let operation =
                    OperationId::parse(&name[..name.len() - 5]).map_err(contract_error)?;
                read_intent_file(&path, &operation)?
                    .ok_or_else(|| error(HistoryErrorKind::MissingObject))?;
                ledger.recovery_bytes = add(ledger.recovery_bytes, meta.len())?;
            } else {
                ledger.residual_bytes = add(ledger.residual_bytes, meta.len())?;
                if !name.starts_with(".tmp-")
                    && !name.contains(".tmp-")
                    && !directory.starts_with(history.join("work"))
                {
                    unknown = true;
                }
            }
        }
    }
    ledger.retained_bytes = add(
        add(ledger.object_bytes, ledger.recovery_bytes)?,
        ledger.residual_bytes,
    )?;
    ledger.warning = u128::from(ledger.retained_bytes) * 5 >= u128::from(quota.limit_bytes) * 4;
    if unknown {
        return Err(error(HistoryErrorKind::InvalidDocument));
    }
    Ok(Inventory { ledger, objects })
}
