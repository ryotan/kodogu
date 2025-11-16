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
suzunari-error = { git = "https://github.com/ryotan/suzunari-error.git" }
suzunari-error-macro-impl = { git = "https://github.com/ryotan/suzunari-error.git" }
snafu = "0.8"

# 暗号化（アダプター層）
rsa = "0.9"              # RSA鍵生成・暗号化
p256 = "0.13"            # ECDSA P-256
p384 = "0.13"            # ECDSA P-384
ed25519-dalek = "2.1"    # Ed25519
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

- suzunari-error + snafu使用
- 階層化されたエラー型
- スタックトレース付き
