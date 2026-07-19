//! `SourceAdapter` trait that every provider integration implements,
//! plus a fixture-backed adapter for local/offline development.
//!
//! Status: fixture-tested. No real external provider adapter (Unusual
//! Whales, SEC EDGAR, etc.) exists yet — those are Phase 5 work. This
//! crate lets the rest of the service be built and tested against
//! `FixtureSourceAdapter` today.

use market_intelligence_core::SourceRecord;

/// Capability/health/fetch contract every source adapter implements.
/// Real adapters must additionally honor the respectful-retrieval rules
/// in `05-source-policy-and-legal-boundaries.md` section 7 (rate limits,
/// conditional requests, no anti-bot bypass, kill switch).
pub trait SourceAdapter {
    fn source_id(&self) -> &str;

    /// Returns currently available fixture/cached records. Real adapters
    /// will additionally expose `fetch_page`/`sync` methods that perform
    /// network I/O; that surface is intentionally not defined yet so it
    /// isn't designed against zero real providers.
    fn records(&self) -> Vec<SourceRecord>;
}

/// An adapter whose only data source is a static, checked-in fixture file
/// or in-memory vector. Used for local demo mode and tests — see
/// `06-implementation-order-and-acceptance.md` Phase 2 acceptance:
/// "a fixture cannot be queried for a purpose that its policy forbids."
pub struct FixtureSourceAdapter {
    source_id: String,
    records: Vec<SourceRecord>,
}

impl FixtureSourceAdapter {
    pub fn new(source_id: impl Into<String>, records: Vec<SourceRecord>) -> Self {
        Self {
            source_id: source_id.into(),
            records,
        }
    }
}

impl SourceAdapter for FixtureSourceAdapter {
    fn source_id(&self) -> &str {
        &self.source_id
    }

    fn records(&self) -> Vec<SourceRecord> {
        self.records.clone()
    }
}
