//! Builds an `EvidenceBundle` from one or more `(SourceRecord,
//! SourceUseDecision)` pairs by running each through
//! `market_intelligence_source_policy::evaluate_record` — the same
//! per-record logic `market-intelligence-server`'s demo binary previously
//! did inline for exactly one record. Generalized here to N records and
//! given a real canonical hash (`market_intelligence_core::canonical_hash`)
//! so the resulting bundle actually satisfies
//! `evidence-bundle.schema.json`'s `canonical-hash` field, not a
//! placeholder string.

use market_intelligence_core::{
    BundlePurpose, EvidenceBundle, EvidenceRecordRef, SourceRecord, SourceUseDecision, UtcTimestamp,
};

pub struct RecordAndDecision {
    pub record: SourceRecord,
    pub decision: SourceUseDecision,
}

/// Runs `evaluate_record` against every `(record, decision)` pair, marking
/// each `EvidenceRecordRef::execution_eligible` accordingly.
/// `quarantine_count` counts records that fail eligibility specifically
/// for a `LiveExecution` bundle purpose (the schema's own meaning of
/// "quarantined" — see `evidence-bundle.schema.json`); a `Research`/
/// `Advisory`/`PaperExecution` bundle can still legitimately contain
/// records that would fail a live-execution check without those being
/// "quarantined" in that sense.
pub fn build_evidence_bundle(
    evidence_bundle_id: &str,
    items: &[RecordAndDecision],
    purpose: BundlePurpose,
    decision_time: UtcTimestamp,
    now: UtcTimestamp,
) -> EvidenceBundle {
    let mut refs = Vec::with_capacity(items.len());
    let mut quarantine_count = 0u32;

    for item in items {
        let eligible =
            market_intelligence_source_policy::evaluate_record(&item.record, &item.decision, purpose, decision_time).is_ok();
        if !eligible && purpose == BundlePurpose::LiveExecution {
            quarantine_count += 1;
        }
        refs.push(EvidenceRecordRef {
            source_record_id: item.record.source_record_id.clone(),
            content_hash: item.record.content_hash.clone(),
            source_use_decision_id: Some(item.decision.source_id.clone()),
            mnpi_classification: item.record.mnpi_classification,
            public_availability_time: item.record.public_availability_time,
            freshness_seconds: item.record.staleness_seconds,
            execution_eligible: eligible,
        });
    }

    let all_records_execution_eligible = !refs.is_empty() && refs.iter().all(|r| r.execution_eligible);
    let canonical_hash = compute_canonical_hash(evidence_bundle_id, purpose, &refs);

    EvidenceBundle {
        evidence_bundle_id: evidence_bundle_id.to_string(),
        created_at: now,
        decision_time: Some(decision_time),
        records: refs,
        bundle_purpose: purpose,
        all_records_execution_eligible,
        quarantine_count,
        service_attestation: None,
        canonical_hash,
    }
}

fn compute_canonical_hash(evidence_bundle_id: &str, purpose: BundlePurpose, refs: &[EvidenceRecordRef]) -> String {
    let payload = serde_json::json!({
        "evidence-bundle-id": evidence_bundle_id,
        "bundle-purpose": purpose,
        "records": refs,
    });
    let body = serde_json::to_string(&payload).expect("Value serialization is infallible");
    market_intelligence_core::canonical_hash(body.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    use market_intelligence_core::{CorrectionState, MnpiClassification, SourceType, UsePermission};

    fn record(id: &str, mnpi: market_intelligence_core::MnpiClassification, execution_eligible: bool) -> SourceRecord {
        SourceRecord {
            schema_version: "2.0.0".into(),
            source_record_id: id.into(),
            source_id: "fixture-source".into(),
            provider_record_id: None,
            source_type: SourceType::OfficialFiling,
            jurisdiction: Some("US".into()),
            retrieval_method: "fixture".into(),
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
            source_id: "fixture-source".into(),
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
    fn research_only_source_produces_a_research_eligible_bundle() {
        let items = vec![RecordAndDecision {
            record: record("rec-1", MnpiClassification::PublicButUseRestricted, true),
            decision: decision(UsePermission::ResearchOnly),
        }];
        let bundle = build_evidence_bundle("bundle-1", &items, BundlePurpose::Research, UtcTimestamp::UNIX_EPOCH, UtcTimestamp::UNIX_EPOCH);
        assert!(bundle.all_records_execution_eligible);
        assert_eq!(bundle.quarantine_count, 0);
        assert!(bundle.canonical_hash.starts_with("sha256:"));
    }

    #[test]
    fn research_only_source_is_quarantined_for_a_live_execution_bundle() {
        let items = vec![RecordAndDecision {
            record: record("rec-1", MnpiClassification::PublicConfirmed, true),
            decision: decision(UsePermission::ResearchOnly),
        }];
        let bundle = build_evidence_bundle("bundle-1", &items, BundlePurpose::LiveExecution, UtcTimestamp::UNIX_EPOCH, UtcTimestamp::UNIX_EPOCH);
        assert!(!bundle.all_records_execution_eligible);
        assert_eq!(bundle.quarantine_count, 1);
        assert!(!bundle.records[0].execution_eligible);
    }

    #[test]
    fn mixed_eligibility_records_are_reflected_individually() {
        let items = vec![
            RecordAndDecision { record: record("rec-eligible", MnpiClassification::PublicConfirmed, true), decision: decision(UsePermission::Allowed) },
            RecordAndDecision { record: record("rec-ineligible", MnpiClassification::SuspectedNonpublic, true), decision: decision(UsePermission::Allowed) },
        ];
        let bundle = build_evidence_bundle("bundle-1", &items, BundlePurpose::LiveExecution, UtcTimestamp::UNIX_EPOCH, UtcTimestamp::UNIX_EPOCH);
        assert!(!bundle.all_records_execution_eligible);
        assert_eq!(bundle.quarantine_count, 1);
        assert!(bundle.records[0].execution_eligible);
        assert!(!bundle.records[1].execution_eligible);
    }

    #[test]
    fn empty_items_produce_an_empty_never_eligible_bundle() {
        let bundle = build_evidence_bundle("bundle-empty", &[], BundlePurpose::Research, UtcTimestamp::UNIX_EPOCH, UtcTimestamp::UNIX_EPOCH);
        assert!(bundle.records.is_empty());
        assert!(!bundle.all_records_execution_eligible);
        assert_eq!(bundle.quarantine_count, 0);
    }

    #[test]
    fn canonical_hash_is_deterministic_for_the_same_inputs() {
        let items = vec![RecordAndDecision {
            record: record("rec-1", MnpiClassification::PublicConfirmed, true),
            decision: decision(UsePermission::Allowed),
        }];
        let a = build_evidence_bundle("bundle-1", &items, BundlePurpose::LiveExecution, UtcTimestamp::UNIX_EPOCH, UtcTimestamp::UNIX_EPOCH);
        let b = build_evidence_bundle("bundle-1", &items, BundlePurpose::LiveExecution, UtcTimestamp::UNIX_EPOCH, UtcTimestamp::UNIX_EPOCH);
        assert_eq!(a.canonical_hash, b.canonical_hash);
    }

    #[test]
    fn canonical_hash_differs_for_a_different_bundle_id() {
        let items = vec![RecordAndDecision {
            record: record("rec-1", MnpiClassification::PublicConfirmed, true),
            decision: decision(UsePermission::Allowed),
        }];
        let a = build_evidence_bundle("bundle-1", &items, BundlePurpose::LiveExecution, UtcTimestamp::UNIX_EPOCH, UtcTimestamp::UNIX_EPOCH);
        let b = build_evidence_bundle("bundle-2", &items, BundlePurpose::LiveExecution, UtcTimestamp::UNIX_EPOCH, UtcTimestamp::UNIX_EPOCH);
        assert_ne!(a.canonical_hash, b.canonical_hash);
    }
}
