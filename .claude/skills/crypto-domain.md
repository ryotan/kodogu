---
name: crypto-domain
description: >-
  Domain specification for the crypto operations library (kernel/crypto).
  Use when working with encryption, decryption, key generation, key derivation,
  digital signatures, MAC, HMAC, AEAD, hybrid encryption, or key agreement.
  Covers AES-GCM, ChaCha20-Poly1305, RSA-OAEP, RSA-PSS, ECDSA, Ed25519,
  X25519, ECDH, HKDF, Argon2, PBKDF2, scrypt, ECIES, HPKE, PEM/DER key formats,
  and the trait-based API design (SymmetricEncryptor, AeadEncryptor, Signer,
  KeyDeriver, KeyPairGenerator, KeyAgreement, etc.).
---

# Crypto Domain Specification

Follow the trait-based design pattern (consistent with kernel/codec) when implementing or modifying crypto code.

Key rules:
- New algorithms must implement the corresponding trait (e.g., AeadEncryptor for AEAD ciphers)
- Errors must use the defined error hierarchy — no ad-hoc error types
- All secret material must be zeroized on drop
- For detailed variant parameters, read `kernel/crypto/.ai/crypto-variants.md`

@../../kernel/crypto/.ai/context.md
