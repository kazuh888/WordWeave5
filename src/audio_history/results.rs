//! Lazy result reads validate membership without requiring original audio.

use acoustic_analysis::AnalysisStatus;

use super::closure::{related, ReadSession, DOCUMENT_CAP, REPORT_CAP};
use super::*;

impl ReadSession<'_, '_> {
    pub(super) fn analysis(
        &mut self,
        trial: &TrialRevision,
        id: &RunId,
    ) -> Result<AnalysisView, HistoryError> {
        let reference = trial
            .analysis
            .get(id)
            .ok_or_else(|| error(HistoryErrorKind::RelationshipMismatch))?;
        let run: AnalysisRun = self.document(reference, DOCUMENT_CAP)?;
        related(
            &run.id == id
                && run.trial == trial.id
                && run.audio == trial.input.audio.original.sha256,
        )?;
        let result = run
            .result
            .as_ref()
            .map(|reference| {
                let report: AnalysisReport = self.document(reference, REPORT_CAP)?;
                check_analysis_result(trial, &run, &report)?;
                Ok(report)
            })
            .transpose()?;
        Ok(AnalysisView { run, result })
    }

    pub(super) fn advice(
        &mut self,
        trial: &TrialRevision,
        id: &RunId,
    ) -> Result<AdviceView, HistoryError> {
        let reference = trial
            .advice
            .get(id)
            .ok_or_else(|| error(HistoryErrorKind::RelationshipMismatch))?;
        let run: AdviceRun = self.document(reference, DOCUMENT_CAP)?;
        related(
            &run.id == id
                && run.trial == trial.id
                && run.audio == trial.input.audio.original.sha256
                && run.reference == trial.input.reference.sha256,
        )?;
        let result = run
            .result
            .as_ref()
            .map(|reference| {
                let result: AdviceResult = self.document(reference, DOCUMENT_CAP)?;
                check_advice_result(trial, &run, &result)?;
                Ok(result)
            })
            .transpose()?;
        Ok(AdviceView { run, result })
    }

    pub(super) fn reflection(
        &mut self,
        trial: &TrialRevision,
        reference: &BlobRef,
    ) -> Result<Reflection, HistoryError> {
        related(trial.reflections.contains(reference))?;
        let reflection: Reflection = self.document(reference, DOCUMENT_CAP)?;
        related(reflection.trial == trial.id)?;
        Ok(reflection)
    }
}

// Payload relation checks are also used before the writer publishes new blobs.
// DTO validation, Run identity and membership remain the caller's responsibility.
pub(super) fn check_analysis_result(
    trial: &TrialRevision,
    run: &AnalysisRun,
    report: &AnalysisReport,
) -> Result<(), HistoryError> {
    related(
        report.profile == run.profile
            && report.sample_rate == trial.input.audio.rate
            && report.channels == trial.input.audio.channels
            && report.input_frames == trial.input.audio.frames
            && matches!(
                (run.state, report.status),
                (AnalysisRunState::Complete, AnalysisStatus::Complete)
                    | (AnalysisRunState::Partial, AnalysisStatus::Partial)
            ),
    )
}

pub(super) fn check_advice_result(
    trial: &TrialRevision,
    run: &AdviceRun,
    result: &AdviceResult,
) -> Result<(), HistoryError> {
    related(matches!(
        (run.state, result.assessment),
        (AdviceRunState::Assessed, AdviceAssessment::Assessed)
            | (
                AdviceRunState::ReferenceMismatch,
                AdviceAssessment::ReferenceMismatch
            )
            | (AdviceRunState::Unassessable, AdviceAssessment::Unassessable)
    ))?;
    for improvement in &result.improvements {
        related(
            trial
                .input
                .reference
                .text
                .contains(&improvement.reference_excerpt),
        )?;
    }
    Ok(())
}
