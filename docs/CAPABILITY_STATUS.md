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
| `market-intelligence-core` | fixture-tested | Domain types hand-transcribed from `market-system-contracts`; 38 unit/boundary/fuzz-smoke tests (was 32 — added `sha256.rs`, duplicated from `smart-dynamic-hedge`/`trade-guard-mcp`'s own hand-rolled SHA-256, verified against the same NIST vectors), including a dependency-free `UtcTimestamp` replacing the `time` crate. |
| `market-intelligence-source-policy` | fixture-tested | Deterministic `SourceUseDecision`/MNPI gate; 27 tests including an exhaustive per-`MnpiClassification` check and a fuzz-smoke test over random decision/purpose/eligibility combinations. |
| `market-intelligence-source-sdk` | fixture-tested | `SourceAdapter` trait + `FixtureSourceAdapter`. No real provider implements it yet. |
| `market-intelligence-signals` | mock | Only `NoActionBaseline` exists, to prove the `SignalPlugin` shape. No calibrated signal family implemented. |
| `market-intelligence-storage` | schema-only | `SourceRecordStore` trait + in-memory implementation; 7 tests including a fuzz-smoke test over random revision sequences. No SQLite/PostgreSQL backend. |
| `market-intelligence-mcp-transport` | **fixture-tested + real end-to-end** | **New this pass.** Hand-rolled MCP stdio JSON-RPC 2.0 transport (no MCP SDK dependency, matching the sibling repositories' own transports) exposing 11 read-only tools over the existing fixture/in-memory-storage/mock-signal crates, plus a new `evidence_builder` module that generalizes what the demo binary used to do inline for one record into a real, tested, canonically-hashed `EvidenceBundle` builder for N records. Verified with a real piped multi-message MCP session against the compiled release binary. |
| `market-intelligence-entity-resolution` | not-started | |
| `market-intelligence-contracts` | not-started | Reserved for generated bindings; not implemented. |

Total: **87 tests** across the workspace (was 54), `cargo test --workspace`
all green, `cargo clippy --workspace --all-targets` clean under
`clippy::all`. Only third-party runtime dependencies: `serde`,
`serde_json`.

## Tools implemented (11, all read-only — zero order-entry tools exist)

```text
health, capabilities, tool-catalog, self-test
list-configured-sources, ingest-source-records
get-source-record-history, get-source-record-as-of
check-source-use-eligibility, build-evidence-bundle, evaluate-signal
```

No order-entry tool exists in this crate, and none ever will — this
repository's whole reason for being a separate security boundary is that
it must never hold broker credentials or submit orders. `evaluate-signal`
only ever runs `NoActionBaseline` today since that's the only signal
implemented — a real signal family plugging into the same tool is Phase 5
work.

## Services

| Service | Status | Notes |
|---|---|---|
| `market-intelligence-server` | fixture-tested + real end-to-end | Demo mode (`cargo run --bin market_intelligence_server` / `--check`) is unchanged from before this pass. **New:** `market_intelligence_server mcp` runs a real MCP stdio server — not a demo binary only, as the previous status note said. |
| `market-intelligence-scheduler` | not-started | No real source adapters exist to schedule. |
| `market-intelligence-worker` | not-started | Sandboxed Python-worker orchestration not built. |
| `market-intelligence-admin` | not-started | Operator dashboard not built. |

## Providers

No external provider (Unusual Whales, SEC EDGAR, US House/Senate/OGE
disclosures, any international disclosure registry, any crypto on-chain
adapter) is implemented at any level. All are `not-started`. This is
intentional — see `06-implementation-order-and-acceptance.md` Phase 5.
The MCP server's `ingest-source-records`/`list-configured-sources` tools
operate only against the one configured `FixtureSourceAdapter` a caller
wires up (the binary wires up exactly one, the same
`us-house-periodic-transaction-report` fixture the demo mode already
used) — there is no dynamic provider registry or real network call
anywhere in this repository.

## What this pass proves, and what it doesn't

Proves: the source-policy gate, in-memory store, and `NoActionBaseline`
signal — all already tested in isolation — compose correctly through a
real MCP JSON-RPC transport, not just through a demo binary's `main()`;
`build-evidence-bundle` generalizes the single-record inline logic the
demo used to have into a real, reusable, canonically-hashed multi-record
builder.

Does not prove: anything about a real external provider (none exists);
anything about durable storage across a process restart (storage is
in-memory only); anything about a remote/authenticated transport (stdio
only, single caller, trusted-local-caller assumption, same as
`smart-dynamic-hedge`'s and `trade-guard-mcp`'s own stdio servers).
