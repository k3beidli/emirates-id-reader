# Security and privacy

Emirates ID data is sensitive personal information. This SDK provides direct
local card access; it does not decide whether an application is legally or
operationally entitled to collect, display, transmit, or retain a field.

## SDK guarantees

- Card reads use the local native PC/SC service.
- The SDK makes no network requests and writes no cardholder data to disk.
- Error messages do not contain decoded cardholder field values.
- The CLI redacts reads by default; output includes field presence/lengths and
  reader, ATR/generation, and group-status diagnostics.
- Fingerprint templates and other biometric/private files are never requested.
- Files requiring authentication or secure messaging are not bypassed.

## Caller responsibilities

- Request only the groups needed. Prefer `ReadOptions::identity_only()` for
  matching and attendance systems.
- Do not log `EmiratesIdData` or serialize it into crash reports. Its derived
  `Debug` and `Serialize` implementations include personal data.
- Clear UI state and release the result when the card is removed. Dropping
  allocations does not guarantee memory zeroization or clear application copies.
- Encrypt intentionally retained personal data and define a suitable retention
  policy.
- Treat `CardGeneration` as compatibility metadata, not proof of authenticity.

## Authenticity boundary

Selecting the Emirates ID application and decoding its public files does not
prove that a card is genuine. The official SDK describes separate genuineness
and data-signature validation operations. This crate does not ship ICP secret
material, a Secure Access Module integration, or a certificate trust policy,
and therefore makes no authenticity claim.

Regulated identity-verification systems should use an authorised ICP
integration and establish the required consent, certificate, and audit
processes. This limitation does not affect local public-data extraction for an
authorised application.
