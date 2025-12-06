# crypto

開発者向けの暗号化ツールライブラリ。文字列の暗号化/復号、鍵生成、鍵導出機能を提供。

## 設計方針

**codec風シンプル設計**: `kernel/codec` と同じ設計パターンを採用。フラット構造、トレイトベース、操作別エラー分離。

## ユーザーストーリー

- [x] 開発者として、RSA/ECDSA/Ed25519の鍵ペアを生成したい
- [x] 開発者として、RSA公開鍵で大きなデータを暗号化したい（RSA/AES-GCM-256ハイブリッド）
- [x] 開発者として、RSA秘密鍵で暗号化された大きなデータを復号したい（RSA/AES-GCM-256ハイブリッド）
- [x] 開発者として、対称鍵（AES-128/AES-256）で文字列を暗号化/復号したい（AES-GCM）
- [ ] 開発者として、パスワードから安全な対称鍵を導出したい
- [ ] 開発者として、対称鍵でファイルを暗号化/復号したい
- [ ] 開発者として、秘密鍵でデータに署名したい
- [ ] 開発者として、公開鍵で署名を検証したい

## 提供機能

### 1. 暗号化/復号（Cipher）

#### 対称鍵暗号（SymmetricCipher）

**2つの使い方**:
```rust
fn example() {
  // 1. ZST型（コンパイル時ディスパッチ）- 鍵を別で管理
  let key = Aes256Key::from([42u8; 32]);
  let ciphertext = Aes256Gcm.encrypt(&plaintext, &key)?;
  let decrypted = Aes256Gcm.decrypt(&ciphertext, &key)?;

  // 2. Enum（ランタイム選択）- 鍵を内包
  let cipher = AesGcm::aes256([42u8; 32]);
  let ciphertext = cipher.encrypt(&plaintext)?;
  let decrypted = cipher.decrypt(&ciphertext)?;
}
```

**バリアント**: `Aes128Gcm`, `Aes256Gcm`（Enum: `AesGcm`）
**鍵型**: `Aes128Key`, `Aes256Key`（Enum: `AesKey`）

**暗号文形式**: `[nonce (12 bytes) | ciphertext + auth_tag]`

#### 非対称鍵暗号（AsymmetricCipher）

**2つの使い方**:
```rust
fn example() {
  // 1. ZST型
  let ciphertext = RsaHybrid2048.encrypt(&plaintext, &public_key)?;
  let decrypted = RsaHybrid2048.decrypt(&ciphertext, &private_key)?;

  // 2. Enum
  let cipher = RsaHybrid::rsa2048();
  let ciphertext = cipher.encrypt(&plaintext, &public_key)?;
  let decrypted = cipher.decrypt(&ciphertext, &private_key)?;
}
```

**バリアント**: `RsaHybrid2048`, `RsaHybrid4096`（Enum: `RsaHybrid`）（RSA + AES-256-GCM ハイブリッド）

**暗号文形式**: `[encrypted_key_len (2 bytes) | encrypted_aes_key | aes_ciphertext]`

---

### 2. 鍵生成（KeyGenerator）

#### 非対称鍵ペア生成

**2つの使い方（RSA）**:
```rust
fn example() {
  // 1. ZST型
  let key_pair = Rsa2048.generate()?;

  // 2. Enum
  let key_pair = RsaKeyGenerator::Rsa2048.generate()?;
}
```

**バリアント**:
- RSA: `Rsa2048`, `Rsa4096`（Enum: `RsaKeyGenerator`）
- ECDSA: `EcdsaP256`, `EcdsaP384`（Enum: `EcdsaKeyGenerator`）
- Ed25519: `Ed25519KeyGen`

#### 対称鍵生成

**2つの使い方**:
```rust
fn example() {
  // 1. ZST型
  let key = RandomAes256Key.generate()?;

  // 2. Enum
  let key = RandomAesKey::Aes256.generate()?;
}
```

**バリアント**: `RandomAes128Key`, `RandomAes256Key`（Enum: `RandomAesKey`）
**戻り値の鍵型**: `Aes128Key`, `Aes256Key`（Enum: `AesKey`）

---

### 3. 鍵導出（KeyDeriver）

**2つの使い方**:
```rust
fn example() {
  // 1. ZST型
  let key = Pbkdf2Aes256.derive_key(password, salt, 100_000)?;

  // 2. Enum
  let key = Pbkdf2::Aes256.derive_key(password, salt, 100_000)?;
}
```

**バリアント**:
- PBKDF2: `Pbkdf2Aes128`, `Pbkdf2Aes256`（Enum: `Pbkdf2`）
- Argon2: 同様
- Scrypt: 同様

**戻り値の鍵型**: `Aes128Key`, `Aes256Key`（Enum: `AesKey`）

---

### 4. 署名/検証（Signer/Verifier）

```
// バリアント
RsaSigner / RsaVerifier
EcdsaSigner / EcdsaVerifier
Ed25519Signer / Ed25519Verifier
```

## 機能要件

### 鍵ペア生成

- [ ] RSAアルゴリズムサポート（2048, 4096ビット）
- [ ] ECDSAアルゴリズムサポート（P-256, P-384）
- [ ] Ed25519アルゴリズムサポート
- [ ] PEM形式での出力
- [ ] DER形式での出力

### 暗号化/復号

- [ ] AES-128-GCM/AES-256-GCM対称暗号
- [ ] RSAハイブリッド暗号（RSA + AES-256-GCM）
- [ ] ChaCha20Poly1305サポート
- [ ] 文字列の暗号化/復号
- [ ] ファイルの暗号化/復号
- [ ] ストリーミング暗号化

### 署名/検証

- [ ] RSA-PSS署名
- [ ] ECDSA署名
- [ ] Ed25519署名
- [ ] 文字列への署名/検証
- [ ] ファイルへの署名/検証

### 鍵管理

- [ ] 非対称鍵ペアの生成
- [ ] 対称鍵のランダム生成
- [ ] PBKDF2-HMAC-SHA256による鍵導出
- [ ] Argon2鍵導出
- [ ] Scrypt鍵導出
- [ ] PEM形式インポート/エクスポート
- [ ] Base64形式インポート/エクスポート

## アーキテクチャ

### ファイル構成

```
kernel/crypto/src/
├── lib.rs          # トレイト、エラー型、値オブジェクト、公開API
├── aes.rs          # Aes128Gcm, Aes256Gcm実装
├── rsa.rs          # RsaHybridCipher実装
├── keygen.rs       # 全KeyGenerator実装
├── pbkdf2.rs       # Pbkdf2実装
├── argon2.rs       # Argon2実装
├── scrypt.rs       # Scrypt実装
└── signature.rs    # Signer/Verifier実装
```

**特徴**:
- フラット構造（domain/adapters分離なし）
- 共有型（値オブジェクト、トレイト、エラー）は lib.rs に集約（codec パターン）
- バリアントや定義が多い場合は別ファイル分割可（例: errors.rs）

### API パターン

**2つの相補的API（codec パターンの変形）**:
1. **ZST型**: `Aes128Gcm`, `Rsa2048` など（コンパイル時ディスパッチ）
2. **Enum**: `AesGcm`, `RsaKeyGenerator` など（ランタイム選択）

**特徴**:
- 暗号化Enum（`AesGcm`）は鍵を内包する設計（鍵管理と暗号化を一体化）
- 鍵生成/導出のEnumは鍵を返す設計（鍵生成と鍵管理を分離）

**codec との違い**: trait なし。暗号操作は鍵の型と結合するため、trait での抽象化は型安全性を損なう。

### 主要な型とバリアント

| 操作         | Enum                | ZST型バリアント                            | 鍵型                                        |
|------------|---------------------|--------------------------------------|-------------------------------------------|
| 対称鍵暗号      | `AesGcm`            | `Aes128Gcm`, `Aes256Gcm`             | `Aes128Key`, `Aes256Key` (Enum: `AesKey`) |
| 非対称鍵暗号     | `RsaHybrid`         | `RsaHybrid2048`, `RsaHybrid4096`     | `PublicKey`, `PrivateKey` (`KeyPair`)     |
| RSA鍵生成     | `RsaKeyGenerator`   | `Rsa2048`, `Rsa4096`                 | `KeyPair`                                 |
| ECDSA鍵生成   | `EcdsaKeyGenerator` | `EcdsaP256`, `EcdsaP384`             | `KeyPair`                                 |
| Ed25519鍵生成 | -                   | `Ed25519KeyGen`                      | `KeyPair`                                 |
| 対称鍵生成      | `RandomAesKey`      | `RandomAes128Key`, `RandomAes256Key` | `Aes128Key`, `Aes256Key` (Enum: `AesKey`) |
| PBKDF2導出   | `Pbkdf2`            | `Pbkdf2Aes128`, `Pbkdf2Aes256`       | `Aes128Key`, `Aes256Key` (Enum: `AesKey`) |

### エラー型

- `CipherError` - 暗号化/復号エラー
- `KeyGenerationError` - 鍵生成エラー
- `KeyDerivationError` - 鍵導出エラー
- `SignatureError` - 署名/検証エラー

## 技術スタック

### 暗号化ライブラリ

- **RustCrypto**: 純Rust実装、モジュラー設計
  - `rsa` - RSA鍵生成・暗号化
  - `p256`, `p384` - ECDSA
  - `ed25519-dalek` - Ed25519
  - `aes-gcm` - AES暗号化
  - `pbkdf2` - 鍵導出

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
- ハッシュ関数
