# 非対称鍵暗号 コマンドリファレンス

OpenSSLとGPGによる非対称鍵暗号操作の具体的なコマンドを定義する。

## 関連ドキュメント

- `symmetric-crypto-concepts.md` : 対称暗号アルゴリズム、AEAD、KDFの概念
- `asymmetric-crypto-concepts.md` : アルゴリズム・鍵形式の分類
- `crypto-operations.md` : 操作インターフェースの抽象定義
- `crypto-variants.md` : 各操作の具体的パラメータ、鍵/署名サイズ

## 鍵生成 (KeyGeneration)

| 操作 | アルゴリズム | 曲線 | 出力形式 | OpenSSL | GPG |
|------|-------------|------|----------|---------|-----|
| RSA鍵生成 | RSA | - | PKCS#8 + PEM | `openssl genpkey -algorithm RSA -pkeyopt rsa_keygen_bits:2048 -out priv.pem` | `gpg --full-generate-key` (RSA選択) |
| EC鍵生成 (P-256) | ECC | P256 | PKCS#8 + PEM | `openssl genpkey -algorithm EC -pkeyopt ec_paramgen_curve:P-256 -out priv.pem` | `gpg --full-generate-key --expert` (ECDSA選択) |
| EC鍵生成 (P-384) | ECC | P384 | PKCS#8 + PEM | `openssl genpkey -algorithm EC -pkeyopt ec_paramgen_curve:P-384 -out priv.pem` | 同上 |
| Ed25519鍵生成 | EdDSA | Curve25519 | PKCS#8 + PEM | `openssl genpkey -algorithm ED25519 -out priv.pem` | `gpg --full-generate-key --expert` (EdDSA選択) |
| Ed448鍵生成 | EdDSA | Curve448 | PKCS#8 + PEM | `openssl genpkey -algorithm ED448 -out priv.pem` | - |
| X25519鍵生成 | ECDH | Curve25519 | PKCS#8 + PEM | `openssl genpkey -algorithm X25519 -out priv.pem` | (暗号化副鍵として自動生成) |
| X448鍵生成 | ECDH | Curve448 | PKCS#8 + PEM | `openssl genpkey -algorithm X448 -out priv.pem` | - |
| ElGamal鍵生成 | ElGamal | - | GPG独自 | - | `gpg --full-generate-key --expert` (ElGamal選択) |

※ ElGamalはGPGでは暗号化専用副鍵として使用可能。署名にはDSA/RSA/ECDSAを使用。
※ Ed448/X448はGPGでは未サポート（2024年時点）。

## 公開鍵抽出

| 操作 | 入力形式 | 出力形式 | OpenSSL | GPG |
|------|----------|----------|---------|-----|
| 公開鍵抽出 | PKCS#8 + PEM | SPKI + PEM | `openssl pkey -in priv.pem -pubout -out pub.pem` | `gpg --export -a <ID>` (ASCII armor) |
| 公開鍵抽出 (バイナリ) | PKCS#8 + PEM | SPKI + DER | `openssl pkey -in priv.pem -pubout -outform DER -out pub.der` | `gpg --export <ID>` |

## 暗号化 (Encryption)

| 操作 | アルゴリズム | スキーム | ハッシュ | OpenSSL | GPG |
|------|-------------|----------|----------|---------|-----|
| RSA-OAEP暗号化 | RSA | OAEP | SHA256 | `openssl pkeyutl -encrypt -pubin -inkey pub.pem -pkeyopt rsa_padding_mode:oaep -pkeyopt rsa_oaep_md:sha256 -in plain.txt -out cipher.bin` | - |
| RSA-OAEP復号 | RSA | OAEP | SHA256 | `openssl pkeyutl -decrypt -inkey priv.pem -pkeyopt rsa_padding_mode:oaep -pkeyopt rsa_oaep_md:sha256 -in cipher.bin -out plain.txt` | - |
| RSA-PKCS1v1.5暗号化 | RSA | PKCS1v15Enc | - | `openssl pkeyutl -encrypt -pubin -inkey pub.pem -pkeyopt rsa_padding_mode:pkcs1 -in plain.txt -out cipher.bin` | - |
| ハイブリッド暗号化 | (自動選択) | (自動) | - | - | `gpg --encrypt -r <ID> -o cipher.gpg plain.txt` |
| ハイブリッド復号 | (自動選択) | (自動) | - | - | `gpg --decrypt -o plain.txt cipher.gpg` |

## 署名 (DigitalSignature)

| 操作 | アルゴリズム | スキーム | ハッシュ | OpenSSL | GPG |
|------|-------------|----------|----------|---------|-----|
| RSA-PSS署名 | RSA | PSS | SHA256 | `openssl dgst -sha256 -sign priv.pem -sigopt rsa_padding_mode:pss -sigopt rsa_pss_saltlen:-1 -out sig.bin data.txt` | - |
| RSA-PSS検証 | RSA | PSS | SHA256 | `openssl dgst -sha256 -verify pub.pem -sigopt rsa_padding_mode:pss -signature sig.bin data.txt` | - |
| RSA-PKCS1v1.5署名 | RSA | PKCS1v15Sig | SHA256 | `openssl dgst -sha256 -sign priv.pem -out sig.bin data.txt` | - |
| RSA-PKCS1v1.5検証 | RSA | PKCS1v15Sig | SHA256 | `openssl dgst -sha256 -verify pub.pem -signature sig.bin data.txt` | - |
| ECDSA署名 (P-256) | ECDSA | - | SHA256 | `openssl dgst -sha256 -sign priv.pem -out sig.bin data.txt` | - |
| ECDSA検証 | ECDSA | - | SHA256 | `openssl dgst -sha256 -verify pub.pem -signature sig.bin data.txt` | - |
| Ed25519署名 | EdDSA | - | (内蔵) | `openssl pkeyutl -sign -inkey priv.pem -rawin -in data.txt -out sig.bin` | - |
| Ed25519検証 | EdDSA | - | (内蔵) | `openssl pkeyutl -verify -pubin -inkey pub.pem -rawin -in data.txt -sigfile sig.bin` | - |
| Ed448署名 | EdDSA | - | (内蔵) | `openssl pkeyutl -sign -inkey priv.pem -rawin -in data.txt -out sig.bin` | - |
| Ed448検証 | EdDSA | - | (内蔵) | `openssl pkeyutl -verify -pubin -inkey pub.pem -rawin -in data.txt -sigfile sig.bin` | - |
| Schnorr署名 (secp256k1) | Schnorr | - | SHA256 | Bitcoin Core等の専用ツール (BIP-340) | - |
| 署名 (自動) | (鍵に依存) | (自動) | (自動) | - | `gpg --sign -o data.sig data.txt` |
| 分離署名 | (鍵に依存) | (自動) | (自動) | - | `gpg --detach-sign -o data.sig data.txt` |
| クリア署名 | (鍵に依存) | (自動) | (自動) | - | `gpg --clear-sign -o data.asc data.txt` |
| 署名検証 | (鍵に依存) | (自動) | (自動) | - | `gpg --verify data.sig data.txt` |

※ Ed25519/Ed448の`-rawin`オプションはOpenSSL 3.0以降で利用可能。

## 鍵交換 (KeyExchange)

| 操作 | アルゴリズム | 曲線 | OpenSSL | GPG |
|------|-------------|------|---------|-----|
| ECDH共有秘密導出 | ECDH | P256 | `openssl pkeyutl -derive -inkey my_priv.pem -peerkey peer_pub.pem -out shared.bin` | - |
| X25519共有秘密導出 | ECDH | Curve25519 | `openssl pkeyutl -derive -inkey my_priv.pem -peerkey peer_pub.pem -out shared.bin` | - |
| X448共有秘密導出 | ECDH | Curve448 | `openssl pkeyutl -derive -inkey my_priv.pem -peerkey peer_pub.pem -out shared.bin` | - |

※ GPGは鍵交換を暗号化処理内部で自動実行（ECDH等）。直接コマンドなし。
※ 共有秘密は直接暗号化鍵として使用せず、HKDFなどのKDFで導出すること。

## フォーマット変換

| 操作 | 入力 | 出力 | OpenSSL |
|------|------|------|---------|
| PEM → DER | PEM | DER | `openssl pkey -in key.pem -outform DER -out key.der` |
| DER → PEM | DER | PEM | `openssl pkey -in key.der -inform DER -out key.pem` |
| PKCS#1 → PKCS#8 | PKCS#1 + PEM | PKCS#8 + PEM | `openssl pkcs8 -topk8 -in rsa_pkcs1.pem -nocrypt -out rsa_pkcs8.pem` |
| PKCS#8 → PKCS#1 | PKCS#8 + PEM | PKCS#1 + PEM | `openssl rsa -in rsa_pkcs8.pem -out rsa_pkcs1.pem` |
| PEM → JWK | PEM | JWK | `openssl pkey -in key.pem -pubout | pem-jwk` (※外部ツール) |

## 鍵情報確認

| 操作 | OpenSSL | GPG |
|------|---------|-----|
| 秘密鍵の詳細表示 | `openssl pkey -in priv.pem -text -noout` | `gpg --list-secret-keys --keyid-format long` |
| 公開鍵の詳細表示 | `openssl pkey -pubin -in pub.pem -text -noout` | `gpg --list-keys --keyid-format long` |
| RSA鍵パラメータ確認 | `openssl rsa -in priv.pem -text -noout` | - |
| EC曲線確認 | `openssl ec -in priv.pem -text -noout` | - |
| 鍵フィンガープリント | `openssl pkey -pubin -in pub.pem -outform DER \| openssl dgst -sha256` | `gpg --fingerprint <ID>` |
| 鍵の編集 | - | `gpg --edit-key <ID>` |

## GPG固有の注意点

- GPGは内部でアルゴリズム・スキームを自動選択するため、OpenSSLのような細かい制御は難しい
- GPGの暗号化は常にハイブリッド暗号（対称鍵で本文暗号化 + 非対称鍵でセッション鍵暗号化）
- GPGとOpenSSLの鍵形式は直接互換性がない（変換には中間ツールが必要）
- GPGの署名形式: embedded署名、detached署名、clearsign署名の3種類

## ポスト量子暗号

### ツールサポート状況

| ツール | ML-KEM | ML-DSA | SLH-DSA | 備考 |
|--------|:------:|:------:|:-------:|------|
| OpenSSL 3.5+ | △ | △ | △ | oqs-providerが必要 |
| liboqs | ✓ | ✓ | ✓ | Open Quantum Safe |
| BouncyCastle | ✓ | ✓ | ✓ | Java/C# |
| pqcrypto (Rust) | ✓ | ✓ | ✓ | Rustライブラリ |

※ 2024年時点で主要ツールへの統合は進行中

### liboqs + oqs-provider (OpenSSL 3.x) でのコマンド例

```bash
# oqs-provider のインストール後

# ML-KEM-768 鍵生成
openssl genpkey -provider oqsprovider -algorithm mlkem768 -out mlkem_priv.pem
openssl pkey -provider oqsprovider -in mlkem_priv.pem -pubout -out mlkem_pub.pem

# ML-DSA-65 鍵生成
openssl genpkey -provider oqsprovider -algorithm mldsa65 -out mldsa_priv.pem
openssl pkey -provider oqsprovider -in mldsa_priv.pem -pubout -out mldsa_pub.pem

# ML-DSA-65 署名
openssl pkeyutl -provider oqsprovider -sign -inkey mldsa_priv.pem -in data.txt -out sig.bin

# ML-DSA-65 検証
openssl pkeyutl -provider oqsprovider -verify -pubin -inkey mldsa_pub.pem -in data.txt -sigfile sig.bin

# SLH-DSA (SPHINCS+) 鍵生成
openssl genpkey -provider oqsprovider -algorithm sphincssha2128fsimple -out slhdsa_priv.pem
```

### ハイブリッドモード (従来型 + ポスト量子)

移行期間中は従来型とポスト量子を組み合わせたハイブリッドモードが推奨されています。

| 組み合わせ | 用途 | 備考 |
|-----------|------|------|
| X25519 + ML-KEM-768 | 鍵交換 | TLS 1.3で採用進行中 |
| ECDSA + ML-DSA | 署名 | 両方の署名を生成・検証 |
| RSA-3072 + ML-KEM | 鍵交換 | レガシー互換 |

```bash
# ハイブリッド鍵交換 (X25519 + ML-KEM-768)
# oqs-provider での例
openssl genpkey -provider oqsprovider -algorithm x25519_mlkem768 -out hybrid_priv.pem
```
