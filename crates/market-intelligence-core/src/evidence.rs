use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::legal_status::MnpiClassification;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BundlePurpose {
    Research,
    Advisory,
    PaperExecution,
    LiveExecution,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct EvidenceRecordRef {
    pub source_record_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_use_decision_id: Option<String>,
    pub mnpi_classification: MnpiClassification,
    #[serde(default, with = "time::serde::rfc3339::option")]
    pub public_availability_time: Option<OffsetDateTime>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub freshness_seconds: Option<i64>,
    pub execution_eligible: bool,
}

/// Immutable, signed/attested bundle of evidence backing a signal or trade
/// intent. `trade-guard-mcp` independently verifies this before any live
/// intent is authorized — see
/// `market-system-contracts/schemas/2.0.0/evidence-bundle.schema.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct EvidenceBundle {
    pub evidence_bundle_id: String,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(default, with = "time::serde::rfc3339::option")]
    pub decision_time: Option<OffsetDateTime>,
    pub records: Vec<EvidenceRecordRef>,
    pub bundle_purpose: BundlePurpose,
    pub all_records_execution_eligible: bool,
    pub quarantine_count: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub service_attestation: Option<String>,
    pub canonical_hash: String,
}

impl EvidenceBundle {
    /// A bundle may back a live-execution intent only if it was built for
    /// that purpose, contains zero quarantined records, and every record
    /// is independently execution-eligible. `trade-guard-mcp` must reject
    /// otherwise — this mirrors the check it performs on its side, so the
    /// intelligence service cannot even emit an inconsistent bundle.
    pub fn is_live_execution_eligible(&self) -> bool {
        self.bundle_purpose == BundlePurpose::LiveExecution
            && self.all_records_execution_eligible
            && self.quarantine_count == 0
            && !self.records.is_empty()
            && self.records.iter().all(|r| r.execution_eligible)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eligible_record() -> EvidenceRecordRef {
        EvidenceRecordRef {
            source_record_id: "rec-1".into(),
            content_hash: None,
            source_use_decision_id: None,
            mnpi_classification: MnpiClassification::PublicConfirmed,
            public_availability_time: Some(OffsetDateTime::UNIX_EPOCH),
            freshness_seconds: Some(60),
            execution_eligible: true,
        }
    }

    #[test]
    fn research_bundle_is_never_live_eligible() {
        let bundle = EvidenceBundle {
            evidence_bundle_id: "b1".into(),
            created_at: OffsetDateTime::UNIX_EPOCH,
            decision_time: None,
            records: vec![eligible_record()],
            bundle_purpose: BundlePurpose::Research,
            all_records_execution_eligible: true,
            quarantine_count: 0,
            service_attestation: None,
            canonical_hash: "sha256:0".into(),
        };
        assert!(!bundle.is_live_execution_eligible());
    }

    #[test]
    fn quarantined_record_blocks_live_eligibility() {
        let mut bundle = EvidenceBundle {
            evidence_bundle_id: "b2".into(),
            created_at: OffsetDateTime::UNIX_EPOCH,
            decision_time: None,
            records: vec![eligible_record()],
            bundle_purpose: BundlePurpose::LiveExecution,
            all_records_execution_eligible: true,
            quarantine_count: 1,
            service_attestation: Some("sig".into()),
            canonical_hash: "sha256:0".into(),
        };
        assert!(!bundle.is_live_execution_eligible());
        bundle.quarantine_count = 0;
        assert!(bundle.is_live_execution_eligible());
    }
}
