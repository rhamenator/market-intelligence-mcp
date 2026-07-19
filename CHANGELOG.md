# Changelog

## Unreleased — 2026-07-19

Initial Phase 2 vertical-slice scaffold:

- Cargo workspace with 8 library crates and 4 service binaries, matching
  the layout in `02-create-market-intelligence-mcp.md`.
- `market-intelligence-core`: hand-transcribed domain types for
  `SourceRecord`, `MarketIntelligenceEvent`, `EvidenceBundle`,
  `SignalAssessment`, `SourceUseDecision`, `LegalStatus`/`MnpiClassification`,
  entity/relationship types.
- `market-intelligence-source-policy`: deterministic source-use/MNPI gate
  with unit tests covering the acceptance scenarios in
  `06-implementation-order-and-acceptance.md` Phase 2 (research-only source
  cannot back a live-execution bundle, suspected-nonpublic records are
  quarantined regardless of source-use decision, future public-availability
  times are rejected).
- `market-intelligence-source-sdk`: `SourceAdapter` trait + fixture adapter.
- `market-intelligence-signals`: `SignalPlugin` trait + `NoActionBaseline`.
- `market-intelligence-storage`: `SourceRecordStore` trait + in-memory
  implementation.
- `market-intelligence-server`: fixture-only demo binary wiring the above
  together end to end with no network access.
- `market-intelligence-mcp-transport`, `-entity-resolution`, `-contracts`,
  and the `scheduler`/`worker`/`admin` services are explicit stubs — see
  `docs/CAPABILITY_STATUS.md`.

No external provider, MCP transport, or durable storage backend exists yet.
