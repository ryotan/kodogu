# Anflug

シンプルなコマンドラインツールで、引数で受け取った文字列を指定されたファイルに追記します。

## 機能

### 基本機能
- 引数で受け取った文字列をファイルに追記
- ファイル名にパラメータ（日付など）を含めることが可能
- 正規表現でファイル内の追記場所を指定
- フォーマット機能（時刻やタグの追加）
- 複数行文字列対応（テンプレートのインデントに従って改行をインデント）
- ファイルからの入力対応
- ドライラン・バックアップ機能

### デフォルト動作
- デフォルトファイル: `notes-{date}.md`
- パターン未指定時: ファイル末尾に追加
- ファイル・ディレクトリ未存在時: 自動作成

### 設定
設定は以下の優先順位で適用されます：
1. コマンドライン引数
2. 環境変数
3. 設定ファイル

## 使用方法

```bash
# 基本的な使用方法
anflug "追記したい文字列"

# ファイルを指定
anflug -f "diary-{date}.md" "今日の出来事"

# 正規表現で追記場所を指定
anflug -p "## 日記" "今日は良い天気でした"

# フォーマットを指定
anflug -t "[{time}] {message}" "作業完了"
```

## コマンドライン引数

### 基本オプション
- `-f, --file <FILE>`: 出力ファイル名（パラメータ使用可能）
- `-p, --pattern <PATTERN>`: 追記場所を指定する正規表現
- `-t, --template <TEMPLATE>`: 出力フォーマット
- `-c, --config <CONFIG>`: 設定ファイルのパス

### 入力オプション
- `-i, --input-file <FILE>`: ファイルから文字列を読み込み

### 挿入位置オプション
- `--insert-before`: パターンの前の行に挿入（デフォルトは次の行）
- `--append-mode`: パターンマッチング無しでファイル末尾に追加

### 実行オプション
- `--dry-run`: 実行前にプレビュー表示（実際のファイル変更は行わない）
- `--backup`: 変更前にバックアップファイル作成（.bak拡張子）

### パターンマッチングオプション
- `--match-first`: 複数マッチ時は最初のみ（デフォルト）
- `--match-last`: 複数マッチ時は最後のみ
- `--match-all`: 複数マッチ時は全てに挿入

## パラメータ

ファイル名やテンプレートで使用できるパラメータ：

### 日時パラメータ
- `{date}`: 現在の日付 (YYYY-MM-DD)
- `{time}`: 現在の時刻 (HH:MM:SS)
- `{datetime}`: 現在の日時 (YYYY-MM-DD HH:MM:SS)
- `{timestamp}`: Unixタイムスタンプ

### 内容パラメータ
- `{message}`: 追記する文字列

### カスタム日時フォーマット
設定ファイルで日時フォーマットをカスタマイズ可能：
```yaml
date_formats:
  date: "YYYY-MM-DD"
  time: "HH:MM:SS"
  custom_date: "YYYY年MM月DD日"
  custom_time: "HH時MM分"
```

## 設定ファイル

TOML形式の設定ファイルをサポートしています。

```toml
# 基本設定
file = "notes-{date}.md"
template = "## {custom_date}\n- {message}"
pattern = "^## "

[date_formats]
date = "%Y-%m-%d"
time = "%H:%M:%S"
custom_date = "%Y年%m月%d日"
custom_time = "%H時%M分"

[date_formats.custom]
jp_time = "%H時%M分"

[error_handling]
no_match = "append_end"     # パターン未マッチ時: "append_end" | "error"
no_file = "create"          # ファイル未存在時: "create" | "error"
no_directory = "create"     # ディレクトリ未存在時: "create" | "error"

[execution]
backup_enabled = false      # 自動バックアップ
match_mode = "first"        # マッチモード: "first" | "last" | "all"
insert_position = "after"   # 挿入位置: "after" | "before"
```

## 例

### 基本的な使用例
```bash
# デフォルト設定でファイル末尾に追加
anflug "今日は良い天気でした"

# 特定のファイルに追記
anflug -f "diary-{date}.md" "今日の出来事"

# パターンマッチングで挿入位置指定
anflug -p "## 今日の出来事" "プロジェクトが完了した"
```

### 高度な使用例
```bash
# 複数行文字列の追記（テンプレートのインデントに従う）
anflug -t "  - タスク: {message}
    状態: 完了
    時刻: {time}" "新機能実装"

# ファイルから読み込んで追記
anflug -i notes.txt -f "diary-{date}.md"

# ドライランで確認
anflug --dry-run -p "## TODO" "新しいタスク"

# バックアップ付きで実行
anflug --backup -f "important.md" "重要な更新"

# 複数マッチの最後に追加
anflug --match-last -p "## 進捗" "作業完了"
```

### ログファイルへの追記
```bash
anflug -f "app.log" -t "[{datetime}] INFO: {message}" "サーバー起動完了"
```

### 環境変数での設定
```bash
export ANFLUG_FILE="todo-{date}.md"
export ANFLUG_PATTERN="## TODO"
export ANFLUG_TEMPLATE="- [ ] {message}"
anflug "新しいタスクを追加"
```

## 複数行文字列とインデント

テンプレートで複数行を指定する場合、テンプレート内のインデントが保持され、各行に適用されます：

```bash
# テンプレート例
anflug -t "- 項目: {message}
  詳細: 追加情報
  作成日時: {datetime}" "新しい項目"

# 結果
- 項目: 新しい項目
  詳細: 追加情報
  作成日時: 2024-01-15 14:30:00
```
