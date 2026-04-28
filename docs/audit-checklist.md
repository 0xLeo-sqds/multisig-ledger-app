# Security Audit Preparation Checklist

## Pre-Audit Requirements

### Code Quality
- [ ] All source code committed and tagged
- [ ] Reproducible build instructions documented
- [ ] Docker build verified (all 5 device targets)
- [ ] No compiler warnings
- [ ] Strict clippy: deny(unsafe_code) except for NVM access

### Security Requirements (Ledger Standards)
- [ ] All signing actions require explicit user confirmation (NBGL review)
- [ ] Critical information displayed before signing (amount, address, action)
- [ ] Blind signing disabled by default
- [ ] Key material zeroed after use (via `zeroize`)
- [ ] No medium/high severity vulnerabilities in dependencies
- [ ] Application flags follow least-privilege principle

### Input Validation
- [ ] Solana message structural validation (all indices bounds-checked)
- [ ] Versioned (v0) messages rejected
- [ ] BIP32 path validation (3-4 hardened, 44'/501')
- [ ] APDU state machine resets on unexpected commands
- [ ] Max transaction buffer enforced (1300 bytes)
- [ ] All on-chain string data sanitized (ASCII-only, 64-byte cap)

### Testing
- [ ] Ragger/Speculos test suite passes on all device targets
- [ ] Fuzz testing (24+ hours, no crashes)
- [ ] Malformed input tests (truncated, invalid indices, oversized)
- [ ] Blind signing toggle tests (enabled/disabled behavior)
- [ ] BIP32 path rejection tests (wrong purpose, coin, unhardened)

### Documentation
- [ ] Derivation path security rationale (docs/derivation-path-rationale.md)
- [ ] APDU protocol specification
- [ ] Threat model documentation
- [ ] User-facing documentation

## Audit Deliverables

1. Source code repository access
2. Build instructions (Docker)
3. Test suite + fuzz corpus
4. Derivation path rationale document
5. APDU protocol specification
6. App icons and catalog listing materials

## Audit Firms
- **Kudelski IoT** — Ledger-approved
- **Quarkslab** — Ledger-approved

## Timeline
1. Internal security review (1 week)
2. Engage audit firm (scheduling: 2-4 weeks)
3. Audit execution (2-3 weeks)
4. Remediation (1-2 weeks)
5. Re-audit if needed (1 week)
6. Ledger submission (1 week)
