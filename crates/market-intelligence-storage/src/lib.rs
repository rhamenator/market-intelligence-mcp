//! Append-only storage abstraction for `SourceRecord` and derived
//! `MarketIntelligenceEvent` history.
//!
//! Status: schema-only. This trait defines the contract; no SQLite or
//! PostgreSQL-backed implementation exists yet (Phase 2 continuation). An
//! in-memory implementation is provided for tests so downstream crates can
//! be developed and tested against the trait before a real backend lands.

use std::collections::HashMap;
use std::fmt;
use std::sync::Mutex;

use market_intelligence_core::{SourceRecord, UtcTimestamp};

pub trait SourceRecordStore: Send + Sync {
    /// Appends a new revision of a record. Implementations must never
    /// overwrite or delete a prior revision — see `SourceRecord::revision`
    /// and `correction_state`.
    fn append(&self, record: SourceRecord) -> Result<(), StoreError>;

    /// Returns every revision stored for `source_record_id`, oldest first.
    fn history(&self, source_record_id: &str) -> Vec<SourceRecord>;

    /// Returns the revision that was the current, non-superseded version as
    /// of `as_of_public_availability_time`, if any — the point-in-time read
    /// path backtests and autonomous decisions must use.
    fn as_of(
        &self,
        source_record_id: &str,
        as_of_public_availability_time: UtcTimestamp,
    ) -> Option<SourceRecord>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoreError {
    RevisionNotMonotonic(u32),
}

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RevisionNotMonotonic(rev) => {
                write!(
                    f,
                    "revision {rev} is not greater than the latest stored revision"
                )
            }
        }
    }
}

impl std::error::Error for StoreError {}

/// In-memory implementation for unit and fixture tests only. Not durable;
/// never use outside test code.
#[derive(Default)]
pub struct InMemorySourceRecordStore {
    records: Mutex<HashMap<String, Vec<SourceRecord>>>,
}

impl InMemorySourceRecordStore {
    pub fn new() -> Self {
        Self::default()
    }
}

impl SourceRecordStore for InMemorySourceRecordStore {
    fn append(&self, record: SourceRecord) -> Result<(), StoreError> {
        let mut guard = self.records.lock().expect("lock poisoned");
        let entry = guard.entry(record.source_record_id.clone()).or_default();
        if let Some(last) = entry.last()
            && record.revision <= last.revision
        {
            return Err(StoreError::RevisionNotMonotonic(record.revision));
        }
        entry.push(record);
        Ok(())
    }

    fn history(&self, source_record_id: &str) -> Vec<SourceRecord> {
        self.records
            .lock()
            .expect("lock poisoned")
            .get(source_record_id)
            .cloned()
            .unwrap_or_default()
    }

    fn as_of(
        &self,
        source_record_id: &str,
        as_of_public_availability_time: UtcTimestamp,
    ) -> Option<SourceRecord> {
        self.history(source_record_id)
            .into_iter()
            .filter(|r| {
                r.public_availability_time
                    .map(|t| t <= as_of_public_availability_time)
                    .unwrap_or(false)
            })
            .max_by_key(|r| r.revision)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use market_intelligence_core::{CorrectionState, MnpiClassification, SourceType};

    fn record_at(revision: u32, public_availability_time: Option<UtcTimestamp>) -> SourceRecord {
        SourceRecord {
            schema_version: "2.0.0".into(),
            source_record_id: "rec-1".into(),
            source_id: "fixture-source".into(),
            provider_record_id: None,
            source_type: SourceType::Other,
            jurisdiction: None,
            retrieval_method: "fixture".into(),
            transaction_time: None,
            event_time: None,
            filing_time: None,
            published_time: None,
            public_availability_time,
            retrieved_time: UtcTimestamp::UNIX_EPOCH,
            corrected_time: None,
            content_hash: None,
            execution_eligible: false,
            redistribution_eligible: false,
            quality: None,
            staleness_seconds: None,
            expected_reporting_lag_seconds: None,
            revision,
            correction_state: CorrectionState::Original,
            untrusted_text: false,
            mnpi_classification: MnpiClassification::PublicConfirmed,
        }
    }

    #[test]
    fn append_rejects_non_monotonic_revision() {
        let store = InMemorySourceRecordStore::new();
        store
            .append(record_at(1, Some(UtcTimestamp::UNIX_EPOCH)))
            .unwrap();
        let result = store.append(record_at(1, Some(UtcTimestamp::UNIX_EPOCH)));
        assert_eq!(result, Err(StoreError::RevisionNotMonotonic(1)));
        let result = store.append(record_at(0, Some(UtcTimestamp::UNIX_EPOCH)));
        assert_eq!(result, Err(StoreError::RevisionNotMonotonic(0)));
    }

    #[test]
    fn append_accepts_strictly_increasing_revisions() {
        let store = InMemorySourceRecordStore::new();
        for revision in 1..=5 {
            store
                .append(record_at(revision, Some(UtcTimestamp::UNIX_EPOCH)))
                .unwrap();
        }
        assert_eq!(store.history("rec-1").len(), 5);
    }

    #[test]
    fn history_is_empty_for_unknown_id() {
        let store = InMemorySourceRecordStore::new();
        assert!(store.history("never-seen").is_empty());
    }

    #[test]
    fn as_of_returns_none_when_all_revisions_are_in_the_future() {
        let store = InMemorySourceRecordStore::new();
        let future = UtcTimestamp::UNIX_EPOCH.checked_add_days(365).unwrap();
        store.append(record_at(1, Some(future))).unwrap();
        assert!(store.as_of("rec-1", UtcTimestamp::UNIX_EPOCH).is_none());
    }

    #[test]
    fn as_of_never_returns_a_revision_published_after_the_query_time() {
        let store = InMemorySourceRecordStore::new();
        let t0 = UtcTimestamp::UNIX_EPOCH;
        let t10 = t0.checked_add_days(10).unwrap();
        let t20 = t0.checked_add_days(20).unwrap();
        store.append(record_at(1, Some(t0))).unwrap();
        store.append(record_at(2, Some(t20))).unwrap();

        let as_of_t10 = store.as_of("rec-1", t10).unwrap();
        assert_eq!(as_of_t10.revision, 1, "revision 2 is not public yet at t10");

        let as_of_t20 = store.as_of("rec-1", t20).unwrap();
        assert_eq!(as_of_t20.revision, 2);
    }

    #[test]
    fn as_of_ignores_records_with_no_public_availability_time() {
        let store = InMemorySourceRecordStore::new();
        store.append(record_at(1, None)).unwrap();
        assert!(store.as_of("rec-1", UtcTimestamp::UNIX_EPOCH).is_none());
    }

    /// A tiny, dependency-free xorshift64 PRNG — see
    /// `market_intelligence_core::utc_timestamp` tests for the rationale.
    struct XorShift64(u64);
    impl XorShift64 {
        fn next(&mut self) -> u64 {
            let mut x = self.0;
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            self.0 = x;
            x
        }
    }

    /// Appends a long random sequence of revisions (some monotonic, some
    /// not) and checks two invariants hold regardless of the random
    /// sequence: (1) the store never panics, and (2) `history()` is always
    /// sorted by strictly increasing revision, with no duplicates.
    #[test]
    fn fuzz_smoke_random_revision_sequence_keeps_history_strictly_increasing() {
        let store = InMemorySourceRecordStore::new();
        let mut rng = XorShift64(0x243F6A8885A308D3);
        let mut highest_accepted = 0u32;
        for _ in 0..5_000 {
            let revision = (rng.next() % 50) as u32; // deliberately churns near the boundary
            let result = store.append(record_at(revision, Some(UtcTimestamp::UNIX_EPOCH)));
            if revision > highest_accepted {
                assert!(
                    result.is_ok(),
                    "expected acceptance of increasing revision {revision}"
                );
                highest_accepted = revision;
            } else {
                assert_eq!(result, Err(StoreError::RevisionNotMonotonic(revision)));
            }
        }
        let history = store.history("rec-1");
        for window in history.windows(2) {
            assert!(
                window[0].revision < window[1].revision,
                "history is not strictly increasing: {} then {}",
                window[0].revision,
                window[1].revision
            );
        }
    }
}
