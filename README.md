<p align="center">
  <img src=".github/assets/logo.png" alt="Market Intelligence MCP logo" width="220">
</p>

# market-intelligence-mcp

Lawful collection, normalization, and point-in-time storage of public or
properly licensed market intelligence — political financial disclosures,
corporate insider transactions, beneficial ownership, institutional filings,
issuer capital activity, unusual options flow, large/block/dark-pool trades,
futures positioning, public policy events, public statements, news, and
on-chain crypto-whale activity.

This service is one of three repositories in the Smart Dynamic Hedge V2
architecture:

- [`smart-dynamic-hedge-project`](https://github.com/rhamenator/smart-dynamic-hedge-project) — strategy, research, GUI, and autonomy plane. Consumes this service's normalized intelligence.
- **`market-intelligence-mcp`** (this repo) — source terms, robots, licensing, provenance, staleness, correction handling, and MNPI quarantine. **No order-entry tools. No broker credentials, ever.**
- [`trade-guard-mcp`](https://github.com/rhamenator/trade-guard-mcp) — account state, risk policy, and execution.

See [`market-system-contracts`](https://github.com/rhamenator/market-system-contracts)
for the schemas all three repositories share.

## Why this is a separate repository, not a module of `smart-dynamic-hedge`

This service ingests untrusted public text (filings, news, social content) —
a prompt-injection and supply-chain attack surface. It must never share a
process, network boundary, or credential scope with anything that holds
broker/execution credentials. See `docs/THREAT_MODEL.md`.

## Status

This is a **Phase 2 fixture-only vertical slice, now with a real MCP
server**. See [`docs/CAPABILITY_STATUS.md`](docs/CAPABILITY_STATUS.md) for
the exact, honest status of every crate, service, and provider — most
providers/signals/storage backends are still `not-started` or `mock`.
Nothing in this repository talks to a real external provider, and no
order-entry or broker-credential type exists anywhere in this codebase —
never will.

## What actually works today

```bash
cargo build --workspace
cargo test --workspace
cargo run --bin market_intelligence_server             # one-shot demo, unchanged from before
cargo run --bin market_intelligence_server -- --check   # nonzero exit on any invariant violation; for CI
cargo run --bin market_intelligence_server -- mcp       # real MCP stdio JSON-RPC server, 11 tools
```

The demo binary constructs a fixture political-disclosure `SourceRecord`
with a `research-only` `SourceUseDecision`, runs it through the deterministic
source-policy engine for both a `research` and a `live-execution` purpose,
and shows the live-execution attempt being correctly denied — then runs the
`NoActionBaseline` signal against the resulting evidence bundle. This proves
the core safety property end to end: **a research-only source can never back
a live-execution decision**, enforced by code, not by convention.

The `mcp` subcommand exposes that same underlying logic (plus the
in-memory store and a new multi-record `EvidenceBundle` builder) as 11
callable, read-only MCP tools — `health`, `capabilities`, `tool-catalog`,
`self-test`, `list-configured-sources`, `ingest-source-records`,
`get-source-record-history`, `get-source-record-as-of`,
`check-source-use-eligibility`, `build-evidence-bundle`,
`evaluate-signal`. See `crates/market-intelligence-mcp-transport/src/tools.rs`
for exact input schemas.

## Layout

```text
crates/
  market-intelligence-contracts/      generated-bindings placeholder (not-started)
  market-intelligence-core/           hand-transcribed domain types (fixture-tested)
  market-intelligence-storage/        SourceRecordStore trait + in-memory impl (schema-only)
  market-intelligence-source-policy/  deterministic SourceUseDecision/MNPI gate (fixture-tested)
  market-intelligence-entity-resolution/  (not-started)
  market-intelligence-signals/        SignalPlugin trait + NoActionBaseline (mock)
  market-intelligence-source-sdk/     SourceAdapter trait + fixture adapter (fixture-tested)
  market-intelligence-mcp-transport/  MCP stdio JSON-RPC transport, 11 tools (fixture-tested + real end-to-end)
services/
  market-intelligence-server/         demo binary + real MCP stdio server (fixture-tested + real end-to-end)
  market-intelligence-scheduler/      respectful-retrieval scheduler (not-started)
  market-intelligence-worker/         sandboxed Python worker host (not-started)
  market-intelligence-admin/          operator dashboard (not-started)
```

## Dependency policy

The only third-party runtime dependencies in this workspace are `serde` and
`serde_json`. Everything else is hand-rolled:

- `market_intelligence_core::utc_timestamp::UtcTimestamp` replaces the
  `time` crate with a UTC-only RFC 3339 parser/formatter (Howard Hinnant's
  public-domain civil-calendar algorithm), because this system never needs
  timezones or leap seconds — only UTC.
- Hand-written `Display`/`Error` impls replace `thiserror` in
  `market-intelligence-source-policy` and `market-intelligence-storage`.
- `uuid` was removed outright — nothing in this codebase actually
  generates a UUID yet; identifiers are opaque `String`s per the shared
  schema.

`serde`/`serde_json` are kept deliberately: this system's wire format is
JSON Schema across three repositories and three languages, and hand-rolling
a JSON parser would trade an extremely widely used, heavily audited
dependency for bespoke parsing code — a worse security trade-off, not a
better one. Every crate enforces `unsafe_code = "forbid"` and
`clippy::all` at the workspace level (`Cargo.toml` `[workspace.lints]`).

## Testing philosophy

The bar here is SQLite-developer-style: boundary/edge-case coverage, no
third-party test-framework dependency, and dependency-free fuzz/robustness
testing using a hand-rolled deterministic PRNG rather than `proptest` or
`cargo-fuzz`. `UtcTimestamp` alone has ~20 boundary tests (leap years
including the 100/400-year rule, fractional seconds, non-ASCII input) plus
three fuzz-smoke tests that feed it thousands of random byte strings,
random mutations of a valid timestamp, and random valid instants — all
asserting no panic, with reproducible seeds. `cargo test --workspace`
exercises more than 90 unit, boundary, robustness, and transport tests. Add
tests for new boundary/adversarial cases *before* trusting new parsing or
policy logic, not after a bug surfaces — that discipline already caught and
fixed a real bug in the shared `decimal-string` schema (see
`market-system-contracts`'s changelog).

## Security and publication status

The repository intentionally contains no provider integrations, order-entry
tools, or broker credentials. See [SECURITY.md](SECURITY.md) for private
vulnerability reporting, [the threat model](docs/THREAT_MODEL.md) for trust
boundaries, and [the publication-readiness audit](docs/PUBLICATION_READINESS.md)
for the checks completed before public release.

## Where this comes from

This repository implements `02-create-market-intelligence-mcp.md` from the
`smart-dynamic-hedge-v2-prompt-bundle` coding-agent prompt package
(2026-07-19), constrained by the mandatory rules in
`05-source-policy-and-legal-boundaries.md` and staged per
`06-implementation-order-and-acceptance.md`.

Two rules from that source-policy document are load-bearing in this code,
not just documentation:

1. Public availability of information is not the same as an unrestricted
   right to trade on it — see `SourceUseDecision::permits_live_execution`.
2. The system must never ask a model to decide whether questionable
   information is legally tradable; the deterministic
   `market-intelligence-source-policy` crate makes that decision.

## License

GNU General Public License v3.0 (or, at your option, any later version). See
[LICENSE](LICENSE) and [NOTICE](NOTICE).
