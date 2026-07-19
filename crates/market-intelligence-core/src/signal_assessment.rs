use serde::{Deserialize, Serialize};

use crate::legal_status::LegalStatus;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SignalType {
    PossibleInformedTrading,
    WhaleFollow,
    WhaleFade,
    PromotionDistributionRisk,
    IssuerBuybackSupport,
    DilutionRisk,
    InsiderCluster,
    ActivistAccumulation,
    PoliticalPolicyExposure,
    CoordinatedPromotionRisk,
    MarketRegime,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SignalDirection {
    Bullish,
    Bearish,
    VolatilityUp,
    VolatilityDown,
    Neutral,
    Uncertain,
}

/// Output of every heuristic, statistical model, and LLM-derived
/// assessment. Must always carry a `no_action_score` — no signal is forced
/// into buy or sell.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct SignalAssessment {
    pub signal_id: String,
    pub signal_type: SignalType,
    #[serde(default)]
    pub instrument_ids: Vec<String>,
    #[serde(default)]
    pub actor_ids: Vec<String>,
    pub direction: SignalDirection,
    pub score: f64,
    pub confidence: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub follow_score: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fade_score: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub no_action_score: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub horizon_seconds: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signal_half_life_seconds: Option<i64>,
    pub evidence_bundle_id: String,
    #[serde(default)]
    pub evidence_ids: Vec<String>,
    pub model_uri: String,
    pub legal_status: LegalStatus,
    /// Must remain `false` unless an authoritative legal source supports an
    /// intent finding — never set by a heuristic or LLM.
    pub intent_inferred: bool,
    #[serde(default)]
    pub limitations: Vec<String>,
}
