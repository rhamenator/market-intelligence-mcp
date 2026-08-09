# Publication-readiness audit

Audit date: 2026-08-09

## Executive summary

No critical or high-severity publication blockers remain in the reviewed
working tree. The repository is suitable for public source visibility as a
fixture-backed research implementation after these changes are committed and
its sibling repository links are made public together.

## Findings and remediation

- **PUB-001 — Medium — Broken architecture link:** README linked to
  `smart-dynamic-hedge`; corrected to the public
  `smart-dynamic-hedge-project` repository.
- **PUB-002 — Medium — Missing automated quality gate:** added pinned GitHub
  Actions for formatting, Clippy, locked tests, executable invariant checks,
  and RustSec dependency auditing.
- **PUB-003 — Medium — Missing private reporting process:** added
  `SECURITY.md` and Dependabot configuration.

## Verification completed

- Gitleaks 8.30.1 scanned all refs, four commits, and the final working tree:
  no findings.
- `cargo test --workspace`: passed.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- `cargo audit`: no known vulnerable dependency found.
- GPL-3.0-or-later license and NOTICE are present.

## Residual limitations

Most provider, persistence, scheduling, and signal implementations are mock or
not started. There is no remote authentication because there is no remote
transport. Public visibility does not make the software production-ready and
does not grant rights to collect or trade on third-party data.
