//! `IntelligenceState` — bundles the configured `FixtureSourceAdapter`s and
//! the in-memory `SourceRecordStore` into the one piece of state
//! `tools.rs`/`mcp.rs` operate on. Single request at a time (stdio), so
//! plain `&mut IntelligenceState` is sufficient — no locking needed here
//! (the store itself is internally `Mutex`-guarded for its own
//! `Send + Sync` trait bound, independent of this).

use market_intelligence_source_sdk::{FixtureSourceAdapter, SourceAdapter};
use market_intelligence_storage::InMemorySourceRecordStore;

pub struct IntelligenceState {
    pub adapters: Vec<FixtureSourceAdapter>,
    pub store: InMemorySourceRecordStore,
}

impl IntelligenceState {
    pub fn new(adapters: Vec<FixtureSourceAdapter>) -> Self {
        IntelligenceState {
            adapters,
            store: InMemorySourceRecordStore::new(),
        }
    }

    pub fn adapter(&self, source_id: &str) -> Option<&FixtureSourceAdapter> {
        self.adapters.iter().find(|a| a.source_id() == source_id)
    }
}
