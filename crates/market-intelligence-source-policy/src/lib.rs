//! Deterministic source-use and MNPI gating engine.
//!
//! Per `05-source-policy-and-legal-boundaries.md` in the originating prompt
//! bundle: "The system must never ask a model to decide whether
//! questionable information is legally tradable. The deterministic
//! source-policy engine makes the gating decision." This crate is that
//! engine. It has no model dependency, no network access, and (by policy —
//! see the workspace `[workspace.lints]`) no `unsafe` code.

use std::fmt;

use market_intelligence_core::{
    BundlePurpose, MnpiClassification, SourceRecord, SourceUseDecision, UtcTimestamp,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolicyDenialReason {
    MnpiNotExecutionEligible(MnpiClassification),
    TradingUseNotPermitted,
    RecordNotExecutionEligible,
    NotYetPubliclyAvailable,
    StaleBeyondPolicy,
}

impl fmt::Display for PolicyDenialReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MnpiNotExecutionEligible(class) => {
                write!(f, "mnpi classification {class:?} is not execution-eligible")
            }
            Self::TradingUseNotPermitted => {
                write!(f, "source-use decision does not permit trading use for the requested purpose")
            }
            Self::RecordNotExecutionEligible => write!(f, "record is not marked execution-eligible"),
            Self::NotYetPubliclyAvailable => {
                write!(f, "public-availability-time is missing or after the decision time")
            }
            Self::StaleBeyondPolicy => {
                write!(f, "record is stale beyond the source's expected reporting lag policy")
            }
        }
    }
}

impl std::error::Error for PolicyDenialReason {}

/// Evaluates one `SourceRecord` against its matching `SourceUseDecision` and
/// the purpose the caller intends to use it for (research, advisory, paper,
/// or live execution). Returns `Ok(())` when permitted, or the first
/// disqualifying `PolicyDenialReason` otherwise. Never partially applies —
/// a single failing check fails the whole record for that purpose.
pub fn evaluate_record(
    record: &SourceRecord,
    decision: &SourceUseDecision,
    purpose: BundlePurpose,
    decision_time: UtcTimestamp,
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

    if let (Some(observed_lag), Some(expected_lag)) =
        (record.staleness_seconds, record.expected_reporting_lag_seconds)
        && purpose == BundlePurpose::LiveExecution
        && observed_lag > expected_lag.saturating_mul(3)
    {
        return Err(PolicyDenialReason::StaleBeyondPolicy);
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
            transaction_time: Some(UtcTimestamp::UNIX_EPOCH),
            event_time: None,
            filing_time: None,
            published_time: None,
            public_availability_time: Some(UtcTimestamp::UNIX_EPOCH),
            retrieved_time: UtcTimestamp::UNIX_EPOCH,
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
            reviewed_at: UtcTimestamp::UNIX_EPOCH,
            reviewed_by: "operator".into(),
            reason_codes: vec![],
        }
    }

    #[test]
    fn research_only_source_cannot_back_a_live_execution_bundle() {
        let r = record(MnpiClassification::PublicConfirmed, true);
        let d = decision(UsePermission::ResearchOnly);
        let result = evaluate_record(&r, &d, BundlePurpose::LiveExecution, UtcTimestamp::UNIX_EPOCH);
        assert_eq!(result, Err(PolicyDenialReason::TradingUseNotPermitted));
    }

    #[test]
    fn research_only_source_is_fine_for_research_purpose() {
        let r = record(MnpiClassification::PublicButUseRestricted, true);
        let d = decision(UsePermission::ResearchOnly);
        let result = evaluate_record(&r, &d, BundlePurpose::Research, UtcTimestamp::UNIX_EPOCH);
        assert_eq!(result, Ok(()));
    }

    #[test]
    fn suspected_nonpublic_record_is_quarantined_even_with_allowed_decision() {
        let r = record(MnpiClassification::SuspectedNonpublic, true);
        let d = decision(UsePermission::Allowed);
        let result = evaluate_record(&r, &d, BundlePurpose::PaperExecution, UtcTimestamp::UNIX_EPOCH);
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
        r.public_availability_time = UtcTimestamp::UNIX_EPOCH.checked_add_days(1);
        let d = decision(UsePermission::Allowed);
        let result = evaluate_record(&r, &d, BundlePurpose::LiveExecution, UtcTimestamp::UNIX_EPOCH);
        assert_eq!(result, Err(PolicyDenialReason::NotYetPubliclyAvailable));
    }

    #[test]
    fn allowed_and_fresh_and_public_permits_live_execution() {
        let r = record(MnpiClassification::PublicConfirmed, true);
        let d = decision(UsePermission::Allowed);
        let result = evaluate_record(&r, &d, BundlePurpose::LiveExecution, UtcTimestamp::UNIX_EPOCH);
        assert_eq!(result, Ok(()));
    }

    #[test]
    fn stale_beyond_policy_blocks_live_execution() {
        let mut r = record(MnpiClassification::PublicConfirmed, true);
        r.staleness_seconds = Some(1_000);
        r.expected_reporting_lag_seconds = Some(100); // 10x expected lag
        let d = decision(UsePermission::Allowed);
        let result = evaluate_record(&r, &d, BundlePurpose::LiveExecution, UtcTimestamp::UNIX_EPOCH);
        assert_eq!(result, Err(PolicyDenialReason::StaleBeyondPolicy));
    }

    #[test]
    fn stale_beyond_policy_does_not_block_research_purpose() {
        let mut r = record(MnpiClassification::PublicButUseRestricted, true);
        r.staleness_seconds = Some(1_000_000);
        r.expected_reporting_lag_seconds = Some(1);
        let d = decision(UsePermission::ResearchOnly);
        let result = evaluate_record(&r, &d, BundlePurpose::Research, UtcTimestamp::UNIX_EPOCH);
        assert_eq!(result, Ok(()));
    }

    #[test]
    fn missing_public_availability_time_blocks_live_execution_even_if_flagged_eligible() {
        let mut r = record(MnpiClassification::PublicConfirmed, true);
        r.public_availability_time = None;
        let d = decision(UsePermission::Allowed);
        let result = evaluate_record(&r, &d, BundlePurpose::LiveExecution, UtcTimestamp::UNIX_EPOCH);
        assert_eq!(result, Err(PolicyDenialReason::NotYetPubliclyAvailable));
    }

    #[test]
    fn record_not_flagged_execution_eligible_blocks_live_even_with_allowed_decision() {
        let r = record(MnpiClassification::PublicConfirmed, false);
        let d = decision(UsePermission::Allowed);
        let result = evaluate_record(&r, &d, BundlePurpose::LiveExecution, UtcTimestamp::UNIX_EPOCH);
        assert_eq!(result, Err(PolicyDenialReason::RecordNotExecutionEligible));
    }

    /// Every `MnpiClassification` other than `PublicConfirmed` must be
    /// rejected for every non-research purpose — this is the exhaustive
    /// version of `suspected_nonpublic_record_is_quarantined...` above,
    /// covering every variant so a newly added classification is forced to
    /// be considered here rather than silently defaulting to "allowed".
    #[test]
    fn every_non_public_confirmed_classification_is_rejected_for_every_non_research_purpose() {
        let all_classes = [
            MnpiClassification::PublicConfirmed,
            MnpiClassification::PublicButUseRestricted,
            MnpiClassification::PublicationStatusUncertain,
            MnpiClassification::AmbiguousOrigin,
            MnpiClassification::SuspectedNonpublic,
            MnpiClassification::KnownNonpublic,
            MnpiClassification::ProhibitedSource,
        ];
        let purposes = [
            BundlePurpose::Advisory,
            BundlePurpose::PaperExecution,
            BundlePurpose::LiveExecution,
        ];
        for class in all_classes {
            let r = record(class, true);
            let d = decision(UsePermission::Allowed);
            for purpose in purposes {
                let result = evaluate_record(&r, &d, purpose, UtcTimestamp::UNIX_EPOCH);
                if class == MnpiClassification::PublicConfirmed {
                    assert_eq!(
                        result,
                        Ok(()),
                        "a well-formed PublicConfirmed record should pass for {purpose:?}"
                    );
                } else {
                    assert_eq!(
                        result,
                        Err(PolicyDenialReason::MnpiNotExecutionEligible(class)),
                        "{class:?} should be rejected for {purpose:?}"
                    );
                }
            }
        }
    }

    /// A tiny, dependency-free xorshift64 PRNG — see
    /// `market_intelligence_core::utc_timestamp` tests for the rationale.
    struct XorShift64(u64);
    impl XorShift64 {
        fn next(&mut self) -> u64 {
            let mut x = self.0;
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            self.0 = x;
            x
        }
        fn choice<T: Copy>(&mut self, options: &[T]) -> T {
            options[(self.next() as usize) % options.len()]
        }
    }

    /// Feeds `evaluate_record` thousands of random-but-well-typed
    /// combinations of MNPI classification, use-permission, purpose,
    /// eligibility flag, and staleness — the function must never panic,
    /// and the invariant "only `PublicConfirmed` clears MNPI for a
    /// non-research purpose" must hold on every draw.
    #[test]
    fn fuzz_smoke_evaluate_record_never_panics_and_respects_mnpi_invariant() {
        let mnpi_options = [
            MnpiClassification::PublicConfirmed,
            MnpiClassification::PublicButUseRestricted,
            MnpiClassification::PublicationStatusUncertain,
            MnpiClassification::AmbiguousOrigin,
            MnpiClassification::SuspectedNonpublic,
            MnpiClassification::KnownNonpublic,
            MnpiClassification::ProhibitedSource,
        ];
        let use_options = [
            UsePermission::Allowed,
            UsePermission::AllowedWithAttribution,
            UsePermission::AllowedUnderContract,
            UsePermission::ResearchOnly,
            UsePermission::RedistributionProhibited,
            UsePermission::ExecutionProhibited,
            UsePermission::LegalReviewRequired,
            UsePermission::Unknown,
            UsePermission::Disabled,
            UsePermission::ProhibitedOrUnclear,
        ];
        let purpose_options = [
            BundlePurpose::Research,
            BundlePurpose::Advisory,
            BundlePurpose::PaperExecution,
            BundlePurpose::LiveExecution,
        ];

        let mut rng = XorShift64(0xA24BAED4963EE407);
        for _ in 0..20_000 {
            let mnpi = rng.choice(&mnpi_options);
            let trading_use = rng.choice(&use_options);
            let purpose = rng.choice(&purpose_options);
            let execution_eligible = rng.next() % 2 == 0;
            let has_public_time = rng.next() % 2 == 0;
            let staleness = (rng.next() % 100_000) as i64;
            let expected_lag = (rng.next() % 100_000) as i64;

            let mut r = record(mnpi, execution_eligible);
            r.public_availability_time = if has_public_time {
                Some(UtcTimestamp::UNIX_EPOCH)
            } else {
                None
            };
            r.staleness_seconds = Some(staleness);
            r.expected_reporting_lag_seconds = Some(expected_lag);
            let d = decision(trading_use);

            let result = evaluate_record(&r, &d, purpose, UtcTimestamp::UNIX_EPOCH);

            if mnpi != MnpiClassification::PublicConfirmed && purpose != BundlePurpose::Research {
                assert_eq!(
                    result,
                    Err(PolicyDenialReason::MnpiNotExecutionEligible(mnpi)),
                    "MNPI invariant violated for mnpi={mnpi:?} purpose={purpose:?}"
                );
            }
        }
    }
}
