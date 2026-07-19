use serde::{Deserialize, Serialize};

use crate::legal_status::MnpiClassification;
use crate::utc_timestamp::UtcTimestamp;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SourceType {
    Api,
    Mcp,
    BulkFile,
    OfficialFiling,
    Rss,
    LicensedFeed,
    WebPage,
    ManualPublicRecord,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CorrectionState {
    Original,
    Supersedes,
    Amends,
    Corrects,
    Withdraws,
    ProviderRetracted,
    ParserCorrected,
    EntityResolutionCorrected,
}

/// Immutable, append-only provenance record for a single retrieved item.
/// Corrections create a new revision; they never overwrite history — a
/// historical backtest must see the version that was public at the
/// simulated time.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct SourceRecord {
    pub schema_version: String,
    pub source_record_id: String,
    pub source_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_record_id: Option<String>,
    pub source_type: SourceType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub jurisdiction: Option<String>,
    pub retrieval_method: String,
    #[serde(default)]
    pub transaction_time: Option<UtcTimestamp>,
    #[serde(default)]
    pub event_time: Option<UtcTimestamp>,
    #[serde(default)]
    pub filing_time: Option<UtcTimestamp>,
    #[serde(default)]
    pub published_time: Option<UtcTimestamp>,
    #[serde(default)]
    pub public_availability_time: Option<UtcTimestamp>,
    pub retrieved_time: UtcTimestamp,
    #[serde(default)]
    pub corrected_time: Option<UtcTimestamp>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_hash: Option<String>,
    pub execution_eligible: bool,
    pub redistribution_eligible: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quality: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub staleness_seconds: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_reporting_lag_seconds: Option<i64>,
    pub revision: u32,
    pub correction_state: CorrectionState,
    pub untrusted_text: bool,
    pub mnpi_classification: MnpiClassification,
}

impl SourceRecord {
    /// Age of this record at `now`, using `retrieved_time` as the fallback
    /// clock if `public_availability_time` is absent. Backtests and
    /// autonomous decisions must use `public-availability-time`, not
    /// transaction time or later-corrected knowledge.
    pub fn age_seconds(&self, now: UtcTimestamp) -> i64 {
        let anchor = self.public_availability_time.unwrap_or(self.retrieved_time);
        now.saturating_diff_seconds(&anchor).max(0)
    }
}
