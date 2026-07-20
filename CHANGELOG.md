# Changelog

## Unreleased (real MCP stdio server: 11 read-only tools, evidence-bundle builder, sha256)

The previous status note said `market-intelligence-mcp-transport` was
not-started and the server binary was "not an MCP server yet." Both are
now real.

- **Added `sha256.rs` to `market-intelligence-core`**, duplicated
  (implementation + NIST test vectors) from `smart-dynamic-hedge`'s
  `smart_hedge_models::sha256` and `trade-guard-mcp`'s
  `trade_guard_core::sha256` — the same hand-roll-and-verify pattern
  already used for `utc_timestamp.rs`. Needed to give
  `evidence_builder::build_evidence_bundle` a real
  `canonical-hash` instead of a placeholder string.
- **`market-intelligence-mcp-transport`**: a hand-rolled MCP stdio
  JSON-RPC 2.0 transport (no MCP SDK dependency, same shape as the
  sibling repositories' own transports) — `state.rs` (`IntelligenceState`
  bundling configured fixture adapters and the in-memory store),
  `evidence_builder.rs` (generalizes what the demo binary used to do
  inline for exactly one record into a real, tested, canonically-hashed
  `EvidenceBundle` builder for N records), `tools.rs` (11 read-only
  tools), `mcp.rs` (the transport itself).
- **11 tools**: `health`, `capabilities`, `tool-catalog`, `self-test`,
  `list-configured-sources`, `ingest-source-records`,
  `get-source-record-history`, `get-source-record-as-of`,
  `check-source-use-eligibility`, `build-evidence-bundle`,
  `evaluate-signal`. No order-entry tool exists in this crate, and none
  ever will.
- **`market-intelligence-server` gained an `mcp` subcommand** that runs
  the real stdio server. The existing no-argument demo and `--check`
  modes are unchanged — verified by re-running `--check` after this pass
  and confirming identical output/exit code to before.
- **87 tests total** (was 54), `cargo test --workspace` all green,
  `cargo clippy --workspace --all-targets` clean under `clippy::all`
  (also fixed two pre-existing `manual is_multiple_of` clippy warnings in
  `market-intelligence-source-policy`'s fuzz test, unrelated to this
  pass's own code but trivial to clean up while touching the workspace).
- Verified end to end against the compiled release binary: a real piped
  multi-message MCP session (`initialize`, `tools/list`, `health`,
  `list-configured-sources`, `ingest-source-records`,
  `get-source-record-history`, `self-test`) produces correct responses
  throughout, including a real record round-tripping through the
  in-memory store.

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
