use serde::{Deserialize, Serialize};

/// Structured, non-accusatory legal-status vocabulary.
///
/// `possible-informed-trading-signal` is a market hypothesis, not a
/// conclusion that a crime occurred; never widen this enum with an
/// accusatory value like "criminal" or "manipulator" — see
/// `05-source-policy-and-legal-boundaries.md` section 6 in the originating
/// prompt bundle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LegalStatus {
    Observed,
    Inferred,
    ReportedBySource,
    AllegedInComplaint,
    UnderInvestigation,
    Charged,
    SettledWithoutAdmission,
    SettledWithAdmission,
    AdjudicatedLiable,
    Convicted,
    Dismissed,
    Reversed,
    Unknown,
}

/// Classification of how public/nonpublic a record is, and whether it may
/// become execution evidence. Only `PublicConfirmed` records with a
/// compatible `SourceUseDecision` may become execution evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MnpiClassification {
    PublicConfirmed,
    PublicButUseRestricted,
    PublicationStatusUncertain,
    AmbiguousOrigin,
    SuspectedNonpublic,
    KnownNonpublic,
    ProhibitedSource,
}

impl MnpiClassification {
    /// Only this classification may ever feed a live-execution evidence
    /// bundle. Every other class must be quarantined per the source-policy
    /// engine, never gated by model judgment.
    pub fn is_execution_eligible_class(self) -> bool {
        matches!(self, MnpiClassification::PublicConfirmed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_public_confirmed_is_execution_eligible() {
        assert!(MnpiClassification::PublicConfirmed.is_execution_eligible_class());
        for other in [
            MnpiClassification::PublicButUseRestricted,
            MnpiClassification::PublicationStatusUncertain,
            MnpiClassification::AmbiguousOrigin,
            MnpiClassification::SuspectedNonpublic,
            MnpiClassification::KnownNonpublic,
            MnpiClassification::ProhibitedSource,
        ] {
            assert!(!other.is_execution_eligible_class());
        }
    }

    #[test]
    fn legal_status_serializes_kebab_case() {
        let s = serde_json::to_string(&LegalStatus::SettledWithoutAdmission).unwrap();
        assert_eq!(s, "\"settled-without-admission\"");
    }
}
