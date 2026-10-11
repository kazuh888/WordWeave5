//! Bounded object traversal, independent of publication and Progress commits.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

use serde::de::DeserializeOwned;

use super::*;
use crate::assets::{AssetKind, AssetRef, AssetStore};

pub(super) const ORIGINAL_CAP: u64 = 6 * 1024 * 1024;
pub(super) const REPORT_CAP: u64 = 2 * 1024 * 1024;
const ROOT_CAP: u64 = 8 * 1024 * 1024;
const TRIAL_CAP: u64 = 256 * 1024;
pub(super) const DOCUMENT_CAP: u64 = 12 * 1024 * 1024;

pub(super) fn error(kind: HistoryErrorKind) -> HistoryError {
    HistoryError { kind }
}

pub(super) fn contract_error(err: ContractError) -> HistoryError {
    error(match err.code {
        ContractErrorCode::UnknownVersion => HistoryErrorKind::UnsupportedVersion,
        ContractErrorCode::LimitExceeded => HistoryErrorKind::LimitExceeded,
        ContractErrorCode::ReferenceMismatch => HistoryErrorKind::RelationshipMismatch,
        ContractErrorCode::Conflict => HistoryErrorKind::Conflict,
        _ => HistoryErrorKind::InvalidDocument,
    })
}

pub(super) fn related(valid: bool) -> Result<(), HistoryError> {
    if valid {
        Ok(())
    } else {
        Err(error(HistoryErrorKind::RelationshipMismatch))
    }
}

pub(super) trait Document: DeserializeOwned {
    fn check(&self) -> Result<(), HistoryError>;
}

macro_rules! documents {
    ($($ty:ty),+ $(,)?) => {$(
        impl Document for $ty {
            fn check(&self) -> Result<(), HistoryError> {
                self.validate().map_err(contract_error)
            }
        }
    )+};
}
documents!(
    HistoryRoot,
    TrialRevision,
    AnalysisRun,
    AdviceRun,
    AdviceResult,
    Reflection
);

impl Document for AnalysisReport {
    fn check(&self) -> Result<(), HistoryError> {
        if self.schema_version != 1 {
            return Err(error(HistoryErrorKind::UnsupportedVersion));
        }
        self.validate()
            .map_err(|_| error(HistoryErrorKind::InvalidDocument))
    }
}

/// Each operation has one shared budget. Failed reads consume their attempted
/// references too, so partial browsing cannot reset the budget for each branch.
pub(super) struct ReadSession<'r, 's> {
    reader: &'r HistoryReader<'s>,
    objects: BTreeMap<Sha256, BlobRef>,
    verified_bytes: BTreeSet<Sha256>,
    total_bytes: u64,
}

impl<'r, 's> ReadSession<'r, 's> {
    pub(super) fn new(reader: &'r HistoryReader<'s>) -> Self {
        Self {
            reader,
            objects: BTreeMap::new(),
            verified_bytes: BTreeSet::new(),
            total_bytes: 0,
        }
    }

    fn account(&mut self, reference: &BlobRef, cap: u64) -> Result<(), HistoryError> {
        if reference.bytes > cap {
            return Err(error(HistoryErrorKind::LimitExceeded));
        }
        reference.validate().map_err(contract_error)?;
        if let Some(existing) = self.objects.get(&reference.sha256) {
            return related(existing.bytes == reference.bytes);
        }
        let total = self
            .total_bytes
            .checked_add(reference.bytes)
            .ok_or_else(|| error(HistoryErrorKind::LimitExceeded))?;
        if self.objects.len() >= self.reader.limits.max_nodes
            || total > self.reader.limits.max_total_bytes
        {
            return Err(error(HistoryErrorKind::LimitExceeded));
        }
        self.objects
            .insert(reference.sha256.clone(), reference.clone());
        self.total_bytes = total;
        Ok(())
    }

    pub(super) fn bytes(&mut self, reference: &BlobRef, cap: u64) -> Result<Vec<u8>, HistoryError> {
        self.account(reference, cap)?;
        let root = self.reader.root.join("audio-history");
        let directory = root.join("assets");
        let path = directory.join(format!("{}.bin", reference.sha256.as_str()));
        // AssetStore checks its own directory and file. Include the storage
        // ancestry so an earlier reparse point cannot redirect this host.
        let storage_path =
            std::path::absolute(self.reader.root).map_err(|_| error(HistoryErrorKind::Io))?;
        let ancestors: Vec<_> = storage_path.ancestors().collect();
        for ancestor in ancestors.into_iter().rev() {
            safe_metadata(ancestor, true)?;
        }
        safe_metadata(&root, true)?;
        safe_metadata(&directory, true)?;
        let metadata = safe_metadata(&path, false)?;
        if metadata.len() != reference.bytes {
            return Err(error(HistoryErrorKind::DamagedObject));
        }
        let result = AssetStore::new(root).read(&AssetRef {
            id: reference.sha256.as_str().to_owned(),
            kind: AssetKind::FileBlob,
            bytes: reference.bytes,
        });
        let bytes = result.map_err(|message| {
            // The existing AssetStore exposes String errors. Match only its
            // fixed integrity codes; OS text is never returned to the caller.
            let kind = match message.as_str() {
                "媒体のサイズが一致しません。"
                | "媒体の破損を検出しました。原本を上書きしません。"
                | "媒体の形式が不正、または12MiBを超えています。" => {
                    HistoryErrorKind::DamagedObject
                }
                "媒体の保存先にリンクは使用できません。" => {
                    HistoryErrorKind::UnsafePath
                }
                _ => HistoryErrorKind::Io,
            };
            error(kind)
        })?;
        self.verified_bytes.insert(reference.sha256.clone());
        Ok(bytes)
    }

    pub(super) fn document<T: Document>(
        &mut self,
        reference: &BlobRef,
        cap: u64,
    ) -> Result<T, HistoryError> {
        let bytes = self.bytes(reference, cap)?;
        let document: T =
            serde_json::from_slice(&bytes).map_err(|_| error(HistoryErrorKind::InvalidDocument))?;
        document.check()?;
        Ok(document)
    }

    fn original(&mut self, reference: &BlobRef) -> Result<(), HistoryError> {
        self.account(reference, ORIGINAL_CAP)?;
        if !self.verified_bytes.contains(&reference.sha256) {
            self.bytes(reference, ORIGINAL_CAP)?;
        }
        Ok(())
    }

    fn trial(&mut self, reference: &BlobRef, id: &TrialId) -> Result<TrialRevision, HistoryError> {
        let trial: TrialRevision = self.document(reference, TRIAL_CAP)?;
        related(&trial.id == id)?;
        check_trial(&trial)?;
        Ok(trial)
    }

    fn previous(&mut self, current: &TrialRevision) -> Result<Option<TrialRevision>, HistoryError> {
        let Some(reference) = &current.previous else {
            return Ok(None);
        };
        let previous = self.trial(reference, &current.id)?;
        related(
            previous.revision.checked_add(1) == Some(current.revision)
                && previous.created_at == current.created_at
                && previous.input == current.input
                && (previous.claim == current.claim
                    || matches!(
                        (&previous.claim, &current.claim),
                        (PracticeClaim::Reference, PracticeClaim::Practice { .. })
                    )),
        )?;
        Ok(Some(previous))
    }

    fn trial_closure(&mut self, mut trial: TrialRevision) -> Result<(), HistoryError> {
        // Revisions strictly decrease. No recursive stack or retained chain.
        loop {
            self.original(&trial.input.audio.original)?;
            for run in trial.analysis.keys() {
                self.analysis(&trial, run)?;
            }
            for run in trial.advice.keys() {
                let view = self.advice(&trial, run)?;
                for evidence in &trial.match_evidence {
                    let expected = match evidence {
                        MatchEvidence::AiAssessed { run: id } if id == run => {
                            Some(AdviceAssessment::Assessed)
                        }
                        MatchEvidence::AiMismatch { run: id } if id == run => {
                            Some(AdviceAssessment::ReferenceMismatch)
                        }
                        MatchEvidence::AiUnassessable { run: id } if id == run => {
                            Some(AdviceAssessment::Unassessable)
                        }
                        _ => None,
                    };
                    if let Some(expected) = expected {
                        related(view.result.as_ref().map(|r| r.assessment) == Some(expected))?;
                    }
                }
            }
            for reference in &trial.reflections {
                self.reflection(&trial, reference)?;
            }
            match self.previous(&trial)? {
                Some(previous) => trial = previous,
                None => return Ok(()),
            }
        }
    }
}

fn safe_metadata(path: &Path, directory: bool) -> Result<fs::Metadata, HistoryError> {
    let metadata = fs::symlink_metadata(path).map_err(|err| {
        error(match err.kind() {
            std::io::ErrorKind::NotFound => HistoryErrorKind::MissingObject,
            _ => HistoryErrorKind::Io,
        })
    })?;
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if metadata.file_attributes() & 0x400 != 0 {
            return Err(error(HistoryErrorKind::UnsafePath));
        }
    }
    if metadata.file_type().is_symlink()
        || (directory && !metadata.is_dir())
        || (!directory && !metadata.is_file())
    {
        return Err(error(HistoryErrorKind::UnsafePath));
    }
    Ok(metadata)
}

pub(super) fn check_trial(trial: &TrialRevision) -> Result<(), HistoryError> {
    trial.validate().map_err(contract_error)?;
    related(hash_reference(&trial.input.reference.text)? == trial.input.reference.sha256)?;
    if let Some(goal) = &trial.input.goal {
        related(hash_goal(&goal.text)? == goal.version)?;
    }
    related(hash_capture(&trial.input.capture)? == trial.input.capture.version)
}

pub(super) fn verify(
    reader: &HistoryReader<'_>,
    root: &RootRef,
) -> Result<VerifiedClosure, HistoryError> {
    let mut session = ReadSession::new(reader);
    let document: HistoryRoot = session.document(root.as_blob(), ROOT_CAP)?;
    for (id, head) in &document.trials {
        let trial = session.trial(&head.document, id)?;
        session.trial_closure(trial)?;
    }
    Ok(VerifiedClosure {
        objects: session.objects.into_values().collect(),
        total_bytes: session.total_bytes,
    })
}

pub(super) fn read_page(
    reader: &HistoryReader<'_>,
    reference: Option<&RootRef>,
    after: Option<&TrialId>,
) -> HistoryView {
    let mut view = HistoryView {
        root: None,
        trials: Vec::new(),
        next_after: None,
        diagnostics: Vec::new(),
    };
    let Some(reference) = reference else {
        if after.is_some() {
            view.diagnostics.push(HistoryDiagnostic {
                trial: None,
                error: error(HistoryErrorKind::RelationshipMismatch),
            });
        }
        return view;
    };
    let mut session = ReadSession::new(reader);
    let root: HistoryRoot = match session.document(reference.as_blob(), ROOT_CAP) {
        Ok(root) => root,
        Err(error) => {
            view.diagnostics
                .push(HistoryDiagnostic { trial: None, error });
            return view;
        }
    };
    if after.is_some_and(|id| !root.trials.contains_key(id)) {
        view.diagnostics.push(HistoryDiagnostic {
            trial: None,
            error: error(HistoryErrorKind::RelationshipMismatch),
        });
        view.root = Some(root);
        return view;
    }
    let mut heads = root
        .trials
        .iter()
        .filter(|(id, _)| after.is_none_or(|after| *id > after));
    let mut last = None;
    for (id, head) in heads.by_ref().take(32) {
        last = Some(id.clone());
        match session.trial(&head.document, id) {
            Err(error) => view.diagnostics.push(HistoryDiagnostic {
                trial: Some(id.clone()),
                error,
            }),
            Ok(document) => {
                let original = session.original(&document.input.audio.original);
                let mut trial_view = TrialView {
                    document,
                    original_available: original.is_ok(),
                    diagnostics: Vec::new(),
                };
                if let Err(error) = original {
                    trial_view.diagnostics.push(HistoryDiagnostic {
                        trial: Some(id.clone()),
                        error,
                    });
                }
                // Browse old revision documents for damage, without eagerly
                // loading the run/result bodies that have individual APIs.
                let mut current = trial_view.document.clone();
                loop {
                    match session.previous(&current) {
                        Ok(Some(previous)) => current = previous,
                        Ok(None) => break,
                        Err(error) => {
                            trial_view.diagnostics.push(HistoryDiagnostic {
                                trial: Some(id.clone()),
                                error,
                            });
                            break;
                        }
                    }
                }
                view.trials.push(trial_view);
            }
        }
    }
    if heads.next().is_some() {
        view.next_after = last;
    }
    view.root = Some(root);
    view
}
