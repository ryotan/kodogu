# ADR-0003: Hook テストに vitest-browser-react を使用する

## ステータス

承認済み

## 背景

React カスタム hook のテストには DOM 環境が必要となる。hook は React の reconciler 内でしか実行できないため、`renderHook` 等のユーティリティを通じてレンダリングする必要がある。

テストライブラリの選択肢として以下の2つが存在する：

1. **@testing-library/react**: React テストのデファクトスタンダード。jsdom または happy-dom 上で動作する
2. **vitest-browser-react**: Vitest Browser Mode 向けの公式コミュニティライブラリ。実ブラウザ上で動作する

### プロジェクトの現状

本プロジェクトでは Vitest のテストプロジェクトを以下のように構成している：

| プロジェクト | 環境 | 用途 |
|---|---|---|
| デフォルト | Node | 純粋関数の unit test |
| storybook | Browser (Playwright) | コンポーネントテスト（Storybook + addon-vitest） |

Hook テストは Node 環境では実行できない（DOM が必要）ため、新たなテスト環境の選定が必要となった。

## 決定

**Hook テストには vitest-browser-react の `renderHook` を使用し、Vitest Browser Mode（Playwright）で実行する。**

@testing-library/react + jsdom/happy-dom は採用しない。

### 比較

| 観点 | vitest-browser-react | @testing-library/react |
|---|---|---|
| 実行環境 | 実ブラウザ（Playwright） | jsdom / happy-dom（エミュレーション） |
| ブラウザ API | 本物（ResizeObserver 等） | エミュレーション（未実装 API あり） |
| `renderHook` | 非同期（`await renderHook()`） | 同期 |
| `act()` | `renderHook` 経由のみ。Locators API のリトライ機構で代替 | 明示的に `act()` で状態更新をラップ |
| 追加依存 | なし（既存の Playwright インフラを共有） | jsdom or happy-dom の追加が必要 |
| Cleanup | テスト開始前に実行（Browser UI での検査が可能） | テスト後に実行 |

### テスト構成（決定後）

| プロジェクト | 環境 | 用途 |
|---|---|---|
| デフォルト | Node | 純粋関数の unit test |
| browser | Browser (Playwright) | Hook の単体テスト（vitest-browser-react） |
| storybook | Browser (Playwright) | コンポーネントテスト（Storybook + addon-vitest） |

## 根拠

### 1. 環境の一貫性

Storybook コンポーネントテストと同じ Playwright インフラを共有するため、ブラウザ環境が統一される。jsdom / happy-dom という第三のランタイムを導入する必要がない。

### 2. ブラウザ API の忠実性

jsdom は `ResizeObserver`、`IntersectionObserver` 等の API を実装していない。hook がこれらの API に依存する場合、jsdom ではモックが必要だが、実ブラウザなら不要。テストの信頼性が高まる。

### 3. 追加依存の最小化

既に Storybook テストで `@vitest/browser-playwright` を使用しているため、新たなテスト依存（jsdom / happy-dom）の追加が不要。

### 4. hook テストの判断基準

hook を `renderHook` で単体テストするかどうかは、**hook が独立した契約（contract）を持つか**で判断する：

- **独立した契約がある**: hook の返り値・振る舞いが特定のコンポーネントに依存せず説明できる → `renderHook` で単体テスト
- **コンポーネントの内部詳細**: hook の振る舞いがコンポーネントの文脈でしか意味を成さない → コンポーネントテスト（Storybook）で検証

コンポーネントが複数の hook を使う場合、hook 単体テストにより組み合わせ爆発を回避し、コンポーネントテストはゴールデンパスに集中できる。

## 結果

### 良い影響

- **環境統一**: Node + Browser（Playwright）の2環境のみ。jsdom / happy-dom を導入しない
- **API 忠実性**: ブラウザ API のモック不要で、テストの信頼性が向上
- **インフラ共有**: Storybook テストと同じ Playwright セットアップを再利用

### 悪い影響

- **実行速度**: jsdom / happy-dom より実ブラウザの方が起動コストが高い
- **学習コスト**: `renderHook` が非同期（`await` 必須）、`act()` の扱いが異なる

### 軽減策

- 純粋関数はこれまで通り Node 環境でテスト（高速）。Browser に回すのは React 統合が必要な hook のみ
- `vitest-browser-react` の API は `@testing-library/react` と類似しており、学習コストは小さい

## 参考文献

- [vitest-browser-react](https://main.vitest.dev/api/browser/react) - Vitest 公式ドキュメント
- [vitest-community/vitest-browser-react](https://github.com/vitest-community/vitest-browser-react) - GitHub リポジトリ
- [Vitest Test Environment](https://vitest.dev/guide/environment) - 環境設定ガイド
- [Kent C. Dodds: How to test custom React hooks](https://kentcdodds.com/blog/how-to-test-custom-react-hooks)
