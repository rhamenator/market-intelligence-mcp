use serde::{Deserialize, Serialize};

use crate::utc_timestamp::UtcTimestamp;

/// Normalized use-permission value for one dimension of a `SourceUseDecision`
/// (automated retrieval, commercial use, trading use, redistribution,
/// model training). Unknown or contradictory policy fails closed for live
/// execution — see the source-policy engine in
/// `market_intelligence_source_policy`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum UsePermission {
    Allowed,
    AllowedWithAttribution,
    AllowedUnderContract,
    ResearchOnly,
    RedistributionProhibited,
    ExecutionProhibited,
    LegalReviewRequired,
    Unknown,
    Disabled,
    ProhibitedOrUnclear,
}

/// Versioned, per-source legal/licensing decision. A record is
/// execution-eligible only when every relevant use decision permits the
/// intended use; the deterministic engine decides this, never a model.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct SourceUseDecision {
    pub source_id: String,
    pub policy_version: String,
    pub public_access: bool,
    pub automated_retrieval: UsePermission,
    pub commercial_use: UsePermission,
    pub trading_use: UsePermission,
    pub redistribution: UsePermission,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_training: Option<UsePermission>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retention_days: Option<u32>,
    pub attribution_required: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub terms_snapshot_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub robots_snapshot_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contract_id: Option<String>,
    pub reviewed_at: UtcTimestamp,
    pub reviewed_by: String,
    #[serde(default)]
    pub reason_codes: Vec<String>,
}

impl SourceUseDecision {
    /// Whether this source-use decision permits an evidence record from it
    /// to become part of a live-execution `EvidenceBundle`. Unknown,
    /// disabled, or research-only values fail closed rather than silently
    /// defaulting to allowed.
    pub fn permits_live_execution(&self) -> bool {
        matches!(
            self.trading_use,
            UsePermission::Allowed
                | UsePermission::AllowedWithAttribution
                | UsePermission::AllowedUnderContract
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_decision(trading_use: UsePermission) -> SourceUseDecision {
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
            reason_codes: vec!["statutory-commercial-use-restriction".into()],
        }
    }

    #[test]
    fn research_only_does_not_permit_live_execution() {
        let d = base_decision(UsePermission::ResearchOnly);
        assert!(!d.permits_live_execution());
    }

    #[test]
    fn allowed_permits_live_execution() {
        let d = base_decision(UsePermission::Allowed);
        assert!(d.permits_live_execution());
    }

    #[test]
    fn unknown_fails_closed() {
        let d = base_decision(UsePermission::Unknown);
        assert!(!d.permits_live_execution());
    }
}
