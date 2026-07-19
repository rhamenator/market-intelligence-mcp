# Capability status

Per `06-implementation-order-and-acceptance.md` in the originating prompt
bundle: "No README may call a provider 'supported' without showing the exact
status." Status values:

```text
not-started, schema-only, mock, fixture-tested, record-replay-tested,
sandbox-tested, demo-tested, paper-tested, certification-tested,
live-tested, disabled-by-policy, unavailable-by-entitlement,
unavailable-by-jurisdiction, deprecated
```

## Crates

| Crate | Status | Notes |
|---|---|---|
| `market-intelligence-core` | fixture-tested | Domain types hand-transcribed from `market-system-contracts`; 32 unit/boundary/fuzz-smoke tests, including a dependency-free `UtcTimestamp` replacing the `time` crate. |
| `market-intelligence-source-policy` | fixture-tested | Deterministic `SourceUseDecision`/MNPI gate; 11 tests including an exhaustive per-`MnpiClassification` check and a fuzz-smoke test over random decision/purpose/eligibility combinations. |
| `market-intelligence-source-sdk` | fixture-tested | `SourceAdapter` trait + `FixtureSourceAdapter`. No real provider implements it yet. |
| `market-intelligence-signals` | mock | Only `NoActionBaseline` exists, to prove the `SignalPlugin` shape. No calibrated signal family implemented. |
| `market-intelligence-storage` | schema-only | `SourceRecordStore` trait + in-memory implementation; 7 tests including a fuzz-smoke test over random revision sequences. No SQLite/PostgreSQL backend. |
| `market-intelligence-entity-resolution` | not-started | |
| `market-intelligence-mcp-transport` | not-started | No MCP SDK selected yet; no stdio/HTTP transport exists. |
| `market-intelligence-contracts` | not-started | Reserved for generated bindings; not implemented. |

Total: 54 tests across the workspace, `cargo test --workspace` all green,
`cargo clippy --workspace` clean under `clippy::all`. Only third-party
runtime dependencies: `serde`, `serde_json` (dev-only in most crates).

## Services

| Service | Status | Notes |
|---|---|---|
| `market-intelligence-server` | fixture-tested | Demo binary only (`cargo run --bin market_intelligence_server`). Not an MCP server. |
| `market-intelligence-scheduler` | not-started | No real source adapters exist to schedule. |
| `market-intelligence-worker` | not-started | Sandboxed Python-worker orchestration not built. |
| `market-intelligence-admin` | not-started | Operator dashboard not built. |

## Providers

No external provider (Unusual Whales, SEC EDGAR, US House/Senate/OGE
disclosures, any international disclosure registry, any crypto on-chain
adapter) is implemented at any level. All are `not-started`. This is
intentional — see `06-implementation-order-and-acceptance.md` Phase 5.
