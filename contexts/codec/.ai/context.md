# codec

バイト列と文字列間の変換を行うエンコーディングライブラリ。UTF-8、Hex、Base32/58/64/85、URL Encoding、Quoted-Printableなど、多様なエンコーディング形式をサポートし、開発を支援する。

## ユーザーストーリー

- [ ] 開発者として、バイト列をUTF-8文字列にエンコード/デコードしたい
- [ ] 開発者として、バイト列をHex（16進数）文字列にエンコード/デコードしたい
- [ ] 開発者として、バイト列をBase32文字列にエンコード/デコードしたい（複数バリアント対応）
- [ ] 開発者として、バイト列をBase58文字列にエンコード/デコードしたい（Bitcoin/Flickrアルファベット）
- [ ] 開発者として、バイト列をBase64文字列にエンコード/デコードしたい（Standard/URL-safe/MIME）
- [ ] 開発者として、バイト列をBase85文字列にエンコード/デコードしたい（RFC 1924）
- [ ] 開発者として、文字列をURL-safeにエンコード/デコードしたい（Percent Encoding）
- [ ] 開発者として、メールデータをQuoted-Printableにエンコード/デコードしたい
- [ ] 開発者として、カスタムアルファベットでBase系エンコーディングを使用したい

## 機能要件

### UTF-8

- [x] バイト列からUTF-8文字列へのエンコード（Lossy/Strict）
- [x] UTF-8文字列からバイト列へのデコード
- [x] 無効なUTF-8シーケンスのエラーハンドリング

### Hex (Base16)

- [x] 小文字形式（0-9a-f）
- [x] 大文字形式（0-9A-F）
- [x] 大文字小文字混在対応（デコード時）
- [x] カスタムアルファベット対応
- [x] エンコード/デコード

### Base32

- [ ] RFC 4648標準形式（A-Z2-7）
- [ ] RFC 4648 Hex形式（0-9A-V）
- [ ] Crockford形式（大文字/小文字）
- [ ] カスタムアルファベット対応
- [ ] パディング処理

### Base58

- [ ] Bitcoinアルファベット
- [ ] Flickrアルファベット
- [ ] カスタムアルファベット対応
- [ ] エンコード/デコード

### Base64

- [ ] Standard形式（A-Za-z0-9+/）
- [ ] URL-safe形式（A-Za-z0-9-_）
- [ ] MIME形式（改行あり）
- [ ] カスタムアルファベット対応
- [ ] パディング処理

### Base85

- [ ] RFC 1924形式
- [ ] Ascii85形式（Adobe）
- [ ] Z85形式（ZeroMQ）
- [ ] エンコード/デコード

### URL Encoding (Percent Encoding)

- [ ] RFC 3986形式（スペース=%20）
- [ ] Form URL Encoded形式（スペース=+）
- [ ] Component形式（全非英数字をエンコード）
- [ ] エンコード/デコード

### Quoted-Printable

- [ ] MIME準拠エンコード
- [ ] デコード（Robust parsing mode）
- [ ] エラーハンドリング

## エラー設計

### エラー型の分離

- **`BinaryCodecError`**: バイナリ-テキスト変換（Hex, Base64等）のエラー
- **`TextCodecError`**: テキストエンコーディング（UTF-8等）のエラー
- suzunari-errorを使用した型安全なエラー階層

### エンコーディング識別

- **`BinaryCodecs`** enum: バイナリコーデックの種別（Hex, Base32, Base58, Base64, Url）
- **`TextCodecs`** enum: テキストコーデックの種別（Utf8）

### BinaryCodecError の共通エラーパターン

- `InvalidCharacter` - 無効な文字
- `InvalidLength` - 不正な長さ
- `InvalidPadding` - 不正なパディング
- `InvalidCharacterSet` - 不正なアルファベット定義

### エンコーディング固有エラー

- `HexOddLength` - Hex文字列の奇数長
- `UrlInvalidPercent` - 不正なパーセントエンコーディング（将来のurl_encoding用）
- `UrlInvalidUtf8` - URLデコード後の無効なUTF-8
- `DataEncodingSpec` - data-encodingライブラリの仕様エラー
- `DataEncodingDecode` - data-encodingライブラリのデコードエラー

### TextCodecError のエラーパターン

- `Utf8Invalid` - 無効なUTF-8シーケンス
- `InvalidSequence` - 無効なバイトシーケンス
- `UnmappableCharacter` - エンコード不可能な文字

## 設計方針

### API設計パターン

各エンコーディングモジュールは以下の多層APIアーキテクチャを提供する:

#### 1. Zero-Sized Types (ZST) - コンパイル時型安全性

コンパイル時にコーデックバリアントが決定している場合に使用。ゼロコスト抽象化を実現。

```rust
// 例: HexLower, HexUpper, HexCaseInsensitive
#[derive(Default, Copy, Debug, Clone, Eq, PartialEq)]
pub struct HexLower;

impl BinaryCodec for HexLower {
  fn encode(&self, bytes: &[u8]) -> String { /* ... */ }
  fn decode(&self, text: &str) -> Result<Vec<u8>, BinaryCodecError> { /* ... */ }
}

// 使用例
fn process_hex(data: &[u8]) -> String {
  let codec = HexLower;
  codec.encode(data)  // "48656c6c6f"
}
```

**特徴**:
- `size_of::<HexLower>() == 0` - メモリ消費ゼロ
- Copy可能 - 軽量な値渡し
- コンパイル時に完全に最適化される
- 関数引数として渡せる
- トレイトオブジェクト `&dyn BinaryCodec` として使用可能

#### 2. Enum Types - ランタイム柔軟性

ランタイムでコーデックバリアントを選択する必要がある場合に使用。

```rust
// 例: Hex enum
pub enum Hex {
    CaseInsensitive,
    Lower,
    Upper,
    Custom(HexCustom),
}

// 使用例
fn process_hex(config: &Config, bytes: &[u8]) -> String {
  let codec = match config.format {
    "upper" => Hex::Upper,
    "lower" => Hex::Lower,
    _ => Hex::CaseInsensitive,
  };
  codec.encode(bytes)
}
```

**特徴**:
- ランタイムでバリアント選択
- 設定ファイル、環境変数、ユーザー入力からの動的選択に対応
- ZST型よりオーバーヘッドがあるが柔軟性が高い

#### 3. Custom Types - カスタム設定

カスタムアルファベットなど、ランタイムデータを保持する必要がある場合に使用。

```rust
// 例: HexCustom
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct HexCustom {
    encoding: data_encoding::Encoding,  // private
}

impl HexCustom {
    pub fn new(alphabet: &str) -> Result<Self, BinaryCodecError> { /* ... */ }
}

// 使用例
fn process_hex(alphabet: &str, bytes: &[u8]) -> String {
  let codec = HexCustom::new("0123456789ABCDEF")?;
  codec.encode(bytes)
}
```

**特徴**:
- ランタイムデータを保持（カスタムアルファベット等）
- フィールドはprivate
- コンストラクタでバリデーション実施

#### 4. Static Methods - 便利API

最も一般的なユースケース向けの簡潔なAPI。

```rust
impl Hex {
  // デフォルト（簡潔さ優先）
  pub fn encode(bytes: &[u8]) -> String { /* ... */ }
  pub fn decode(text: &str) -> Result<Vec<u8>, BinaryCodecError> { /* ... */ }

  // 明示的バリアント（明確さ優先）
  pub fn encode_lower(bytes: &[u8]) -> String { /* ... */ }
  pub fn decode_lower(text: &str) -> Result<Vec<u8>, BinaryCodecError> { /* ... */ }
  pub fn encode_upper(bytes: &[u8]) -> String { /* ... */ }
  pub fn decode_upper(text: &str) -> Result<Vec<u8>, BinaryCodecError> { /* ... */ }

  // カスタム用コンストラクタ
  pub fn new(alphabet: &str) -> Result<Self, BinaryCodecError> { /* ... */ }
}
```

**特徴**:
- デフォルトメソッドと明示的メソッドの重複は許容
  - デフォルト: 簡潔さ優先（初心者向け）
  - 明示的: 明確さ優先（可読性重視）
- 内部でZST型を使用して実装

#### 5. Marker Traits - ポリモーフィズム

関連するコーデックをグループ化し、ジェネリック関数での使用を可能にする。

```rust
// Hex系コーデックのマーカー
pub trait HexCodec: BinaryCodec {}

// 自動実装
impl HexCodec for HexLower {}
impl HexCodec for HexUpper {}
impl HexCodec for HexCaseInsensitive {}
impl HexCodec for HexCustom {}
impl HexCodec for Hex {}

// ジェネリック関数での使用
fn process_hex<C: HexCodec>(codec: &C, data: &[u8]) -> String {
    codec.encode(data)
}

// 使用例
fn main() {
  let result = process_hex(&HexLower, b"Hello");
  let result = process_hex(&Hex::Upper, b"Hello");
}
```

**特徴**:
- ユーザー拡張可能（sealedではない）
- ジェネリック関数での型制約に使用
- トレイトオブジェクト `&dyn HexCodec` として使用可能

#### 6. Trait Objects - 動的ディスパッチ

コンパイル時に型が不明な場合の動的ディスパッチ。

```rust
// トレイトオブジェクトとして使用
fn encode_with_codec(codec: &dyn BinaryCodec, data: &[u8]) -> String {
    codec.encode(data)
}

// 使用例
fn main() {
  let result = encode_with_codec(&HexLower, b"Hello");
  let result = encode_with_codec(&HexUpper, b"Hello");
  let result = encode_with_codec(&Hex::Lower, b"Hello");
}
```

**特徴**:
- すべてのコーデック型で使用可能
- ランタイムの動的ディスパッチ
- vtableによるわずかなオーバーヘッド

### API選択ガイドライン

| 状況       | 推奨API                              | 理由        |
|----------|------------------------------------|-----------|
| クイックスタート | Static Methods (`Hex::encode()`)   | 最も簡潔      |
| 型が明確     | ZST (`HexLower`)                   | ゼロコスト、型安全 |
| 設定から選択   | Enum (`Hex::Upper`)                | ランタイム柔軟性  |
| カスタム設定   | Custom Types (`HexCustom::new()`)  | 拡張性       |
| ジェネリック関数 | Marker Traits (`impl HexCodec`)    | 型制約       |
| 型不明      | Trait Objects (`&dyn BinaryCodec`) | 動的ディスパッチ  |

### スレッドセーフティ

- `BinaryCodec` / `TextCodec` トレイト自体には `Send + Sync` を要求しない
- 各ZST型（`HexLower`, `HexUpper`等）は自動的に `Send + Sync`
- Custom型（`HexCustom`等）は実装による
  - `data_encoding::Encoding` は `Send + Sync` のため、`HexCustom` も `Send + Sync`

### 既存ライブラリの活用

- `data-encoding` クレートの既存定数を直接使用
  - 例: `HEXLOWER`, `HEXUPPER`, `HEXLOWER_PERMISSIVE`, `BASE32`, `BASE64`
- `bs58` クレートの`Alphabet`を直接使用
  - 例: `bs58::Alphabet::BITCOIN`, `bs58::Alphabet::FLICKR`
- ZST実装でこれらの定数を直接参照

### 可視性設計

- ZST型: フィールドなし、全てpublic
- Enum型: バリアントpublic、内部データprivate
- Custom型: フィールドprivate、メソッドpublic
- Static Methods: 全てpublic
- Trait実装メソッド: public（トレイト定義による）
- **理由**: 明確なAPI境界、内部実装の隠蔽

### 実装例（hex.rs）

```rust
// ZST型
#[derive(Default, Copy, Debug, Clone, Eq, PartialEq)]
pub struct HexLower;

#[derive(Default, Copy, Debug, Clone, Eq, PartialEq)]
pub struct HexUpper;

#[derive(Default, Copy, Debug, Clone, Eq, PartialEq)]
pub struct HexCaseInsensitive;

// Custom型
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct HexCustom {
  encoding: data_encoding::Encoding,
}

impl HexCustom {
  pub fn new(alphabet: &str) -> Result<Self, BinaryCodecError> { /* ... */ }
}

// Enum型
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Hex {
  CaseInsensitive,
  Lower,
  Upper,
  Custom(HexCustom),
}

impl Hex {
  // Static methods（便利API）
  pub fn encode(bytes: &[u8]) -> String {
    HexCaseInsensitive.encode(bytes)
  }

  pub fn encode_lower(bytes: &[u8]) -> String {
    HexLower.encode(bytes)
  }

  pub fn encode_upper(bytes: &[u8]) -> String {
    HexUpper.encode(bytes)
  }

  pub fn decode(text: &str) -> Result<Vec<u8>, BinaryCodecError> {
    HexCaseInsensitive.decode(text)
  }

  pub fn new(alphabet: &str) -> Result<Self, BinaryCodecError> {
    HexCustom::new(alphabet).map(Hex::Custom)
  }
}

// Marker trait
pub trait HexCodec: BinaryCodec {}

impl HexCodec for HexLower {}
impl HexCodec for HexUpper {}
impl HexCodec for HexCaseInsensitive {}
impl HexCodec for HexCustom {}
impl HexCodec for Hex {}

// BinaryCodec実装（各型に対して）
impl BinaryCodec for HexLower {
  fn encode(&self, bytes: &[u8]) -> String {
    HEXLOWER.encode(bytes)
  }
  fn decode(&self, text: &str) -> Result<Vec<u8>, BinaryCodecError> {
    // odd-length check + decode
  }
}
// ... 他の型も同様
```

## 非機能要件

- Rustライブラリとして実装
- CLI/GUIから利用可能な設計
- テスト可能な構造
- 外部クレート依存:
  - `data-encoding` (Base16/32/64)
  - `bs58` (Base58)
  - `base85` (Base85)
  - `percent-encoding` (URL Encoding)
  - `quoted_printable` (Quoted-Printable)
  - `suzunari-error` (エラーハンドリング)

## スコープ外

- 文字エンコーディング変換（UTF-8以外のShift-JIS、EUC-JPなど）
- データ圧縮（gzip、zstdなど）
- 暗号化処理（crypto-contextの責務）
- バイナリシリアライゼーション（MessagePack、Protobufなど）
