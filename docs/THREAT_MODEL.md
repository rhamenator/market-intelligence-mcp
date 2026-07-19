# Threat model (initial)

This is an initial, incomplete threat model matching the Phase 2 scope. It
must be expanded before any real provider adapter is added.

## Trust boundary

This service is the only place in the three-repository system permitted to
retrieve and parse untrusted public/open-web content (filings, news, social
text, provider metadata). It must never:

- hold a broker, brokerage, exchange, custody, or withdrawal credential of
  any kind;
- expose an order-entry, `place-order`, `submit-order`, or account-transfer
  tool;
- pass raw untrusted source text into a model's system/developer
  instructions;
- allow retrieved text to select tools, change policy, or alter execution
  mode (`market-intelligence-signals` currently contains no model calls at
  all, so this is enforced by absence rather than by a guard — that will
  change once an LLM-assisted extraction layer is added, and this document
  must be updated then).

## What's in scope today

- `market-intelligence-source-policy` — deterministic, no model dependency,
  no network access. Unit-tested for the "research-only source cannot back
  a live-execution bundle" and "suspected-nonpublic is always quarantined"
  properties.
- `market-intelligence-signals::NoActionBaseline` — deterministic, always
  returns `no-action`, cannot be steered by input content.

## Explicitly out of scope until a real adapter exists

- Prompt injection via retrieved filing/news/social text — no such content
  is retrieved yet (`FixtureSourceAdapter` only reads a static in-memory
  fixture).
- Source-credential exfiltration — no source holds credentials yet.
- Rate-limit/anti-bot-bypass behavior — no adapter performs network I/O yet.
- MCP transport authentication/authorization — no transport exists yet.

Update this document alongside `docs/CAPABILITY_STATUS.md` every time a
crate's status changes from `not-started`/`mock`/`fixture-tested` to
something that talks to a real network endpoint.
