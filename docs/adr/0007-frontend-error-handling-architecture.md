# ADR-0007: フロントエンドのエラーハンドリング基本方針

## ステータス

承認済み

## 背景

KoDoGuはRustバックエンド（Tauri v2）とTypeScript製のReactフロントエンドで構成される道具箱型アプリ。Rust側のエラー戦略はADR-0004（Visitor Pattern + Application/Platform分類）で確立されているが、TypeScript側の設計判断はADRとして残されていなかった。

KDG-26（フロントエンドのグローバルエラーハンドリング方針決定・実装）の検討で以下を決める必要が出てきた。

- TS側のエラー型階層をどう設計するか
- エラーメッセージの責務（誰が、いつ、何を書くか）をどう定めるか
- 規約に沿わないエラーをユーザーにどう見せるか

なお、Rust→JavaScriptの構造化エラー伝播（IPC越境のwire format、cause chainの保持等）はKDG-27のスコープで、本ADRでは扱わない（別ADRに切り出す前提）。

### 要求される特性

1. **設計品質**: KoDoGuのショーケース哲学に沿った筋の通った設計
2. **言語横断の一貫性**: ADR-0004（Rust側）と揃った分類軸を持つ
3. **ユーザー向けと開発者向けの両立**: 同じエラーを、ユーザーには分かる言葉で、開発者にはデバッグに使える情報で見せる
4. **スケーラビリティ**: エラー型が増えても破綻しない実装パターン
5. **JavaScript/Reactのエラー機構との整合性**: stack trace、causeチェーン、ErrorBoundaryで自然に利用できる

## 検討した選択肢

### 1. エラー型の表現

#### 選択肢1-A: クラス階層

`RootError → ApplicationError / PlatformError → 個別エラー`をクラスの継承で表現する。

```ts
class RootError extends Error { ... }
class ApplicationError extends RootError {}
class PlatformError extends RootError {}
class ConfigNotLoadedError extends ApplicationError { ... }
```

#### 選択肢1-B: discriminated union

タグ付きデータとしてエラーを表現する。

```ts
type AppError =
  | { _tag: 'ConfigNotFound'; path: string }
  | { _tag: 'NetworkTimeout'; url: string }
  | { _tag: 'Unknown'; cause: unknown };
```

### 2. メッセージの責務

#### 選択肢2-A: パターンA（投げる側が日本語を書く）

`ApplicationError`等のmessageに、ユーザーに見せる自然な日本語を投げる側が直接書く。

```ts
throw new ApplicationError('設定ファイルが見つかりません');
```

#### 選択肢2-B: パターンB（メッセージID + i18nテーブル）

エラーは識別子（コード）を持ち、表示層が翻訳テーブルから引く。

```ts
throw new ApplicationError({code: 'CONFIG_NOT_FOUND', args: {path}});
// 表示時: i18n.t('errors.CONFIG_NOT_FOUND', {path})
```

#### 選択肢2-C: 中間（messageもコードも持つ）

両方を持たせ、表示時に選ぶ。

```ts
throw new ApplicationError('設定ファイルが見つかりません', {code: 'CONFIG_NOT_FOUND'});
```

### 3. 規約に沿わないエラーの扱い

ライブラリやランタイムが投げるエラー（`TypeError`等）は、私たちの規約の枠外にある。これをユーザー向け表示でどう扱うか。

#### 選択肢3-A: そのままmessageを出す

技術的な英語メッセージがそのままユーザーに見える。

#### 選択肢3-B: 定型文へのフォールバック

`RootError`を継承しないエラーは「予期しないエラーが発生しました」に置き換える。

## 比較表

### エラー型の表現（クラス階層 vs discriminated union）

| 評価軸 | クラス階層 | discriminated union |
|---|---|---|
| **型レベルの網羅検査** | ⚠️ 効かない（`instanceof`はランタイムでの判定） | ✅ 効く（`switch` + `never`で保証） |
| **型解析の負荷** | ✅ クラスごとに局所にとどまる | ❌ 大規模なunionで顕著、IDE応答性にも影響が出る |
| **ErrorBoundary互換性** | ✅ JSのErrorとして自然に利用できる | ❌ Errorに包むラッパー層が要る |
| **stack trace / causeチェーン** | ✅ JSのErrorの仕組みをそのまま使える | ❌ 別に用意する必要がある |
| **関数型寄りの記述との親和性** | ⚠️ `instanceof`ベース | ✅ パターンマッチ寄り |
| **Rust側（ADR-0004）との並行性** | ✅ Application/Platform分類がそのまま対応する | ⚠️ tag体系を独自に組み直す |
| **呼び出し側の部分的な分岐との相性** | ✅ 興味のあるエラーだけ`instanceof`で判定できる | ⚠️ union全体への参照があちこちに発生する |
| **エラー型が増えたときのスケール** | ✅ 各クラスが独立、影響が局所にとどまる | ⚠️ union規模に比例して影響が広がる |

### メッセージの責務（パターンA vs パターンB vs 中間）

| 評価軸 | パターンA（投げる側が日本語） | パターンB（コード + i18n） | 中間（messageもコードも持つ） |
|---|---|---|---|
| **実装の簡潔さ** | ✅ いちばんシンプル | ⚠️ 翻訳テーブルの維持コストがかかる | ❌ messageとコードの二重管理 |
| **単一言語アプリへの適合** | ✅ 過不足ない | ❌ 過剰 | ⚠️ もてあます |
| **多言語対応** | ⚠️ 拡張が必要 | ✅ 仕組みとしてそのまま使える | ✅ どちらでも使える |
| **ログ・観測との連携** | ⚠️ 文字列ベースのgrepしかできない | ✅ コードで集約しやすい | ✅ コードで集約できる |
| **投げる側の負荷** | ⚠️ 毎回日本語を書く | ✅ コードと引数を渡すだけ | ❌ messageもコードも両方を意識する |
| **ショーケースの観点** | ✅ kodoguの規模に合っている | ⚠️ 今の単一言語には過剰 | ⚠️ 今の単一言語には過剰 |

## 決定

### 1. エラー型の表現: クラス階層を採用

`RootError → ApplicationError / PlatformError → 個別エラー`のクラス階層を導入する。

**選択理由**:

- ADR-0004で確立したApplication/Platform分類は言語に依存しない設計原則であり、TypeScriptでも独立して意味を持つ
- discriminated unionはTypeScriptの型解析の負荷が大きく、エラー型が増えていくKoDoGuには合わない
- JavaScriptのErrorの仕組み（stack/cause/name/`instanceof`/ErrorBoundary）をそのまま利用できる
- 呼び出し側では「自分が興味のあるエラーだけ特別扱いし、残りはまとめて扱う」分岐が普通で、`instanceof`はこのパターンに素直に対応する

### 2. メッセージの責務: パターンA

`ApplicationError`等のmessageには、投げる側が自然な日本語を直接書く。

**選択理由**:

- KoDoGuは今のところ日本語だけのアプリで、パターンBの間接化は過剰
- 投げる側が文脈をいちばん持っているので、自然な日本語を書きやすい
- 多言語化が必要になったときに、messageに加えてコード（i18nキー）を持つ形へ拡張する余地はある
- 「中間案」は二重管理になり、書く側が毎回どちらに何を書くか考えることになるので、現時点では選ばない

### 3. 規約に沿わないエラー: 定型文へのフォールバック

`RootError`を継承していないエラーがユーザー向け表示に届いたら、定型文「予期しないエラーが発生しました」に置き換える。

**選択理由**:

- ユーザーに技術的な英語メッセージが漏れないことを保証する
- 投げる側が規約に従っているかを「`RootError`継承の有無」で判別できるので、表示層の判断が単純
- 開発者向けには情報を捨てない（開発ビルドの別表示でstack/cause/name等を残す）

### 4. KDG-27（Rust→JSの構造化エラー伝播）は別ADRで扱う

Rust→JSの構造化エラー伝播（IPC越境のwire format、`TauriIpcApplicationError`等の境界クラス、causeチェーンの保持等）は本ADRのスコープ外。KDG-27で議論し、別のADRとして起こす。

その際は本ADRで定めた階層に乗せる形で、IPC越境エラーを`ApplicationError`/`PlatformError`の派生クラスとして実装する見込み（詳細はKDG-27のコメント参照）。

## 実装ガイドライン

### 基底クラス: `RootError`

kodoguのすべてのエラーは`RootError`を継承する。コンストラクタで`this.name`を実際のクラス名に揃え、stack traceやログに型情報が乗るようにする。

```ts
export class RootError extends Error {
  constructor(message: string, options?: ErrorOptions) {
    super(message, options);
    this.name = this.constructor.name;
  }
}
```

### Application / Platform 分類

`RootError`の直下にApplication系とPlatform系を置く。中身を持たないmarker classとしてスタートし、必要が出てきたら共通の振る舞いを足す。

分類軸は「投げ手が誰か（私たちのコードか／ライブラリか）」ではなく「失敗の性質」で切る。広義のドメインの言葉で説明できる失敗はApplication、ランタイム・実行環境が応えられなかった失敗はPlatform。

```ts
/** アプリのドメインの言葉で意味付けできる失敗（規約違反、状態違反、入力不整合、不変条件違反等） */
export class ApplicationError extends RootError {}

/** ランタイム・実行環境が応えられなかった失敗（Tauri IPC、filesystem、network、ブラウザAPI不在等） */
export class PlatformError extends RootError {}
```

### 個別エラーの実装

具体的なエラーは`ApplicationError`か`PlatformError`のどちらかを継承する。構造化された文脈情報は`readonly`フィールドとして持つ。

```ts
// Application系: ドメインの言葉で意味付けできる失敗
export class ConfigNotLoadedError extends ApplicationError {
  constructor() {
    super('設定がまだ読み込まれていません');
  }
}

export class ConnectionStateError extends ApplicationError {
  constructor(public readonly currentState: ConnectionState) {
    super(`現在の状態（${currentState}）からは接続できません`);
  }
}

// Platform系: ランタイム・実行環境が応えられなかった失敗
export class BrowserApiUnavailableError extends PlatformError {
  constructor(public readonly apiName: string) {
    super(`${apiName}が利用できない環境です`);
  }
}
```

### 投げ方（パターンAの規約）

ユーザー向け表示に届きうる箇所では、`message`に自然な日本語を書く。原因となるエラーがあれば`cause`に詰めて伝播させる。

```ts
try {
  await loadConfigFromDisk();
} catch (cause) {
  throw new ConfigNotLoadedError({cause});
}
```

### 表示層での判別とフォールバック

表示層は「`RootError`を継承するエラーか、それ以外か」を`instanceof`で判別する。継承していれば`message`をそのまま出し、継承していなければ定型文に置き換える。

```ts
function getUserMessage(error: unknown): string {
  if (error instanceof RootError) {
    return error.message;  // パターンAの規約に沿っているので、messageを信頼してそのまま出す
  }
  return '予期しないエラーが発生しました';  // 規約外: フォールバック
}
```

開発ビルドでは、フォールバックに置き換える前のエラー情報（stack、cause、name）を別表示で見せる。`__DEV_TOOLS__`フラグで切り替える形を取る（詳細はrules/kodogu-app-frontend.md参照）。

### ErrorBoundaryとの連携

JSの仕様として、`Error`以外（文字列やオブジェクト等）もthrowできる。ReactのErrorBoundaryがこういう値を受けることもあるので、境界で`Error`に揃える必要がある。`fallbackRender`形式を採用して、境界でそのまま正規化する。

```tsx
const fallbackRender = ({error, resetErrorBoundary}: FallbackProps) => (
  <PopoverAppErrorPage
    error={error instanceof Error ? error : new ApplicationError(String(error))}
    reset={resetErrorBoundary}
  />
);
```

（このパターンの詳細はrules/kodogu-app-frontend.mdのError Boundaries節を参照）

## 結果

### 良い影響

- ✅ ADR-0004との並行性が取れ、Rust/TS両側で「Application（広義のドメインで説明できる失敗）」と「Platform（ランタイム・実行環境由来の失敗）」という共通の分類軸を持てる
- ✅ クラス階層を採ったことで、エラー型が増えても型解析の負荷が局所にとどまる
- ✅ パターンAを採ったことで、メッセージの責務がシンプルになる（投げる側が日本語を書き、表示層はそれを出すだけ）
- ✅ 規約外のフォールバックを置いたことで、ユーザーに技術的な英語メッセージが漏れない
- ✅ JSのError機構（causeチェーン、ErrorBoundary、stack trace）にそのまま乗る
- ✅ 投げる側・表示する側・開発ビルドでの表示、それぞれの責務が明確に分かれる

### トレードオフ

- ⚠️ クラス階層は型レベルの網羅検査が効かない → 必要な場面では`instanceof X`とデフォルトのフォールバックの組み合わせで対応する
- ⚠️ 多言語化のときにパターンAからの拡張が要る → messageに加えてコード（i18nキー）を持つ形へ移行することで対応する
- ⚠️ 規約外フォールバックは、投げる側の意図したメッセージをユーザー向け表示からは捨てる → 開発ビルドの別表示でstack/cause/name等は残して見せる
- ⚠️ クラス階層の維持コスト（基底クラスと個別エラーの定義）はゼロではない → discriminated unionと比べたときのトレードオフとして受け入れる

## 参考文献

- [ADR-0004: Rustにおけるエラーハンドリング戦略](./0004-error-handling-strategy-in-rust.md)
- [TypeScript Patterns for React Developers](https://dev.to/muhammad_zulqarnainakram/typescript-patterns-every-react-developer-should-know-in-2025-2264)
- [Better Error Handling in Typescript](https://edg.foo/posts/better-errors-ts)
- [Rust Error Handling in Tauri Commands — The Pattern That Actually Works](https://dev.to/hiyoyok/rust-error-handling-in-tauri-commands-the-pattern-that-actually-works-35le)
- KDG-26: フロントエンドのグローバルエラーハンドリング方針決定・実装
- KDG-27: Rust→JSエラー伝播の設計・実装（本ADRのスコープ外、別ADRで扱う）
- rules/typescript.md: Error Handling節（実装のhow）
- rules/kodogu-app-frontend.md: Error Boundaries節（ErrorBoundary連携のhow）
