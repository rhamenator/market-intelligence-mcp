//! Versioned, testable signal plugins over normalized evidence.
//!
//! Status: mock. `NoActionBaseline` is the only implemented signal today,
//! and it exists to prove the `SignalPlugin` shape and the "no signal is
//! forced into buy or sell" requirement end-to-end against fixtures. Real
//! signal families (whale-follow, whale-fade, insider-cluster, etc.) are
//! Phase 2 continuation / Phase 5 work — see
//! `06-implementation-order-and-acceptance.md`.

use market_intelligence_core::LegalStatus;
use market_intelligence_core::{EvidenceBundle, SignalAssessment, SignalDirection, SignalType};

pub trait SignalPlugin {
    fn signal_type(&self) -> SignalType;
    fn evaluate(&self, bundle: &EvidenceBundle) -> SignalAssessment;
}

/// Deterministic baseline that always emits `no-action` with an explicit
/// `no_action_score` of 1.0. Useful as a fixture-testable floor: any real
/// signal must be measurably better than always saying nothing.
pub struct NoActionBaseline;

impl SignalPlugin for NoActionBaseline {
    fn signal_type(&self) -> SignalType {
        SignalType::MarketRegime
    }

    fn evaluate(&self, bundle: &EvidenceBundle) -> SignalAssessment {
        SignalAssessment {
            signal_id: format!("no-action-baseline:{}", bundle.evidence_bundle_id),
            signal_type: SignalType::MarketRegime,
            instrument_ids: vec![],
            actor_ids: vec![],
            direction: SignalDirection::Uncertain,
            score: 0.0,
            confidence: 0.0,
            follow_score: Some(0.0),
            fade_score: Some(0.0),
            no_action_score: Some(1.0),
            horizon_seconds: None,
            signal_half_life_seconds: None,
            evidence_bundle_id: bundle.evidence_bundle_id.clone(),
            evidence_ids: vec![],
            model_uri: "heuristic://no-action-baseline".into(),
            legal_status: LegalStatus::Observed,
            intent_inferred: false,
            limitations: vec!["baseline placeholder; no real signal logic yet".into()],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use market_intelligence_core::BundlePurpose;
    use time::OffsetDateTime;

    #[test]
    fn no_action_baseline_never_recommends_direction() {
        let bundle = EvidenceBundle {
            evidence_bundle_id: "b1".into(),
            created_at: OffsetDateTime::UNIX_EPOCH,
            decision_time: None,
            records: vec![],
            bundle_purpose: BundlePurpose::Research,
            all_records_execution_eligible: false,
            quarantine_count: 0,
            service_attestation: None,
            canonical_hash: "sha256:0".into(),
        };
        let assessment = NoActionBaseline.evaluate(&bundle);
        assert_eq!(assessment.no_action_score, Some(1.0));
        assert!(!assessment.intent_inferred);
    }
}
