# Changelog

## Unreleased (dependency minimization + SQLite-grade testing pass)

- **Dependency policy tightened**: removed `uuid` (was unused), `time`, and
  `thiserror` from every crate. `market-intelligence-core` now provides a
  hand-rolled, dependency-free `UtcTimestamp` (RFC 3339 UTC-only parsing,
  formatting, arithmetic, manual serde impls) built on Howard Hinnant's
  public-domain civil-calendar algorithm, plus hand-written
  `Display`/`Error` impls replacing `thiserror`-derived ones in
  `market-intelligence-source-policy` and `market-intelligence-storage`.
  The only remaining third-party dependencies in the whole workspace are
  `serde`/`serde_json` (kept deliberately — see workspace `Cargo.toml`
  comment: hand-rolling JSON parsing would trade a heavily-audited,
  near-universal dependency for bespoke code, a worse security trade-off,
  not a better one).
- Added `[workspace.lints]` with `unsafe_code = "forbid"` and
  `clippy.all = "warn"`, applied to every crate/service via
  `[lints] workspace = true`.
- **Testing bar raised to SQLite-developer style**: `UtcTimestamp` ships
  ~20 boundary tests (epoch, leap years incl. the 100/400-year rule,
  fractional seconds, malformed input, non-ASCII bytes) plus three
  dependency-free fuzz-smoke tests using a hand-rolled xorshift64 PRNG
  (deterministic, reproducible seeds) that hammer the parser with random
  bytes, random mutations of a valid timestamp, and random valid instants
  checked for round-trip stability — all with a no-panic guarantee.
  `market-intelligence-source-policy` and `market-intelligence-storage`
  gained equivalent fuzz-smoke tests and additional boundary/exhaustive
  tests (every `MnpiClassification` variant checked against every
  non-research `BundlePurpose`, monotonic-revision fuzzing). Total test
  count: 54 (was 13).
- `market-intelligence-server`'s demo binary gained a real `--check` mode
  (previously described in a doc comment but not implemented) that asserts
  the three core invariants and exits nonzero on violation, for CI use.
- Added `.cargo/config.toml` disabling incremental compilation — this
  machine's antivirus was intermittently corrupting incremental build
  artifacts (`LNK1201`/"Access is denied" during linking); a clean full
  rebuild is more reliable than a flaky faster one.

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
