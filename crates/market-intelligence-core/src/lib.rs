//! Hand-written Rust domain types mirroring `market-system-contracts`
//! `schemas/2.0.0/*.schema.json`.
//!
//! These are transcribed by hand today. A future milestone should generate
//! this crate (or validate it against) the canonical JSON Schema so the two
//! cannot drift silently — see the workspace README "Status" section.

pub mod entity;
pub mod evidence;
pub mod legal_status;
pub mod market_intelligence_event;
pub mod signal_assessment;
pub mod source_record;
pub mod source_use_decision;

pub use entity::{ActorRelationship, EntityProfile, EntityType, RelationshipType};
pub use evidence::{BundlePurpose, EvidenceBundle, EvidenceRecordRef};
pub use legal_status::{LegalStatus, MnpiClassification};
pub use market_intelligence_event::{EventDirection, EventType, MarketIntelligenceEvent};
pub use signal_assessment::{SignalAssessment, SignalDirection, SignalType};
pub use source_record::{CorrectionState, SourceRecord, SourceType};
pub use source_use_decision::{SourceUseDecision, UsePermission};

/// The schema-version string this crate currently implements.
pub const SCHEMA_VERSION: &str = "2.0.0";
