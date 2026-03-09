# ADR-0004: 永続化ストアにredbを採用する

## ステータス

承認済み

## 背景

Tray Popoverの実装でウィンドウ配置・タイミングに関わる定数（余白・デバウンス間隔等）とデザイントークン（色・スペーシング・フォント）が
RustコードとTypeScriptコードの複数箇所にハードコードされており、変更時の漏れや重複が生じていた。
これらを一元管理し、将来UIから変更できるようにするため、永続化方式を決定する必要がある。

KoDoGuは今後も設定値やアプリデータを追加していく予定であり（Tsunagiの接続設定等）、ここで決定した永続化方式がアプリ全体の基盤になる。

### 要件

- UI（設定画面）から変更できること
- RustとTypeScriptの両側から参照できること
- シングルユーザーのデスクトップアプリ（macOS）

### 選択肢

| 方式 | 特徴 |
|---------------------------------|------|
| [redb](https://docs.rs/redb) | Pure Rustの組み込みkey-valueストア。複数の名前付きテーブルをサポート。シリアライズ（[Serde](https://docs.rs/serde) + [rmp-serde](https://docs.rs/rmp-serde)）でネストした値も格納可能 |
| SQLite ([rusqlite](https://docs.rs/rusqlite), [SQLx](https://docs.rs/sqlx), [SeaORM](https://docs.rs/sea-orm)) | 組み込みRDBMS。リレーショナルデータ・複雑なJOINクエリに強い。C依存あり |
| TOML / JSONファイル | 人が直接編集できる。UIからの書き込みには不向き |
| Rust constのみ | コンパイル時定数。UIから変更不可 |

## 決定

**KoDoGuの永続化ストアとしてredbを採用する。** アプリの設定値・アプリデータの永続化が必要になった場合はredbに格納する。

- データベースファイル: `~/Library/Application Support/kodogu/kodogu.redb`
- 値のシリアライズ: [Serde](https://docs.rs/serde) + [rmp-serde](https://docs.rs/rmp-serde)（MessagePack、マップ形式）
- JSへの公開: Tauriコマンド経由
- 初回起動: Rustにハードコードされたデフォルト値をredbに書き込む

## 理由

**redbを選んだ理由:**

- **Pure Rust**: C依存（rusqlite）を避けられる。クロスコンパイル・CIがシンプルになる
- **key-valueで十分**: 設定値は単純なフィールド集合。接続設定等もネストしたstructとして格納でき、JOINが不要
- **FK制約不要**: シングルユーザー・シングルプロセスの環境では、Rustの型システムとドメインロジックで整合性を保証できる。DB側のFK制約に頼る必要がない

**シリアライザにrmp-serde（MessagePack・マップ形式）を選んだ理由:**

`bincode`・`bitcode`・`postcard`のような位置ベースのバイナリフォーマットはコンパクトだが、フィールドの順番と数をバイト列に含めないため、
フィールドの追加・削除・並べ替えがすべて破壊的変更になる。
`rmp-serde`のマップ形式は各フィールドの値をフィールド名（文字列キー）と対応付けて保存するため、
JSONと同様にSerdeの`#[serde(default)]`が有効に機能し、フィールドの追加・削除が後方互換になる。
設定値はレコード数が少なくサイズより保守性を優先すべきである。

**SQLiteを選ばなかった理由:**

- 現時点でリレーショナルなJOINクエリが必要なユースケースがない
- C依存がビルドの複雑さを増す
- スキーマ管理（migration）のオーバーヘッドがある

**TOML/JSONを選ばなかった理由:**

- UIからの書き込みには不向き（ファイルフォーマットの維持が煩雑）

## トレードオフ

- **型安全なクエリがない**: SQLの型付きクエリに相当するものは自前で実装が必要
- **`index.html`の制約**: Pre-ReactローディングスタイルはTauriコマンド呼び出し前に描画されるため、redbから読んだ値を注入できない。これらの値はハードコードのまま維持する（許容範囲）

## スキーママイグレーションの方針

### デフォルト値の定義

各structに`Default`トレイトを実装し、structに`#[serde(default)]`を付与する。
デフォルト値は`Default`の1箇所にまとめ、初回起動時の書き込みと欠損フィールドの補完の両方で参照する。

```rust
#[serde(default)]   // 欠損フィールドはAppConfig::default()の対応フィールド値で補完される
#[derive(Serialize, Deserialize)]
struct AppConfig {
    bottom_margin: f64,
    new_field: u64,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self { bottom_margin: 8.0, new_field: 300 }
    }
}
```

### 後方互換性のある変更（新コードが旧データを読める）

- **フィールドの追加**: 旧データにキーが存在しない場合、`AppConfig::default()`の対応フィールド値で補完される
- **フィールドの削除**: 旧データに残る不明なキーはSerdeが無視する

### 破壊的変更（バージョニングが必要）

- フィールドの**型変更**（`f64` → `u32`等）
- フィールドの**リネーム**（保存キーが変わるため、旧データのキーが不明フィールドとして捨てられる）

型変更・リネームが必要な場合はバージョン付きenumでラップする。**バージョニングenumは初日から導入しないと、後から追加すること自体が破壊的変更になる**ため、最初のバージョンから使用する：

```rust
// 初日から使用する
#[derive(Serialize, Deserialize)]
#[serde(tag = "version")]
enum StoredAppConfig {
    V1(AppConfig),
}

// 破壊的変更が必要になったとき: 既存のAppConfigをAppConfigV1にリネームし、新しいAppConfigを定義する
#[derive(Serialize, Deserialize)]
#[serde(tag = "version")]
enum StoredAppConfig {
    V1(AppConfigV1),
    V2(AppConfig),
}
```

起動時に旧variantを検出したら新variantに変換して書き直す。
