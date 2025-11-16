# crypto-context 設計決定事項

## アーキテクチャ

- **構造**: Clean Architecture + DDD
- **レイヤー**: Domain → Use Cases → Adapters
- **プロジェクト配置**: `contexts/crypto/rust/`
- **独立crate**: workspace化は後回し

## 暗号化ライブラリ

- **選択**: RustCrypto (純Rust、モジュラー設計)
- **配置**: アダプター層のみで使用（ドメイン層は依存しない）

### 依存crate

```toml
# エラーハンドリング
snafu = "0.8"

# 暗号化（アダプター層）
rsa = "0.9"              # RSA鍵生成・暗号化
p256 = "0.13"            # ECDSA P-256
p384 = "0.13"            # ECDSA P-384
ed25519-dalek = { version = "2.1", features = ["pkcs8", "pem"] }  # Ed25519
pkcs8 = "0.10"           # PEM形式
aes-gcm = "0.10"         # AES暗号化
pbkdf2 = "0.12"          # 鍵導出
rand = "0.8"             # 乱数生成
base64 = "0.22"          # エンコーディング
zeroize = "1.7"          # メモリ安全性
```

## ドメインモデル

### 値オブジェクト（Newtype Pattern）

```rust
pub struct PublicKey(Vec<u8>);
pub struct PrivateKey(Vec<u8>);  // ZeroizeOnDrop
pub struct SymmetricKey(Vec<u8>); // ZeroizeOnDrop
pub struct Plaintext(Vec<u8>);
pub struct Ciphertext(Vec<u8>);
```

### 集約

- `KeyPair`: 公開鍵と秘密鍵のペア

### Traits（抽象）

```rust
pub trait KeyGenerator { ... }
pub trait Encryptor { ... }
pub trait Decryptor { ... }
```

## プロジェクト構造

```
contexts/crypto/rust/
├── src/
│   ├── domain/
│   │   ├── key_pair.rs
│   │   ├── values/
│   │   └── traits/
│   ├── usecases/
│   └── adapters/
│       └── rustcrypto/
└── Cargo.toml
```

## 対称鍵暗号化の仕様

### 鍵の形式
- **AES鍵**: Base64エンコード
- **パスワード文字列**: UTF-8そのまま

### 文字列を鍵として使用時のオプション
1. PBKDF2で対称鍵を導出
2. UTF-8バイト列を直接使用

## エラーハンドリング

- suzunari-error使用（rust-practices.mdに準拠）
- `#[suzunari_error]`マクロ + `#[snafu(visibility(pub))]`
- スタックトレースコンテキストの自動伝播
- シンプルなエラー型（reasonフィールドのみ）

## 実装状態（最終）

### 完了機能

- ✅ 鍵ペア生成（RSA 2048/4096, ECDSA P-256/P-384, Ed25519）
- ✅ 対称鍵生成（AES-128/AES-256）
- ✅ パスワード文字列生成（ランダム英数字）
- ✅ Base64インポート/エクスポート
- ✅ PBKDF2鍵導出（100,000イテレーション、SHA-256）
- ✅ AES-GCM暗号化/復号（nonce自動生成、12バイト）
- ✅ RSAハイブリッド暗号化/復号（RSA-PKCS1v15 + AES-256-GCM）
- ✅ 統合テスト（24テスト全パス）

### 暗号文形式

**対称暗号（AES-GCM）**:
```
[nonce (12 bytes) | ciphertext + auth_tag]
```

**非対称暗号（RSAハイブリッド）**:
```
[encrypted_key_len (2 bytes big-endian) | encrypted_aes_key | aes_ciphertext]
```

### 未実装（将来のスコープ）

- 署名/検証機能
- DER形式サポート
- ストリーミング暗号化
- クレデンシャルストア連携
