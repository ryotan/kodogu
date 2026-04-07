# ADR-0006: フロントエンドのアプリケーションディレクトリ構造

## ステータス

承認済み

## 背景

当初は `routes/`, `hooks/`, `components/`, `commands/` という最小構成を採用していた。
機能を追加するにあたり、どこに何を置くかが曖昧なままでは実装のたびに迷いが生じる。
フロントエンドのグローバルエラーハンドリングを検討する中で、エラー関連のユーティリティやコンポーネントの置き場所が定まらず、
先にアプリ全体の構造を決める必要があると判断した。

## 決定

Clean Architecture の層構造を基本に、以下のディレクトリ構成を採用する。

### ディレクトリ構成

```
app/src/
  routes/                         # TanStack Router（ファイルベースルーティング）
  features/
    <bounded-context-name>/       # ドメイン境界ごとに分割
      domain/                     # Domain Layer: Entities, Value Objects, Domain Behaviors, Domain Events, Adapter Interfaces
      usecases/                   # Use Cases Layer: Use Cases (Commands/Queries), Input Port, Output Port
                                  # （Form の型定義・バリデーションルールを含む）
      adapters/                   # Adapters Layer: UI components <-> UseCase (Controllers/Presenters)、UseCase <-> External (Gateways for Tauri IPC etc.)
      components/                 # Frameworks Layer: UI components <-> Design System
  shared/
    core/                         # 複数の shared/* から参照される基盤コード
      errors/
      messages/
      config/
      utils/
    commands/                     # Tauri IPC surface
    notifications/
    logging/
    validation/
    date/
  ui/                             # Design System (based on Radix UI)
  assets/
  (prototype)/                    # プロトタイプ（出荷しない）
  (dev)/                          # 開発専用ユーティリティ
```

### 依存方向

```
routes/      → features/, ui/
components/  → usecases/, adapters/, domain/, ui/
usecases/    → domain/
adapters/    → usecases/, domain/
shared/xxx   → shared/core/
ui/          → なし
```

`shared/*` はどの層からも参照してよい。なお、`shared/` 内の横断参照も許容するが、循環参照は許容できない。
複数の `shared/*` から参照されるコードは `shared/core/` への移動を検討する（必須ではなく、「core らしいか」という感覚で判断してよい）。

### Adaptersの役割

`adapters/` は UI と UseCase、UseCase と外部システムの橋渡しを担う Adapters 層（Clean Architecture における Interface Adapters 層）に対応する。

- **UI → UseCase**: `useForm()` を使ったフォームの実装、controller hooks など
- **UseCase → 外部**: Tauri IPC、REST、GraphQL など

`queries/`, `translators/`, `hooks/` のようなステレオタイプでサブディレクトリを分けない。
「1ファイル=1つの関心事」として、データ変換・クエリ定義・hooks を 1 つのファイルにまとめる。

```ts
// adapters/connections.ts — データ変換・クエリ定義・hooks をまとめて定義
const connectionsQuery = queryOptions({
  queryFn: async () => {
    const raw = await getConnectionsCommand()
    return raw.map(toConnectionProfile)
  },
})

export function useConnections() {
  return useQuery(connectionsQuery)
}
```

### Form の責務分担

Form の型定義とバリデーションルールは、UseCase の Input Port として `usecases/` に置く。
`useForm()` を使った入力状態管理は `adapters/` に置く。

```ts
// usecases/createConnection.ts — Input Port（何を受け付けるか）
export const CreateConnectionInput = type({
  name: "string > 0",
  port: "1 <= number.integer <= 65535",
})

// adapters/createConnection.ts — UseCase を React で実装された入力要素に組み込む
export function useCreateConnectionForm() {
  return useForm({
    validators: { onChange: CreateConnectionInput },
    onSubmit: ({ value }) => createConnectionUseCase(value),
  })
}
```

### shared/commands/ の位置づけ

Tauri コマンドは backend 側が API を決定する（将来的には自動生成の可能性もある）。
features/ ごとに独自のコマンドラッパーを定義するのではなく、`shared/commands/` に集約する。

### shared/ と ui/ の分離

`ui/` はデザインシステム（Radix ベースの UI コンポーネント群）であり、アプリ固有のビジネスロジックを持たない。
`shared/` はアプリ横断の関心事（エラー処理、ロギング、通知、バリデーション等）を扱う。
この 2 つを明確に分け、 `shared/` から `ui/` への依存を避けるため、`ui/` を `shared/` の外に置く。

## 結果

**よくなること:**

- 各ファイルの責務が Clean Architecture の層構造と対応するため、「どこに置くか」の判断基準が明確になる
- `hooks/` というディレクトリが消え、hooks が実装手段ではなく責務に対応するディレクトリに置かれる
- データ変換・クエリ・hooks を 1 ファイルにまとめるため、関連コードが複数ディレクトリに散らばらなくなる

**トレードオフ:**

- `adapters/` という名前は広すぎると感じる場面もある。しかし `translators/`, `queries/`, `hooks/` に分けると、関連するコードが複数ディレクトリに散って全体像が掴みにくくなる
- `shared/core/` への移動基準が感覚的であるため、判断がぶれる可能性がある
