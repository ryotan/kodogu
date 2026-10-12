# KoDoGu アプリケーション

KoDoGuのデスクトップアプリです。macOSのメニューバーに常駐します。

## 技術スタック

- フロントエンド: React + TypeScript（TanStack Router、TanStack Query）
- バックエンド: Rust（Tauri v2）
- パッケージマネージャ: Bun

## 開発

[Tauriの前提条件](https://tauri.app/start/prerequisites/)を満たした上で、このディレクトリで実行します。

```bash
bun install
bun run tauri dev     # アプリを起動
bun run storybook     # Storybookを起動
bun run lint          # 静的解析
bun run test          # フロントエンドとRustのテスト
bun run tauri:build   # 配布用にビルド
```

開発の決まりごとは[CONTRIBUTING.md](../CONTRIBUTING.md)を参照してください。
