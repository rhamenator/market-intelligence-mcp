use serde::{Deserialize, Serialize};

use crate::legal_status::LegalStatus;
use crate::utc_timestamp::UtcTimestamp;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EntityType {
    Issuer,
    CorporateInsider,
    DirectorOfficerPdmr,
    BeneficialOwner,
    InstitutionalManager,
    ActivistInvestor,
    SovereignWealthFund,
    GovernmentEntity,
    ElectedOfficial,
    AppointedOfficial,
    RelatedPerson,
    RelatedEntity,
    PromoterInfluencer,
    MarketMaker,
    Fund,
    AnonymousWhale,
    WalletOrWalletCluster,
    ExchangeVenue,
    Broker,
    NewsSourceOrganization,
    Other,
    Unknown,
}

/// An actor, resolved by role and effective date rather than a hard-coded
/// current name — officeholders and corporate roles change.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct EntityProfile {
    pub entity_id: String,
    pub entity_type: EntityType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub jurisdiction: Option<String>,
    #[serde(default)]
    pub role_effective_from: Option<UtcTimestamp>,
    #[serde(default)]
    pub role_effective_to: Option<UtcTimestamp>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RelationshipType {
    OfficerOf,
    DirectorOf,
    SpouseOf,
    DependentOf,
    BeneficialOwnerOf,
    Controls,
    ManagedBy,
    AffiliatedWith,
    PubliclyEndorsed,
    CounterpartyTo,
    Other,
}

/// A temporal, evidence-backed relationship between two entities. Every
/// relationship needs provenance, confidence, effective dates, and a
/// relationship type — never inferred from name matching alone.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ActorRelationship {
    pub relationship_id: String,
    pub from_entity_id: String,
    pub to_entity_id: String,
    pub relationship_type: RelationshipType,
    pub effective_from: UtcTimestamp,
    #[serde(default)]
    pub effective_to: Option<UtcTimestamp>,
    pub confidence: f64,
    pub evidence_ids: Vec<String>,
    pub legal_status: LegalStatus,
}
