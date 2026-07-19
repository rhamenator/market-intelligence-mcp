//! Deterministic source-use and MNPI gating engine.
//!
//! Per `05-source-policy-and-legal-boundaries.md` in the originating prompt
//! bundle: "The system must never ask a model to decide whether
//! questionable information is legally tradable. The deterministic
//! source-policy engine makes the gating decision." This crate is that
//! engine. It has no model dependency and no network access.

use market_intelligence_core::{BundlePurpose, MnpiClassification, SourceRecord, SourceUseDecision};
use time::OffsetDateTime;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum PolicyDenialReason {
    #[error("mnpi classification {0:?} is not execution-eligible")]
    MnpiNotExecutionEligible(MnpiClassification),
    #[error("source-use decision does not permit trading use for the requested purpose")]
    TradingUseNotPermitted,
    #[error("record is not marked execution-eligible")]
    RecordNotExecutionEligible,
    #[error("public-availability-time is missing or after the decision time")]
    NotYetPubliclyAvailable,
    #[error("record is stale beyond the source's expected reporting lag policy")]
    StaleBeyondPolicy,
}

/// Evaluates one `SourceRecord` against its matching `SourceUseDecision` and
/// the purpose the caller intends to use it for (research, advisory, paper,
/// or live execution). Returns `Ok(())` when permitted, or the first
/// disqualifying `PolicyDenialReason` otherwise. Never partially applies —
/// a single failing check fails the whole record for that purpose.
pub fn evaluate_record(
    record: &SourceRecord,
    decision: &SourceUseDecision,
    purpose: BundlePurpose,
    decision_time: OffsetDateTime,
) -> Result<(), PolicyDenialReason> {
    if !record.mnpi_classification.is_execution_eligible_class() && purpose != BundlePurpose::Research
    {
        return Err(PolicyDenialReason::MnpiNotExecutionEligible(
            record.mnpi_classification,
        ));
    }

    if matches!(purpose, BundlePurpose::PaperExecution | BundlePurpose::LiveExecution)
        && !decision.permits_live_execution()
    {
        return Err(PolicyDenialReason::TradingUseNotPermitted);
    }

    if purpose == BundlePurpose::LiveExecution && !record.execution_eligible {
        return Err(PolicyDenialReason::RecordNotExecutionEligible);
    }

    if let Some(public_time) = record.public_availability_time {
        if public_time > decision_time {
            return Err(PolicyDenialReason::NotYetPubliclyAvailable);
        }
    } else if purpose == BundlePurpose::LiveExecution {
        // No verified public-availability time and the caller wants to
        // trade on it: fail closed rather than assume it was always public.
        return Err(PolicyDenialReason::NotYetPubliclyAvailable);
    }

    if let (Some(observed_lag), Some(expected_lag)) = (
        record.staleness_seconds,
        record.expected_reporting_lag_seconds,
    ) {
        if purpose == BundlePurpose::LiveExecution && observed_lag > expected_lag.saturating_mul(3) {
            return Err(PolicyDenialReason::StaleBeyondPolicy);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use market_intelligence_core::{CorrectionState, SourceType, UsePermission};

    fn record(mnpi: MnpiClassification, execution_eligible: bool) -> SourceRecord {
        SourceRecord {
            schema_version: "2.0.0".into(),
            source_record_id: "rec-1".into(),
            source_id: "us-oge-public-financial-disclosures".into(),
            provider_record_id: None,
            source_type: SourceType::OfficialFiling,
            jurisdiction: Some("US".into()),
            retrieval_method: "api".into(),
            transaction_time: Some(OffsetDateTime::UNIX_EPOCH),
            event_time: None,
            filing_time: None,
            published_time: None,
            public_availability_time: Some(OffsetDateTime::UNIX_EPOCH),
            retrieved_time: OffsetDateTime::UNIX_EPOCH,
            corrected_time: None,
            content_hash: None,
            execution_eligible,
            redistribution_eligible: false,
            quality: Some(0.9),
            staleness_seconds: Some(60),
            expected_reporting_lag_seconds: Some(120),
            revision: 1,
            correction_state: CorrectionState::Original,
            untrusted_text: false,
            mnpi_classification: mnpi,
        }
    }

    fn decision(trading_use: UsePermission) -> SourceUseDecision {
        SourceUseDecision {
            source_id: "us-oge-public-financial-disclosures".into(),
            policy_version: "2026-07-19.1".into(),
            public_access: true,
            automated_retrieval: UsePermission::LegalReviewRequired,
            commercial_use: UsePermission::ProhibitedOrUnclear,
            trading_use,
            redistribution: UsePermission::ProhibitedOrUnclear,
            model_training: None,
            retention_days: None,
            attribution_required: true,
            terms_snapshot_hash: None,
            robots_snapshot_hash: None,
            contract_id: None,
            reviewed_at: OffsetDateTime::UNIX_EPOCH,
            reviewed_by: "operator".into(),
            reason_codes: vec![],
        }
    }

    #[test]
    fn research_only_source_cannot_back_a_live_execution_bundle() {
        let r = record(MnpiClassification::PublicConfirmed, true);
        let d = decision(UsePermission::ResearchOnly);
        let result = evaluate_record(&r, &d, BundlePurpose::LiveExecution, OffsetDateTime::UNIX_EPOCH);
        assert_eq!(result, Err(PolicyDenialReason::TradingUseNotPermitted));
    }

    #[test]
    fn research_only_source_is_fine_for_research_purpose() {
        let r = record(MnpiClassification::PublicButUseRestricted, true);
        let d = decision(UsePermission::ResearchOnly);
        let result = evaluate_record(&r, &d, BundlePurpose::Research, OffsetDateTime::UNIX_EPOCH);
        assert_eq!(result, Ok(()));
    }

    #[test]
    fn suspected_nonpublic_record_is_quarantined_even_with_allowed_decision() {
        let r = record(MnpiClassification::SuspectedNonpublic, true);
        let d = decision(UsePermission::Allowed);
        let result = evaluate_record(&r, &d, BundlePurpose::PaperExecution, OffsetDateTime::UNIX_EPOCH);
        assert_eq!(
            result,
            Err(PolicyDenialReason::MnpiNotExecutionEligible(
                MnpiClassification::SuspectedNonpublic
            ))
        );
    }

    #[test]
    fn future_public_availability_time_is_rejected() {
        let mut r = record(MnpiClassification::PublicConfirmed, true);
        r.public_availability_time = Some(OffsetDateTime::UNIX_EPOCH + time::Duration::days(1));
        let d = decision(UsePermission::Allowed);
        let result = evaluate_record(&r, &d, BundlePurpose::LiveExecution, OffsetDateTime::UNIX_EPOCH);
        assert_eq!(result, Err(PolicyDenialReason::NotYetPubliclyAvailable));
    }

    #[test]
    fn allowed_and_fresh_and_public_permits_live_execution() {
        let r = record(MnpiClassification::PublicConfirmed, true);
        let d = decision(UsePermission::Allowed);
        let result = evaluate_record(&r, &d, BundlePurpose::LiveExecution, OffsetDateTime::UNIX_EPOCH);
        assert_eq!(result, Ok(()));
    }
}
