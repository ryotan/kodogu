# 暗号バリアント一覧

各暗号操作の具体的なアルゴリズム別パラメータ、鍵サイズ、署名サイズなどを定義する。

## 関連ドキュメント

- `symmetric-crypto-concepts.md` : 対称暗号アルゴリズム、AEAD、KDFの概念
- `asymmetric-crypto-concepts.md` : 非対称暗号アルゴリズム、鍵形式の分類
- `crypto-operations.md` : 操作インターフェースの抽象定義
- `cli-reference.md` : OpenSSL/GPGコマンド

---

## 対称鍵操作

### SymmetricEncryptor / Decryptor

| バリアント | 内部状態 | 鍵長 | ブロック/IV | 備考 |
|-----------|----------|------|------------|------|
| AES-128-CBC | SecretKey, IV | 128 bits | 16B/16B | パディング必要 |
| AES-256-CBC | SecretKey, IV | 256 bits | 16B/16B | パディング必要 |
| AES-128-CTR | SecretKey, Nonce | 128 bits | 16B/16B | パディング不要 |
| AES-256-CTR | SecretKey, Nonce | 256 bits | 16B/16B | パディング不要 |
| ChaCha20 | SecretKey, Nonce | 256 bits | -/12B | ストリーム暗号 |

### AEADEncryptor / Decryptor

| バリアント | 内部状態 | 鍵長 | Nonce長 | タグ長 | 備考 |
|-----------|----------|------|---------|--------|------|
| AES-128-GCM | SecretKey, Nonce, AAD? | 128 bits | 96 bits | 128 bits | 標準、高速 |
| AES-256-GCM | SecretKey, Nonce, AAD? | 256 bits | 96 bits | 128 bits | 標準、高速 |
| ChaCha20-Poly1305 | SecretKey, Nonce, AAD? | 256 bits | 96 bits | 128 bits | ソフトウェア向け |
| XChaCha20-Poly1305 | SecretKey, Nonce, AAD? | 256 bits | 192 bits | 128 bits | 拡張Nonce |
| AES-256-GCM-SIV | SecretKey, Nonce, AAD? | 256 bits | 96 bits | 128 bits | Nonce再利用耐性 |

### MACGenerator / Verifier

| バリアント | 内部状態 | 出力サイズ | 備考 |
|-----------|----------|-----------|------|
| HMAC-SHA256 | SecretKey | 256 bits | 推奨 |
| HMAC-SHA384 | SecretKey | 384 bits | 推奨 |
| HMAC-SHA512 | SecretKey | 512 bits | 推奨 |
| CMAC-AES | SecretKey (128/256 bits) | 128 bits | ブロック暗号ベース |
| GMAC | SecretKey, Nonce | 128 bits | GCMの認証部分 |
| Poly1305 | SecretKey (256 bits) | 128 bits | 一回限りの鍵 |

---

## 非対称鍵操作

### Signer / Verifier

| バリアント | 内部状態 | sign入力 | sign出力 | 署名サイズ | 備考 |
|-----------|----------|----------|----------|-----------|------|
| RSASigner (PSS) | RSAPrivateKey, Hash, SaltLength | message: bytes | signature: bytes | 鍵長/8 B | 2048bit鍵→256B |
| RSASigner (PKCS1v15) | RSAPrivateKey, Hash | message: bytes | signature: bytes | 鍵長/8 B | 同上 |
| ECDSASigner | ECPrivateKey, Curve, Hash | message: bytes | (r, s): bytes | 曲線サイズ×2 | P-256: 64B (raw) |
| Ed25519Signer | Ed25519PrivateKey | message: bytes | signature: bytes | 64B | hash内蔵 |
| Ed448Signer | Ed448PrivateKey | message: bytes | signature: bytes | 114B | hash内蔵、224-bit security |
| SchnorrSigner | ECPrivateKey, Curve | message: bytes | (R, s): bytes | 64B | BIP-340 |
| MLDSASigner | MLDSAPrivateKey, ParameterSet | message: bytes | signature: bytes | 2420/3309/4627B | ML-DSA-44/65/87 |
| SLHDSASigner | SLHDSAPrivateKey, ParameterSet | message: bytes | signature: bytes | 7856〜49856B | パラメータ依存 |

| バリアント | 内部状態 | verify入力 | verify出力 |
|-----------|----------|------------|------------|
| RSAVerifier (PSS) | RSAPublicKey, Hash, SaltLength | message, signature | bool |
| RSAVerifier (PKCS1v15) | RSAPublicKey, Hash | message, signature | bool |
| ECDSAVerifier | ECPublicKey, Curve, Hash | message, signature | bool |
| Ed25519Verifier | Ed25519PublicKey | message, signature | bool |
| Ed448Verifier | Ed448PublicKey | message, signature | bool |
| SchnorrVerifier | ECPublicKey, Curve | message, signature | bool |
| MLDSAVerifier | MLDSAPublicKey, ParameterSet | message, signature | bool |
| SLHDSAVerifier | SLHDSAPublicKey, ParameterSet | message, signature | bool |

### AsymmetricEncryptor / Decryptor (直接暗号化)

| バリアント | 内部状態 | encrypt入力 | encrypt出力 | 制限 |
|-----------|----------|-------------|-------------|------|
| RSAEncryptor (OAEP) | RSAPublicKey, Hash, MGF | plaintext: bytes | ciphertext: bytes | 鍵長 - 2*hashLen - 2 |
| RSAEncryptor (PKCS1v15) | RSAPublicKey | plaintext: bytes | ciphertext: bytes | 鍵長 - 11 |

| バリアント | 内部状態 | decrypt入力 | decrypt出力 |
|-----------|----------|-------------|-------------|
| RSADecryptor (OAEP) | RSAPrivateKey, Hash, MGF | ciphertext: bytes | plaintext: bytes |
| RSADecryptor (PKCS1v15) | RSAPrivateKey | ciphertext: bytes | plaintext: bytes |

※ ECCには直接暗号化なし

### KeyAgreement / KeyEncapsulation

従来の鍵交換（KeyAgreement）とポスト量子の鍵カプセル化（KEM）はAPIが異なります。

### 従来型 KeyAgreement

| バリアント | 内部状態 | derive入力 | derive出力 | 備考 |
|-----------|----------|------------|------------|------|
| DHKeyAgreement | DHPrivateKey, (p, g) | peerPublicKey | sharedSecret: bytes | パラメータ依存 |
| ECDHKeyAgreement | ECPrivateKey, Curve | peerPublicKey | sharedSecret: bytes | 曲線サイズ依存 |
| X25519KeyAgreement | X25519PrivateKey | peerPublicKey | sharedSecret: bytes | 32バイト固定 |
| X448KeyAgreement | X448PrivateKey | peerPublicKey | sharedSecret: bytes | 56バイト固定、224-bit security |

### ポスト量子 KEM

| バリアント | 内部状態 | encapsulate入力 | encapsulate出力 | 暗号文サイズ | 共有秘密 |
|-----------|----------|-----------------|-----------------|-------------|---------|
| MLKEMEncapsulator | MLKEMPublicKey, ParameterSet | (なし) | (ciphertext, sharedSecret) | 768/1088/1568B | 32B |
| FrodoKEMEncapsulator | FrodoKEMPublicKey, ParameterSet | (なし) | (ciphertext, sharedSecret) | 約9.7KB | 16/24/32B |
| NTRUEncapsulator | NTRUPublicKey, ParameterSet | (なし) | (ciphertext, sharedSecret) | 約1KB | 32B |
| McElieceEncapsulator | McEliecePublicKey | (なし) | (ciphertext, sharedSecret) | 128〜240B | 32B |

| バリアント | 内部状態 | decapsulate入力 | decapsulate出力 |
|-----------|----------|-----------------|-----------------|
| MLKEMDecapsulator | MLKEMPrivateKey, ParameterSet | ciphertext | sharedSecret |
| FrodoKEMDecapsulator | FrodoKEMPrivateKey, ParameterSet | ciphertext | sharedSecret |
| NTRUDecapsulator | NTRUPrivateKey, ParameterSet | ciphertext | sharedSecret |
| McElieceDecapsulator | McEliecePrivateKey | ciphertext | sharedSecret |

### HybridEncryptor / HybridDecryptor

### 従来型KEM

| バリアント | 内部状態 | encrypt入力 | encrypt出力構造 |
|-----------|----------|-------------|-----------------|
| RSA-KEM + AES-GCM | RSAPublicKey, Scheme, Hash, DEMAlgorithm | plaintext: bytes | (encapsulatedKey, ciphertext, tag) |
| ECIES | ECPublicKey, Curve, KDF, DEMAlgorithm | plaintext: bytes | (ephemeralPubKey, ciphertext, tag) |
| HPKE | PublicKey, KEM, KDF, AEAD | plaintext: bytes | (kemOutput, ciphertext) |

| バリアント | 内部状態 | decrypt入力 | decrypt出力 |
|-----------|----------|-------------|-------------|
| RSA-KEM + AES-GCM | RSAPrivateKey, Scheme, Hash, DEMAlgorithm | (encapsulatedKey, ciphertext, tag) | plaintext: bytes |
| ECIES | ECPrivateKey, Curve, KDF, DEMAlgorithm | (ephemeralPubKey, ciphertext, tag) | plaintext: bytes |
| HPKE | PrivateKey, KEM, KDF, AEAD | (kemOutput, ciphertext) | plaintext: bytes |

### ポスト量子KEM

| バリアント | 内部状態 | encrypt出力構造 | 備考 |
|-----------|----------|-----------------|------|
| ML-KEM + AES-GCM | MLKEMPublicKey, ParameterSet, DEMAlgorithm | (ciphertext, encryptedData, tag) | 推奨 |
| FrodoKEM + AES-GCM | FrodoKEMPublicKey, ParameterSet, DEMAlgorithm | (ciphertext, encryptedData, tag) | 保守的 |
| McEliece + AES-GCM | McEliecePublicKey, DEMAlgorithm | (ciphertext, encryptedData, tag) | 鍵サイズ大 |

### ハイブリッド (従来型 + ポスト量子)

| バリアント | 内部状態 | encrypt出力構造 | 備考 |
|-----------|----------|-----------------|------|
| X25519 + ML-KEM | ECPublicKey, MLKEMPublicKey, KDF, DEM | (ecdhOutput, kemOutput, ciphertext) | 移行期間推奨 |
| RSA + ML-KEM | RSAPublicKey, MLKEMPublicKey, KDF, DEM | (rsaOutput, kemOutput, ciphertext) | レガシー互換 |

### KeyPairGenerator

### 従来型

| バリアント | 内部状態 | generate出力 | 鍵サイズ (公開/秘密) |
|-----------|----------|--------------|---------------------|
| RSAKeyGenerator | keySize (2048/3072/4096) | RSAKeyPair | 可変 |
| DSAKeyGenerator | keySize, (p, q, g) | DSAKeyPair | 可変 (非推奨) |
| ECKeyGenerator | Curve | ECKeyPair | 曲線依存 |
| Ed25519KeyGenerator | (なし) | Ed25519KeyPair | 32B/32B |
| Ed448KeyGenerator | (なし) | Ed448KeyPair | 57B/57B |
| X25519KeyGenerator | (なし) | X25519KeyPair | 32B/32B |
| X448KeyGenerator | (なし) | X448KeyPair | 56B/56B |

### ポスト量子

| バリアント | 内部状態 | generate出力 | 鍵サイズ (公開/秘密) |
|-----------|----------|--------------|---------------------|
| MLKEMKeyGenerator | ParameterSet (512/768/1024) | MLKEMKeyPair | 1184B/2400B (ML-KEM-768) |
| MLDSAKeyGenerator | ParameterSet (44/65/87) | MLDSAKeyPair | 1952B/4032B (ML-DSA-65) |
| SLHDSAKeyGenerator | ParameterSet (128f/192f/256f等) | SLHDSAKeyPair | 32B/64B (128f) |
| FrodoKEMKeyGenerator | ParameterSet | FrodoKEMKeyPair | 約9.7KB/約19KB |
| NTRUKeyGenerator | ParameterSet | NTRUKeyPair | 約1KB |
| McElieceKeyGenerator | ParameterSet | McElieceKeyPair | 約261KB/約6KB |

---

## 共通操作

### KeyDeriver

共有秘密（SharedSecret）から用途別の鍵を導出する操作。SharedSecretを直接暗号化鍵として使用してはならない。

| バリアント | 内部状態 | derive入力 | derive出力 | 備考 |
|-----------|----------|------------|------------|------|
| HKDFDeriver | Hash (SHA256等) | sharedSecret, salt?, info | keyMaterial: bytes | RFC 5869、推奨 |
| ConcatKDFDeriver | Hash | sharedSecret, otherInfo | keyMaterial: bytes | NIST SP 800-56A |
| X963KDFDeriver | Hash | sharedSecret, sharedInfo | keyMaterial: bytes | ANSI X9.63 |

| 出力の用途分離例 | info/label | 導出される鍵 |
|-----------------|------------|-------------|
| 暗号化鍵 | "encryption" | AES鍵 |
| MAC鍵 | "authentication" | HMAC鍵 |
| IV導出 | "iv" | 初期化ベクトル |

### PasswordBasedKeyDeriver

パスワードから鍵を導出する操作。低エントロピーの入力から安全な鍵を生成する。

| バリアント | 内部状態 | derive入力 | derive出力 | 備考 |
|-----------|----------|------------|------------|------|
| PBKDF2-SHA256 | iterations, Hash | password, salt | keyMaterial: bytes | 反復回数10万以上推奨 |
| bcrypt | cost | password | hash: bytes | コスト12以上推奨 |
| scrypt | N, r, p | password, salt | keyMaterial: bytes | メモリハード |
| Argon2id | memory, time, parallelism | password, salt | keyMaterial: bytes | 推奨、ハイブリッド |

---

## 付録

### 内部状態の分類

| 種類 | 例 | 特徴 |
|------|-----|------|
| 鍵 | RSAPrivateKey, ECPublicKey, MLKEMPublicKey | 必須 |
| アルゴリズムパラメータ | Curve, (p, g) | アルゴリズム依存 |
| スキーム | PSS, OAEP, PKCS1v15 | RSA専用 |
| ハッシュ | SHA256, SHA384 | 選択可能（EdDSAは内蔵） |
| KDF | HKDF, PBKDF2 | ハイブリッド暗号で使用 |
| DEMアルゴリズム | AES-GCM, ChaCha20-Poly1305 | ハイブリッド暗号で使用 |
| ParameterSet | ML-KEM-768, ML-DSA-65 | ポスト量子で使用 |

### 従来型 vs ポスト量子の比較

| 観点 | 従来型 (RSA/ECC) | ポスト量子 |
|------|------------------|-----------|
| 鍵サイズ | 小さい (32B〜4KB) | 大きい (1KB〜261KB) |
| 署名サイズ | 小さい (64B〜512B) | 大きい (2.4KB〜49KB) |
| 計算速度 | 高速 | やや遅い |
| 量子耐性 | なし | あり |
| 成熟度 | 高い | 発展中 |
| API | KeyAgreement (derive) | KEM (encapsulate/decapsulate) |
