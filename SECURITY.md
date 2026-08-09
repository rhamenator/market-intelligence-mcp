# Security policy

## Supported code

Security fixes are applied to the current `main` branch. This project is a
fixture-backed research implementation and does not provide a live market-data
or order-execution service.

## Reporting a vulnerability

Please use GitHub's private vulnerability-reporting flow from the repository's
**Security** tab. If that flow is unavailable, email
`rich@yourfoxprodeveloper.com` with the repository name, affected revision,
impact, reproduction steps, and any proposed mitigation.

Do not open a public issue for an unpatched vulnerability and do not include
real credentials, nonpublic financial information, or personal data in a
report. You should receive an acknowledgment within seven days.

## Security boundary

This repository must never contain broker credentials or an order-entry tool.
It treats collected text as untrusted input and keeps source-use eligibility in
deterministic policy code. See [docs/THREAT_MODEL.md](docs/THREAT_MODEL.md).
