//! Append-only storage abstraction for `SourceRecord` and derived
//! `MarketIntelligenceEvent` history.
//!
//! Status: schema-only. This trait defines the contract; no SQLite or
//! PostgreSQL-backed implementation exists yet (Phase 2 continuation). An
//! in-memory implementation is provided for tests so downstream crates can
//! be developed and tested against the trait before a real backend lands.

use market_intelligence_core::SourceRecord;
use std::collections::HashMap;
use std::sync::Mutex;

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
        as_of_public_availability_time: time::OffsetDateTime,
    ) -> Option<SourceRecord>;
}

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("revision {0} is not greater than the latest stored revision")]
    RevisionNotMonotonic(u32),
}

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
        if let Some(last) = entry.last() {
            if record.revision <= last.revision {
                return Err(StoreError::RevisionNotMonotonic(record.revision));
            }
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
        as_of_public_availability_time: time::OffsetDateTime,
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
