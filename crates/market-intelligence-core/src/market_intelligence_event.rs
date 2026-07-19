use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EventType {
    PoliticalTransactionDisclosure,
    InsiderTransaction,
    BeneficialOwnershipChange,
    InstitutionalHoldings,
    IssuerRepurchase,
    IssuerIssuance,
    LargeTrade,
    BlockTrade,
    DarkPoolPrint,
    OptionsFlow,
    FuturesPositioning,
    OnChainTransfer,
    ExchangeInflow,
    PublicStatement,
    PolicyEvent,
    PromotionEvent,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EventDirection {
    Buy,
    Sell,
    Long,
    Short,
    TransferIn,
    TransferOut,
    Issuance,
    Repurchase,
    Unknown,
}

/// Normalized public-disclosure or whale event. Never assert aggressor side
/// or intent without a confidence and method; always preserve alternative
/// explanations (hedging, index rebalance, market making, tax planning...).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct MarketIntelligenceEvent {
    pub event_id: String,
    pub event_type: EventType,
    pub actor_ids: Vec<String>,
    #[serde(default)]
    pub issuer_or_asset_ids: Vec<String>,
    pub instrument_ids: Vec<String>,
    pub direction: EventDirection,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub direction_confidence: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quantity: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notional: Option<String>,
    #[serde(default, with = "time::serde::rfc3339::option")]
    pub transaction_time: Option<OffsetDateTime>,
    #[serde(default, with = "time::serde::rfc3339::option")]
    pub public_availability_time: Option<OffsetDateTime>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reporting_lag_seconds: Option<i64>,
    pub source_record_ids: Vec<String>,
    pub execution_eligible: bool,
    /// Must be non-empty for any event type that could plausibly have an
    /// innocent explanation — which is nearly all of them. See
    /// `05-source-policy-and-legal-boundaries.md`.
    pub alternative_explanations: Vec<String>,
}
