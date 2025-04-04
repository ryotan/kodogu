# KoDoGu

KoDoGu（コドグ）は、私が使いたいツールを集めておくためのアプリです。Tauri v2とRustで構築されており、軽量かつ高速な操作を実現します。

## 主な機能

- Gitリポジトリ管理
- 未コミットの変更の可視化
- コードの差分表示
- その他の開発者支援機能

## インストール

```bash
# リポジトリのクローン
git clone https://github.com/ryotan/kodogu.git
cd kodogu/app

# 依存関係のインストール
bun install

# アプリケーションの実行
bun run tauri dev
```

## システム要件

- [Tauriの要件](https://tauri.app/guides/prerequisites/)を満たすシステム
- Windows 10/11、macOS 11以降、またはLinux

## 開発者向け情報

このプロジェクトに貢献する方法については、[CONTRIBUTING.md](./CONTRIBUTING.md)をご覧ください。

## ディレクトリ構造

```
kodogu/
├── app/                      # フロントエンドのソースコード
│   ├── src/                  # Reactコンポーネント、スタイル、ルーティングなど
│   ├── public/               # 静的アセット
│   ├── index.html            # HTMLエントリポイント
│   ├── package.json          # npm依存関係とスクリプト
│   └── vite.config.ts      # Viteの設定
├── app/src-tauri/            # Tauriバックエンドのソースコード
│   ├── src/                  # Rustのソースコード
│   ├── Cargo.toml            # Rustの依存関係
│   └── tauri.conf.json       # Tauriの設定
├── CONTRIBUTING.md         # 貢献ガイドライン
└── README.md                 # プロジェクトの概要とセットアップ手順
```

## ライセンス

[MIT](./LICENSE)
