---
name: codec-domain
description: >-
  Domain specification for the codec encoding/decoding library (kernel/codec).
  Use when working with Hex, Base32, Base58, Base64, Base85, URL encoding
  (Percent Encoding), Quoted-Printable, or UTF-8 conversion. Covers the 6-layer API design
  (ZST, Enum, Custom, Static, Marker Traits, Trait Objects), BinaryCodec/TextCodec
  trait hierarchy, and error types (BinaryCodecError, TextCodecError).
---

# Codec Domain Specification

Follow the API design patterns defined in this specification when implementing or modifying codec code.

Key rules:
- New encodings must implement all 6 API layers (ZST, Enum, Custom, Static, Marker Traits, Trait Objects)
- Errors must use BinaryCodecError or TextCodecError — no ad-hoc error types

@../../kernel/codec/.ai/context.md
