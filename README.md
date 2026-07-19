# market-intelligence-mcp

Lawful collection, normalization, and point-in-time storage of public or
properly licensed market intelligence — political financial disclosures,
corporate insider transactions, beneficial ownership, institutional filings,
issuer capital activity, unusual options flow, large/block/dark-pool trades,
futures positioning, public policy events, public statements, news, and
on-chain crypto-whale activity.

This service is one of three repositories in the Smart Dynamic Hedge V2
architecture:

- [`smart-dynamic-hedge`](https://github.com/rhamenator/smart-dynamic-hedge) — strategy, research, GUI, and autonomy plane. Consumes this service's normalized intelligence.
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

This is a **Phase 2 fixture-only vertical slice**. See
[`docs/CAPABILITY_STATUS.md`](docs/CAPABILITY_STATUS.md) for the exact,
honest status of every crate, service, and provider — most things here are
`not-started` or `mock`. Nothing in this repository talks to a real external
provider, and no order-entry or broker-credential type exists anywhere in
this codebase.

## What actually works today

```bash
cargo build --workspace
cargo test --workspace
cargo run --bin market_intelligence_server
```

The demo binary constructs a fixture political-disclosure `SourceRecord`
with a `research-only` `SourceUseDecision`, runs it through the deterministic
source-policy engine for both a `research` and a `live-execution` purpose,
and shows the live-execution attempt being correctly denied — then runs the
`NoActionBaseline` signal against the resulting evidence bundle. This proves
the core safety property end to end: **a research-only source can never back
a live-execution decision**, enforced by code, not by convention.

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
  market-intelligence-mcp-transport/  MCP stdio/HTTP transport (not-started)
services/
  market-intelligence-server/         demo binary (fixture-tested)
  market-intelligence-scheduler/      respectful-retrieval scheduler (not-started)
  market-intelligence-worker/         sandboxed Python worker host (not-started)
  market-intelligence-admin/          operator dashboard (not-started)
```

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
