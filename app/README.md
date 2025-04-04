# KoDoGu アプリケーション

このディレクトリには、KoDoGuアプリケーションのソースコードが含まれています。

## 技術スタック

- フロントエンド: Vue.js + TypeScript
- バックエンド: Rust (Tauri v2)
- パッケージマネージャ: Bun

## 開発方法

### 前提条件

- [Rust](https://www.rust-lang.org/tools/install)
- [Node.js](https://nodejs.org/) または [Bun](https://bun.sh/)
- [Tauriの前提条件](https://tauri.app/guides/prerequisites/)

### 開発サーバーの起動

```bash
# 依存関係のインストール
bun install

# 開発サーバーの起動
bun run tauri dev
```

### ビルド

```bash
bun run tauri build
```

ビルドされたアプリケーションは `src-tauri/target/release/bundle` に出力されます。

## プロジェクト構造

```
app/
├── src/                  # フロントエンドのソースコード
│   ├── api/              # バックエンドAPIとの通信
│   ├── components/       # Vueコンポーネント
│   ├── stores/           # 状態管理
│   └── views/            # ページビュー
├── src-tauri/            # Rustバックエンドのソースコード
│   ├── src/              # Rustソース
│   └── tauri.conf.json   # Tauriの設定ファイル（v2形式）
├── public/               # 静的アセット
└── dist/                 # ビルドされたフロントエンド
```

## Tauri v2の主な変更点

- APIリファレンスが更新され、より一貫性のある設計になりました
- プラグインシステムが強化され、拡張性が向上しました
- モバイルプラットフォームのサポートが追加されました
- パフォーマンスとセキュリティが向上しました

詳細は[Tauriの公式ドキュメント](https://tauri.app/)を参照してください。

## テスト

```bash
# フロントエンドのテスト実行
bun run test

# Rustバックエンドのテスト実行
cd src-tauri && cargo test
```

## 詳細情報

より詳細な開発ガイドについては、プロジェクトルートの[CONTRIBUTING.md](../CONTRIBUTING.md)を参照してください。
