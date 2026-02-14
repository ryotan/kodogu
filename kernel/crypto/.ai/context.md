# crypto

開発者向けの暗号化ツールライブラリ。文字列の暗号化/復号、鍵生成、鍵導出機能を提供。

## 関連ドキュメント

- `crypto-operations.md` : 暗号操作の抽象インターフェース（trait 設計）
- `crypto-variants.md` : 各操作の具体的バリアントとパラメータ
- `symmetric-crypto-concepts.md` : 対称暗号アルゴリズム、AEAD、KDFの概念
- `asymmetric-crypto-concepts.md` : 非対称暗号アルゴリズム、鍵形式の分類
- `cli-reference.md` : OpenSSL/GPGコマンドリファレンス

## 設計方針

**trait ベース設計**: `kernel/codec` と同じ設計パターンを採用。

- **trait**: 暗号操作の抽象インターフェース（`crypto-operations.md` で定義）
- **ZST型**: コンパイル時ディスパッチ用の具体型（`Aes256Gcm`, `Rsa2048` など）
- **Enum**: ランタイム選択用（`AesGcm`, `RsaKeyGenerator` など）
- **操作別エラー分離**: 暗号化、鍵生成、鍵導出、署名それぞれに専用エラー型

## trait 一覧

`crypto-operations.md` で定義された全 trait。

### 対称鍵操作

| trait | 用途 | メソッド |
|-------|------|---------|
| SymmetricEncryptor | ブロック暗号暗号化 | `encrypt(plaintext, key, iv) -> ciphertext` |
| SymmetricDecryptor | ブロック暗号復号 | `decrypt(ciphertext, key, iv) -> plaintext` |
| AeadEncryptor | 認証付き暗号化 | `encrypt(plaintext, key, nonce, aad) -> (ciphertext, tag)` |
| AeadDecryptor | 認証付き復号 | `decrypt(ciphertext, tag, key, nonce, aad) -> plaintext` |
| MACGenerator | MAC生成 | `generate(message, key) -> tag` |
| MACVerifier | MAC検証 | `verify(message, tag, key) -> bool` |

### 非対称鍵操作

| trait | 用途 | メソッド |
|-------|------|---------|
| AsymmetricEncryptor | 直接暗号化 | `encrypt(plaintext, publicKey) -> ciphertext` |
| AsymmetricDecryptor | 直接復号 | `decrypt(ciphertext, privateKey) -> plaintext` |
| HybridEncryptor | KEM+DEM暗号化 | `encrypt(plaintext, publicKey) -> encryptedPackage` |
| HybridDecryptor | KEM+DEM復号 | `decrypt(encryptedPackage, privateKey) -> plaintext` |
| Signer | 署名 | `sign(message, privateKey) -> signature` |
| Verifier | 検証 | `verify(message, signature, publicKey) -> bool` |
| KeyAgreement | 鍵共有 | `derive(myPrivate, peerPublic) -> sharedSecret` |
| Encapsulator | KEM | `encapsulate(publicKey) -> (ciphertext, sharedSecret)` |
| Decapsulator | KEM | `decapsulate(ciphertext, privateKey) -> sharedSecret` |
| KeyPairGenerator | 鍵ペア生成 | `generate() -> keyPair` |

### 共通操作

| trait | 用途 | メソッド |
|-------|------|---------|
| KeyDeriver | 共有秘密から鍵導出 | `derive(sharedSecret, salt?, info) -> derivedKey` |
| PasswordBasedKeyDeriver | パスワードから鍵導出 | `derive(password, salt, params) -> derivedKey` |

## 機能要件（実装状況）

`crypto-variants.md` のバリアントを実装状況とともに一覧化。詳細は `crypto-variants.md` を参照。

### 対称鍵操作

#### SymmetricEncryptor / Decryptor
- [x] AES-128-CBC
- [x] AES-256-CBC
- [x] AES-128-CTR
- [x] AES-256-CTR
- [x] ChaCha20

#### AeadEncryptor / Decryptor
- [x] AES-128-GCM
- [x] AES-256-GCM
- [x] ChaCha20-Poly1305
- [x] XChaCha20-Poly1305
- [x] AES-256-GCM-SIV

#### MACGenerator / Verifier
- [x] HMAC-SHA256
- [x] HMAC-SHA384
- [x] HMAC-SHA512
- [x] CMAC-AES-128
- [x] CMAC-AES-256
- [x] GMAC
- [x] Poly1305

### 非対称鍵操作

#### Signer / Verifier
- [ ] RSA-PSS (2048)
- [ ] RSA-PSS (4096)
- [ ] RSA-PKCS1v15 (2048)
- [ ] RSA-PKCS1v15 (4096)
- [ ] ECDSA (P-256)
- [ ] ECDSA (P-384)
- [ ] Ed25519
- [ ] Ed448
- [ ] Schnorr (secp256k1)

#### AsymmetricEncryptor / Decryptor
- [ ] RSA-OAEP (2048)
- [ ] RSA-OAEP (4096)
- [ ] RSA-PKCS1v15 (2048)
- [ ] RSA-PKCS1v15 (4096)

#### HybridEncryptor / Decryptor
- [ ] RSA-KEM + AES-256-GCM (2048)
- [ ] RSA-KEM + AES-256-GCM (4096)
- [ ] ECIES (P-256)
- [ ] ECIES (P-384)
- [ ] HPKE

#### KeyAgreement
- [ ] DH
- [ ] ECDH (P-256)
- [ ] ECDH (P-384)
- [ ] X25519
- [ ] X448

#### KeyPairGenerator
- [ ] RSA (2048)
- [ ] RSA (3072)
- [ ] RSA (4096)
- [ ] DSA (非推奨)
- [ ] ECDSA (P-256)
- [ ] ECDSA (P-384)
- [ ] Ed25519
- [ ] Ed448
- [ ] X25519
- [ ] X448

### 共通操作

#### KeyDeriver
- [ ] HKDF-SHA256
- [ ] HKDF-SHA384
- [ ] ConcatKDF
- [ ] X963KDF

#### PasswordBasedKeyDeriver
- [ ] PBKDF2-SHA256
- [ ] bcrypt
- [ ] scrypt
- [ ] Argon2id

### 鍵形式
- [ ] PEM 形式インポート/エクスポート
- [ ] DER 形式インポート/エクスポート

## アーキテクチャ

### ファイル構成

```
kernel/crypto/src/
├── lib.rs          # trait 定義、エラー型、値オブジェクト、公開API
├── aead.rs         # AEADEncryptor/Decryptor 実装
├── symmetric.rs    # SymmetricEncryptor/Decryptor 実装
├── mac.rs          # MACGenerator/Verifier 実装
├── hybrid.rs       # HybridEncryptor/Decryptor 実装
├── asymmetric.rs   # AsymmetricEncryptor/Decryptor 実装
├── signature.rs    # Signer/Verifier 実装
├── keygen.rs       # KeyPairGenerator 実装
├── kdf.rs          # KeyDeriver, PasswordBasedKeyDeriver 実装
└── agreement.rs    # KeyAgreement, Encapsulator/Decapsulator 実装
```

### エラー型

| エラー型 | 用途 |
|---------|------|
| EncryptionError | 暗号化エラー |
| DecryptionError | 復号エラー |
| KeyGenerationError | 鍵生成エラー |
| KeyDerivationError | 鍵導出エラー |
| SignatureError | 署名/検証エラー |
| KeyAgreementError | 鍵交換エラー |

## 技術スタック

### 暗号化ライブラリ

- **RustCrypto**: 純Rust実装、モジュラー設計
  - `aes-gcm` - AES-GCM
  - `chacha20poly1305` - ChaCha20-Poly1305
  - `rsa` - RSA鍵生成・暗号化
  - `p256`, `p384` - ECDSA/ECDH
  - `ed25519-dalek` - Ed25519
  - `x25519-dalek` - X25519
  - `pbkdf2` - PBKDF2
  - `argon2` - Argon2
  - `hkdf` - HKDF
  - `hmac` - HMAC

### その他

- `snafu` + `suzunari-error` - エラーハンドリング
- `zeroize` - メモリ安全性
- `rand` - 暗号学的乱数生成

## 非機能要件

- Rustライブラリとして実装
- CLI/GUIから利用可能な設計
- テスト可能な構造（ラウンドトリップテスト）
- codec設計パターンの踏襲

## スコープ外

- 証明書管理
- ネットワーク通信
- TLS/SSL実装
- ハッシュ関数（単体）
- ポスト量子暗号（ML-KEM, ML-DSA 等）
