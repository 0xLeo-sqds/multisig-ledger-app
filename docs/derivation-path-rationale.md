# Derivation Path Security Rationale

## Summary

The Squads Ledger app uses BIP44 derivation path `44'/501'/account'/change'` — the same path as the official Solana Ledger app. This document explains why shared derivation paths are necessary and safe.

## Why the Same Path

Squads multisig transactions **are** Solana transactions. They use the same Ed25519 keypairs, the same account model, and the same transaction format. A user's identity in Squads is their Solana pubkey — the same key they use in Phantom, Solflare, and the Solana Ledger app.

If the Squads app used a different derivation path (e.g., `44'/9999'/...`), it would derive a completely different keypair. The user would have a different address in Squads than in their wallet, defeating the purpose of hardware wallet signing.

## Precedent

Multiple Ledger apps share the `44'/60'` Ethereum derivation path:
- The official Ethereum app
- Paraswap, 1inch, and other DeFi apps (via the plugin system)
- StarkNet (shares the Ethereum app for key derivation)

This is an established pattern in the Ledger ecosystem.

## Security Analysis

### What sharing enables
- User's Solana address is identical across the Solana app and the Squads app
- A transaction signed in the Squads app produces a valid signature that the Solana runtime accepts
- The user can verify their address matches across both apps

### What sharing does NOT enable
- The Squads app cannot access keys from other coin types (e.g., `44'/60'` for Ethereum) — BOLOS enforces path restrictions per app
- The Squads app cannot sign without user confirmation — NBGL review is mandatory
- One app cannot interfere with the other — BOLOS runs one app at a time with memory isolation

### Risk assessment
- **Risk**: A malicious Squads app could sign Solana transactions without proper display
- **Mitigation**: Mandatory Ledger security audit (Kudelski or Quarkslab) before app store listing
- **Mitigation**: Open-source code for community review
- **Mitigation**: Clear signing displays full transaction details — the user sees exactly what they sign

## Conclusion

Using `44'/501'` is the only viable derivation path for a Solana-ecosystem Ledger app. Any other path would produce different keys, making the app useless for signing Squads transactions with the user's existing Solana identity.
