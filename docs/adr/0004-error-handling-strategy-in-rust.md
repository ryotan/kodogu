# ADR-0004: Rustにおけるエラーハンドリング戦略

## ステータス

承認済み

## 背景

### プロジェクトのエラーハンドリング原則

[coding-practices.md](../.ai/coding-practices.md#make-errors-explicit)で定義された原則：

> - **Default to exceptions** with a global error handling system for centralized management
> - Use Result/Either types **only when** callers must handle errors differently based on the error type

この原則の意図：
- **集中管理**: エラー処理ロジック（ログ、メトリクス、レスポンス生成）を一箇所に集約
- **ドメインの純粋性**: ドメイン層はプレゼンテーション層（HTTP/CLI）の詳細を知らない
- **一貫性**: すべてのエラーを統一的に扱う

### Rustでの課題

Rustには例外機構がない。`Result<T, E>`型でエラーを表現する：

```rust
fn find_user(id: &str) -> Result<User, UserError> { ... }
```

この設計により：

1. **型による表現**: エラーは型システムで表現される
2. **明示的な伝播**: `?`演算子による明示的なエラー伝播
3. **実行時型判定の制限**: `Box<dyn Error>`から具体型への判定が困難

**問題：「集中管理」をどう実現するか？**

例外を持つ言語では：
```kotlin
// エラー定義は分散
class UserNotFoundError : Exception()
class AuthError : Exception()

// Global handlerで集中管理
@ExceptionHandler
fun handleError(e: Exception) = when(e) {
    is UserNotFoundError -> HttpResponse.notFound()
    is AuthError -> HttpResponse.unauthorized()
}
```

Rustでは型システムの制約により、異なるアプローチが必要。

### 要求される特性

1. **集中管理**: エラー処理ロジックを一箇所に集約
2. **正しい依存関係**: app → domain（逆向きの依存を避ける）
3. **ドメイン層の純粋性**: ドメインエラーはHTTP/CLIを知らない
4. **複雑度に応じた段階性**: シンプルなケースから始め、必要に応じて高度化

## 検討した選択肢

### 選択肢1: 統一エラー型

すべてのエラーを単一enumに定義：

```rust
use suzunari_error::*;

#[suzunari_error]
pub enum RootError {
    #[snafu(display("User not found: {id}"))]
    UserNotFound { id: String },

    #[snafu(display("User already exists: {email}"))]
    UserAlreadyExists { email: String },

    #[snafu(display("Authentication failed"))]
    AuthUnauthorized,

    #[snafu(display("Database connection failed"))]
    DbConnectionFailed,
}

// Global handler
impl RootError {
    pub fn to_http_response(&self) -> HttpResponse {
        match self {
            RootError::UserNotFound { .. } => HttpResponse::not_found(...),
            RootError::UserAlreadyExists { .. } => HttpResponse::conflict(...),
            RootError::AuthUnauthorized => HttpResponse::unauthorized(...),
            RootError::DbConnectionFailed => HttpResponse::internal_server_error(...),
        }
    }
}
```

**評価：**
- ✅ 集中管理：完全。すべてのマッピングが一箇所
- ✅ シンプル：型変換が最小限
- ✅ コンパイル時チェック：網羅性が保証される
- ❌ ドメイン層の純粋性：低い。すべてがアプリケーション層の関心事と混在
- ❌ スケーラビリティ：enumが肥大化（50種類を超えると管理困難）

### 選択肢2: モジュール分離 + Trait委譲

各モジュールが独自のエラー型を持ち、共通traitで変換：

```rust
// ドメイン層
pub enum UserError {
    NotFound { id: String },
    AlreadyExists { email: String },
}

pub enum AuthError {
    Unauthorized,
    TokenExpired,
}

// 変換trait
trait IntoHttpResponse {
    fn to_http_response(&self) -> HttpResponse;
}

// 各ドメインエラーが実装
impl IntoHttpResponse for UserError {
    fn to_http_response(&self) -> HttpResponse {
        match self {
            UserError::NotFound { .. } => HttpResponse::not_found(...),
            UserError::AlreadyExists { .. } => HttpResponse::conflict(...),
        }
    }
}

impl IntoHttpResponse for AuthError {
    fn to_http_response(&self) -> HttpResponse {
        match self {
            AuthError::Unauthorized => HttpResponse::unauthorized(...),
            AuthError::TokenExpired => HttpResponse::unauthorized(...),
        }
    }
}

// トップレベル
pub enum RootError {
    User(UserError),
    Auth(AuthError),
}

impl IntoHttpResponse for RootError {
    fn to_http_response(&self) -> HttpResponse {
        match self {
            RootError::User(e) => e.to_http_response(),
            RootError::Auth(e) => e.to_http_response(),
        }
    }
}
```

**評価：**
- ⚠️ 集中管理：中程度。マッピングロジックが各モジュールに分散
- ❌ ドメイン層の純粋性：低い。ドメインエラーがHTTPレスポンスを知る必要がある
- ❌ 依存関係：逆転している（domain → app）
- ⚠️ スケーラビリティ：中程度

### 選択肢3: Type Erasure + Downcast

型消去と実行時ダウンキャスト：

```rust
// ドメイン層（純粋）
use suzunari_error::*;

#[suzunari_error]
pub enum UserError {
    #[snafu(display("User not found: {id}"))]
    NotFound { id: String },

    #[snafu(display("User already exists: {email}"))]
    AlreadyExists { email: String },
}
// std::error::Errorのみ実装。HTTPを知らない

// アプリケーション層
use std::any::Any;

pub struct RootError {
    inner: Box<dyn std::error::Error + Send + Sync>,
}

impl RootError {
    pub fn new<E: std::error::Error + Send + Sync + 'static>(error: E) -> Self {
        Self { inner: Box::new(error) }
    }

    fn downcast_ref<E: std::error::Error + 'static>(&self) -> Option<&E> {
        (self.inner.as_ref() as &dyn Any).downcast_ref::<E>()
    }
}

impl<E: std::error::Error + Send + Sync + 'static> From<E> for RootError {
    fn from(error: E) -> Self {
        Self::new(error)
    }
}

// Global handler
fn global_error_handler(error: RootError) -> HttpResponse {
    // 横断的関心事
    log::error!("Error: {}", error.inner);

    // ダウンキャストしてマッピング
    if let Some(e) = error.downcast_ref::<UserError>() {
        return match e {
            UserError::NotFound { id } =>
                HttpResponse::not_found(json!({
                    "error": "USER_NOT_FOUND",
                    "message": format!("User {} not found", id),
                })),
            UserError::AlreadyExists { email } =>
                HttpResponse::conflict(json!({
                    "error": "USER_ALREADY_EXISTS",
                    "message": format!("User {} already exists", email),
                })),
        };
    }

    if let Some(e) = error.downcast_ref::<AuthError>() {
        return match e {
            AuthError::Unauthorized =>
                HttpResponse::unauthorized(json!({
                    "error": "AUTH_UNAUTHORIZED",
                })),
            AuthError::TokenExpired =>
                HttpResponse::unauthorized(json!({
                    "error": "AUTH_TOKEN_EXPIRED",
                })),
        };
    }

    // Fallback
    HttpResponse::internal_server_error(json!({
        "error": "INTERNAL_SERVER_ERROR",
    }))
}
```

**評価：**
- ✅ 集中管理：高い。マッピングロジックがglobal_handlerに集中
- ✅ ドメイン層の純粋性：高い。std::error::Errorのみ実装
- ✅ 依存関係：正しい（app → domain）
- ⚠️ ボイラープレート：エラー型ごとにif-letとmatchが必要。variant追加時は`_`でfallback可能
- ⚠️ コンパイル時チェック：部分的。ダウンキャスト後のmatchでのみ網羅性チェック
- ⚠️ 実行時コスト：型判定がある（通常は問題にならない）

### 選択肢4: Visitor Pattern（エラーカテゴリ別）

エラーをカテゴリに分類し、Visitorで処理：

```rust
// ドメイン層（純粋）
use suzunari_error::*;

#[suzunari_error]
pub enum UserError {
    #[snafu(display("User not found: {id}"))]
    NotFound { id: String },

    #[snafu(display("User already exists: {email}"))]
    AlreadyExists { email: String },
}

// アプリケーション層
// エラーカテゴリ（固定・少数）
trait ErrorPresenter {
    type Output;

    fn present_not_found(&self, message: String) -> Self::Output;
    fn present_conflict(&self, message: String) -> Self::Output;
    fn present_unauthorized(&self, message: String) -> Self::Output;
    fn present_bad_request(&self, message: String) -> Self::Output;
    fn present_internal_error(&self, message: String) -> Self::Output;
}

trait PresentableError {
    fn present_with<P: ErrorPresenter>(&self, presenter: &P) -> P::Output;
}

// Extension trait（アプリ層で実装）
impl PresentableError for UserError {
    fn present_with<P: ErrorPresenter>(&self, presenter: &P) -> P::Output {
        match self {
            UserError::NotFound { id } =>
                presenter.present_not_found(format!("User {} not found", id)),
            UserError::AlreadyExists { email } =>
                presenter.present_conflict(format!("User {} already exists", email)),
        }
    }
}

impl PresentableError for AuthError {
    fn present_with<P: ErrorPresenter>(&self, presenter: &P) -> P::Output {
        match self {
            AuthError::Unauthorized =>
                presenter.present_unauthorized("Authentication required".to_string()),
            AuthError::TokenExpired =>
                presenter.present_unauthorized("Token has expired".to_string()),
        }
    }
}

// HTTP用Presenter
struct HttpErrorPresenter;

impl ErrorPresenter for HttpErrorPresenter {
    type Output = HttpResponse;

    fn present_not_found(&self, message: String) -> HttpResponse {
        HttpResponse::not_found(json!({
            "error": "NOT_FOUND",
            "message": message,
        }))
    }

    fn present_conflict(&self, message: String) -> HttpResponse {
        HttpResponse::conflict(json!({
            "error": "CONFLICT",
            "message": message,
        }))
    }

    // ... 他のカテゴリ
}

// CLI用Presenter
struct CliErrorPresenter;

impl ErrorPresenter for CliErrorPresenter {
    type Output = (i32, String);

    fn present_not_found(&self, message: String) -> (i32, String) {
        (1, format!("Error: {}", message))
    }

    fn present_conflict(&self, message: String) -> (i32, String) {
        (1, format!("Error: {}", message))
    }

    // ... 他のカテゴリ
}

// Global handler
fn http_error_handler(error: RootError) -> HttpResponse {
    log::error!("Error: {:?}", error);
    error.present_with(&HttpErrorPresenter)
}

fn cli_error_handler(error: RootError) -> (i32, String) {
    let (code, msg) = error.present_with(&CliErrorPresenter);
    eprintln!("{}", msg);
    (code, msg)
}
```

**評価：**
- ✅ 集中管理：高い。すべて（trait/出力形式/PresentableError実装）をアプリ層の同一ファイルに定義可能
- ✅ 構造化：カテゴリと出力形式で明確に構造化される
- ✅ ドメイン層の純粋性：高い（アプリ層でPresentableError実装）
- ✅ 依存関係：正しい（app → domain）
- ✅ 複数出力形式：Presenter実装を追加するだけ
- ⚠️ ボイラープレート：エラー型ごとにPresentableError implとmatchが必要。variant追加時は`_`でfallback可能
- ⚠️ ErrorPresenterメソッド数：エラーカテゴリ数（5〜10個程度で安定）

## パターンの比較

### 全体比較表

| 評価軸            | 統一エラー型                       | Trait委譲                         | Type Erasure                               | Visitor Pattern                                           |
|----------------|------------------------------|---------------------------------|--------------------------------------------|-----------------------------------------------------------|
| **集中管理**       | ✅ 完全<br>すべてのマッピングが一箇所        | ❌ 低い<br>各モジュールに分散               | ✅ 高い<br>global_handlerに集中                  | ✅ 高い<br>アプリ層の同一ファイルに定義可能                                  |
| **構造化**        | -                            | ⚠️ モジュールごとに分散                   | ⚠️ if-elseの羅列                              | ✅ カテゴリと出力形式で構造化                                           |
| **ドメイン層の純粋性**  | ❌ 低い<br>アプリ層の関心事と混在          | ❌ 低い<br>HTTPレスポンスを知る必要          | ✅ 高い<br>std::error::Errorのみ                | ✅ 高い<br>std::error::Errorのみ                               |
| **依存関係**       | ❌ 曖昧<br>分離なし                 | ❌ 逆転<br>domain → app            | ✅ 正しい<br>app → domain                      | ✅ 正しい<br>app → domain                                     |
| **ボイラープレート**   | -                            | ⚠️ エラー型ごとにtrait impl            | ⚠️ エラー型ごとにif-let + match<br>`_`でfallback可能 | ⚠️ エラー型ごとにPresentableError impl + match<br>`_`でfallback可能 |
| **複数出力形式**     | ⚠️ 関数を複数定義<br>ドメインエラー数×出力形式数 | ⚠️ traitを複数定義<br>ドメインエラー数×出力形式数 | ⚠️ 関数を複数定義<br>ドメインエラー数×出力形式数               | ✅ Visitor実装を追加<br>エラーカテゴリ数×出力形式数                          |
| **コンパイル時チェック** | ✅ 完全<br>網羅性保証                | ⚠️ 部分的<br>各モジュールのmatch内のみ       | ⚠️ 部分的<br>match内のみ                         | ✅ 完全<br>網羅性保証                                             |
| **スケーラビリティ**   | ❌ 低い<br>50種類超で管理困難           | ⚠️ 中程度                          | ✅ 高い                                       | ✅ 高い                                                      |
| **シンプルさ**      | ✅ 最もシンプル                     | ⚠️ trait実装が必要                   | ⚠️ ダウンキャスト必要                               | ⚠️ trait設計必要                                              |

## 決定

### 推奨パターン: Visitor Pattern

このプロジェクトでは**Visitor Pattern（選択肢4）** を標準として採用する。

**選択理由：**

1. **依存関係が正しい**（app → domain）
   - ドメイン層はHTTP/CLIの詳細を知らない
   - アプリケーション層でPresentableError trait実装（extension trait）

2. **ドメイン層の純粋性が高い**
   - ドメインエラーは`std::error::Error`のみ実装
   - プレゼンテーション層の関心事と分離

3. **コンパイル時チェックが完全**
   - ドメインエラーのvariant追加時、PresentableError実装のmatchで非網羅エラー
   - RootErrorに新ドメイン追加時、RootError::present_withのmatchで非網羅エラー
   - コンパイラが変更漏れを防ぐ

4. **エラーカテゴリベースで構造化される**
   - エラーを少数のカテゴリ（NotFound、Conflict、Unauthorized等）に抽象化
   - 共通部品が作りやすい
   - 複数出力形式への対応がスケールする（エラーカテゴリ数×出力形式数）

5. **集中管理を実現**
   - すべて（ErrorPresenter trait、Presenter実装、PresentableError実装）をアプリ層の同一ファイルに定義可能
   - global_error_handlerで横断的関心事を一箇所に集約

**統一エラー型（選択肢1）との比較：**
- 統一エラー型：シンプルだがドメイン純粋性低い、依存関係曖昧、50種超でスケール困難
- Visitor Pattern：アーキテクチャ原則を守り、長期的に保守しやすい

### 例外: プロトタイプ・実験コードのみ統一エラー型を許容

以下の条件を**すべて**満たす場合のみ、統一エラー型を使用可能：

- ✅ プロトタイプまたは実験用コード
- ✅ 本番環境にデプロイしない
- ✅ 数日〜数週間で破棄する想定

**理由：**
- シンプルさを最優先し、高速な実験を可能にする
- 破棄前提なので、依存関係の問題が顕在化しない

本番コードでは**常にVisitor Pattern**を使用する。

### 実装ガイドライン

#### 統一エラー型での実装

<details>
<summary>コード例を表示</summary>

```rust
// src/error.rs
use suzunari_error::*;

#[suzunari_error]
pub enum RootError {
    #[snafu(display("User not found: {id}"))]
    UserNotFound { id: String },

    #[snafu(display("User already exists: {email}"))]
    UserAlreadyExists { email: String },

    #[snafu(display("Authentication failed"))]
    AuthUnauthorized,

    #[snafu(display("Database connection failed"))]
    DbConnectionFailed,
}

impl RootError {
    pub fn to_response(&self) -> Response {
        match self {
            RootError::UserNotFound { id } => Response::not_found(
                format!("User {} not found", id)
            ),
            RootError::UserAlreadyExists { email } => Response::conflict(
                format!("User {} already exists", email)
            ),
            RootError::AuthUnauthorized => Response::unauthorized(
                "Authentication failed".to_string()
            ),
            RootError::DbConnectionFailed => Response::internal_server_error(
                "Database error".to_string()
            ),
        }
    }
}

// main.rs
fn main() -> Result<(), RootError> {
    run()?;
    Ok(())
}
```

</details>

#### Visitor Patternでの実装

##### 基本実装（シンプルなケース）

<details>
<summary>コード例を表示</summary>

```rust
// ============================================
// ドメイン層: user_domain.rs
// ============================================
use suzunari_error::*;

#[suzunari_error]
pub enum UserError {
    #[snafu(display("User not found: {id}"))]
    NotFound { id: String },

    #[snafu(display("User already exists: {email}"))]
    AlreadyExists { email: String },
}

pub fn find_user(id: &str) -> Result<User, UserError> {
    Err(UserError::NotFound { id: id.to_string() })
}

// ============================================
// ドメイン層: auth_domain.rs
// ============================================
use suzunari_error::*;

#[suzunari_error]
pub enum AuthError {
    #[snafu(display("Authentication failed"))]
    Unauthorized,

    #[snafu(display("Token expired"))]
    TokenExpired,
}

// ============================================
// アプリ層: error.rs
// ============================================
use suzunari_error::*;
use snafu::ResultExt;
use user_domain::UserError;
use auth_domain::AuthError;

// ドメインエラーをまとめるenum
#[suzunari_error]
pub enum RootError {
    #[snafu(display("User error: {source}"))]
    User { source: UserError },

    #[snafu(display("Auth error: {source}"))]
    Auth { source: AuthError },
}

// Presenter trait
pub trait ErrorPresenter {
    type Output;

    fn present_not_found(&self, message: String) -> Self::Output;
    fn present_conflict(&self, message: String) -> Self::Output;
    fn present_unauthorized(&self, message: String) -> Self::Output;
}

pub trait PresentableError {
    fn present_with<P: ErrorPresenter>(&self, presenter: &P) -> P::Output;
}

// Extension trait実装
impl PresentableError for UserError {
    fn present_with<P: ErrorPresenter>(&self, presenter: &P) -> P::Output {
        match self {
            UserError::NotFound { id } =>
                presenter.present_not_found(format!("User {} not found", id)),
            UserError::AlreadyExists { email } =>
                presenter.present_conflict(format!("User {} already exists", email)),
        }
    }
}

impl PresentableError for AuthError {
    fn present_with<P: ErrorPresenter>(&self, presenter: &P) -> P::Output {
        match self {
            AuthError::Unauthorized =>
                presenter.present_unauthorized("Authentication required".to_string()),
            AuthError::TokenExpired =>
                presenter.present_unauthorized("Token has expired".to_string()),
        }
    }
}

impl PresentableError for RootError {
    fn present_with<P: ErrorPresenter>(&self, presenter: &P) -> P::Output {
        match self {
            RootError::User { source } => source.present_with(presenter),
            RootError::Auth { source } => source.present_with(presenter),
        }
    }
}

// ============================================
// アプリ層: http_handler.rs
// ============================================
pub struct HttpErrorPresenter;

impl ErrorPresenter for HttpErrorPresenter {
    type Output = HttpResponse;

    fn present_not_found(&self, message: String) -> HttpResponse {
        HttpResponse {
            status: 404,
            body: format!(r#"{{"error":"NOT_FOUND","message":"{}"}}"#, message),
        }
    }

    fn present_conflict(&self, message: String) -> HttpResponse {
        HttpResponse {
            status: 409,
            body: format!(r#"{{"error":"CONFLICT","message":"{}"}}"#, message),
        }
    }

    fn present_unauthorized(&self, message: String) -> HttpResponse {
        HttpResponse {
            status: 401,
            body: format!(r#"{{"error":"UNAUTHORIZED","message":"{}"}}"#, message),
        }
    }
}

pub fn global_error_handler(error: RootError) -> HttpResponse {
    eprintln!("Error occurred: {:?}", error);
    error.present_with(&HttpErrorPresenter)
}

// ============================================
// エントリポイント: main.rs
// ============================================
use snafu::ResultExt;

fn handle_request(user_id: &str) -> Result<String, RootError> {
    let user = user_domain::find_user(user_id).context(error::UserSnafu)?;
    auth_domain::authenticate("token").context(error::AuthSnafu)?;
    Ok(format!("Success: {:?}", user))
}

fn main() {
    match handle_request("user123") {
        Ok(result) => println!("{}", result),
        Err(e) => {
            let response = global_error_handler(e);
            println!("HTTP {} - {}", response.status, response.body);
        }
    }
}
```

</details>

##### Application/Platformパターン（インフラエラーを含むケース）

インフラストラクチャ障害（DB接続、ネットワークタイムアウト等）を扱う場合、各ドメインのエラーを**Application**（ビジネスルール違反）と**Platform**（外部システム障害）に分類する。

<details>
<summary>コード例を表示</summary>

```rust
// ============================================
// Shared Kernel: shared_kernel.rs
// ============================================
// インフラストラクチャエラー（ドメイン横断）
use suzunari_error::*;

#[suzunari_error]
pub enum InfraError {
    #[snafu(display("Database connection failed"))]
    #[snafu(visibility(pub))]
    DatabaseConnection { source: std::io::Error },

    #[snafu(display("Network timeout"))]
    #[snafu(visibility(pub))]
    NetworkTimeout { source: std::io::Error },

    #[snafu(display("Cache error"))]
    #[snafu(visibility(pub))]
    CacheError { source: std::io::Error },
}

// ============================================
// ドメイン層: user_domain.rs
// ============================================
use suzunari_error::*;
use snafu::ResultExt;
use crate::shared_kernel::InfraError;

// ドメインエラー（ビジネスルール違反）
#[suzunari_error]
pub enum UserError {
    #[snafu(display("User not found: {id}"))]
    #[snafu(visibility(pub))]
    NotFound { id: String },

    #[snafu(display("User already exists: {email}"))]
    #[snafu(visibility(pub))]
    AlreadyExists { email: String },
}

// ドメイン全体のエラー（Application + Platform）
#[suzunari_error]
pub enum UserDomainError {
    /// アプリケーションエラー（ビジネスルール違反）
    #[snafu(display("Application error: {source}"))]
    #[snafu(visibility(pub))]
    Application { source: UserError },

    /// プラットフォームエラー（外部システム障害）
    /// transparent: 追加コンテキスト不要、そのまま伝播
    #[snafu(transparent)]
    Platform { source: InfraError },
}

pub fn find_user(id: &str) -> Result<User, UserDomainError> {
    // ユーザーが見つからない（Application error）
    NotFoundSnafu { id: id.to_string() }
        .fail()
        .context(ApplicationSnafu)
}

// ============================================
// ドメイン層: auth_domain.rs
// ============================================
use suzunari_error::*;
use snafu::ResultExt;
use crate::shared_kernel::InfraError;

#[suzunari_error]
pub enum AuthError {
    #[snafu(display("Authentication failed"))]
    #[snafu(visibility(pub))]
    Unauthorized,

    #[snafu(display("Token expired"))]
    #[snafu(visibility(pub))]
    TokenExpired,
}

#[suzunari_error]
pub enum AuthDomainError {
    /// アプリケーションエラー（ビジネスルール違反）
    #[snafu(display("Application error: {source}"))]
    #[snafu(visibility(pub))]
    Application { source: AuthError },

    /// プラットフォームエラー（外部システム障害）
    /// transparent: 追加コンテキスト不要、そのまま伝播
    #[snafu(transparent)]
    Platform { source: InfraError },
}

pub fn authenticate(token: &str) -> Result<(), AuthDomainError> {
    // 外部認証サービスアクセス（Platform error）
    // transparent により InfraError が自動的に AuthDomainError::Platform に変換される
    let _result = std::fs::read_to_string("/nonexistent")
        .context(crate::shared_kernel::NetworkTimeoutSnafu)?;

    UnauthorizedSnafu.fail().context(ApplicationSnafu)
}

// ============================================
// アプリ層: app_error.rs
// ============================================
use suzunari_error::*;
use snafu::ResultExt;
use user_domain::{UserError, UserDomainError};
use auth_domain::{AuthError, AuthDomainError};
use shared_kernel::InfraError;

#[suzunari_error]
pub enum RootError {
    #[snafu(display("User domain error: {source}"))]
    #[snafu(visibility(pub))]
    UserDomain { source: UserDomainError },

    #[snafu(display("Auth domain error: {source}"))]
    #[snafu(visibility(pub))]
    AuthDomain { source: AuthDomainError },
}

// Presenter trait
pub trait ErrorPresenter {
    type Output;

    fn present_not_found(&self, message: String) -> Self::Output;
    fn present_conflict(&self, message: String) -> Self::Output;
    fn present_unauthorized(&self, message: String) -> Self::Output;
    fn present_internal_error(&self, message: String) -> Self::Output;
}

pub trait PresentableError {
    fn present_with<P: ErrorPresenter>(&self, presenter: &P) -> P::Output;
}

// Extension trait実装
impl PresentableError for UserError {
    fn present_with<P: ErrorPresenter>(&self, presenter: &P) -> P::Output {
        match self {
            UserError::NotFound { id, .. } =>
                presenter.present_not_found(format!("User {} not found", id)),
            UserError::AlreadyExists { email, .. } =>
                presenter.present_conflict(format!("User {} already exists", email)),
        }
    }
}

impl PresentableError for AuthError {
    fn present_with<P: ErrorPresenter>(&self, presenter: &P) -> P::Output {
        match self {
            AuthError::Unauthorized { .. } =>
                presenter.present_unauthorized("Authentication required".to_string()),
            AuthError::TokenExpired { .. } =>
                presenter.present_unauthorized("Token has expired".to_string()),
        }
    }
}

impl PresentableError for InfraError {
    fn present_with<P: ErrorPresenter>(&self, presenter: &P) -> P::Output {
        // すべてのプラットフォームエラーは internal_error として扱う
        presenter.present_internal_error(self.to_string())
    }
}

impl PresentableError for UserDomainError {
    fn present_with<P: ErrorPresenter>(&self, presenter: &P) -> P::Output {
        match self {
            UserDomainError::Application { source, .. } => source.present_with(presenter),
            UserDomainError::Platform { source, .. } => source.present_with(presenter),
        }
    }
}

impl PresentableError for AuthDomainError {
    fn present_with<P: ErrorPresenter>(&self, presenter: &P) -> P::Output {
        match self {
            AuthDomainError::Application { source, .. } => source.present_with(presenter),
            AuthDomainError::Platform { source, .. } => source.present_with(presenter),
        }
    }
}

impl PresentableError for RootError {
    fn present_with<P: ErrorPresenter>(&self, presenter: &P) -> P::Output {
        match self {
            RootError::UserDomain { source, .. } => source.present_with(presenter),
            RootError::AuthDomain { source, .. } => source.present_with(presenter),
        }
    }
}

// ============================================
// アプリ層: http_handler.rs
// ============================================
pub struct HttpErrorPresenter;

impl ErrorPresenter for HttpErrorPresenter {
    type Output = HttpResponse;

    fn present_not_found(&self, message: String) -> HttpResponse {
        HttpResponse {
            status: 404,
            body: format!(r#"{{"error":"NOT_FOUND","message":"{}"}}"#, message),
        }
    }

    fn present_conflict(&self, message: String) -> HttpResponse {
        HttpResponse {
            status: 409,
            body: format!(r#"{{"error":"CONFLICT","message":"{}"}}"#, message),
        }
    }

    fn present_unauthorized(&self, message: String) -> HttpResponse {
        HttpResponse {
            status: 401,
            body: format!(r#"{{"error":"UNAUTHORIZED","message":"{}"}}"#, message),
        }
    }

    fn present_internal_error(&self, message: String) -> HttpResponse {
        HttpResponse {
            status: 500,
            body: format!(r#"{{"error":"INTERNAL_SERVER_ERROR","message":"{}"}}"#, message),
        }
    }
}

pub fn global_error_handler(error: RootError) -> HttpResponse {
    eprintln!("Error occurred: {:?}", error);
    error.present_with(&HttpErrorPresenter)
}
```

**ポイント:**

1. **Shared Kernel**: `InfraError`をドメイン横断の共通エラーとして定義
2. **Application/Platform分類**（[coding-practices.md](../.ai/coding-practices.md#error-classification)に準拠）:
   - `Application`: 自分たちのコードのエラー（domain, use case, adapter, framework層）
   - `Platform`: 外部システムのエラー（database, network, external service等）
3. **transparent属性**: `Platform`バリアントに使用し、追加コンテキストなしで自動変換
4. **命名の意図**:
   - レイヤ名（Domain層、Infrastructure層）と区別するため
   - エラーの所有者（自分たち vs 外部）を明確化

</details>

## 結果

### 良い影響

- ✅ **原則との整合性**: coding-practicesの「集中管理」「依存方向」「ドメイン純粋性」を完全に実現
- ✅ **コンパイル時の安全性**: エラー追加時の変更漏れをコンパイラが検知
- ✅ **長期保守性**: アーキテクチャ原則に沿った設計で、スケールしても破綻しない
- ✅ **エラーカテゴリの再利用**: 共通部品が作りやすく、複数出力形式への対応が容易

### トレードオフ

- ⚠️ **初期学習コスト**: Visitor Patternは統一エラー型よりも複雑
  - 軽減策: 実装サンプルを提供、[rust-examples/error-handling.md](../.ai/rust-examples/error-handling.md)で解説
- ⚠️ **初期実装コスト**: プロトタイプでも正しいアーキテクチャを要求される
  - 軽減策: プロトタイプ・実験コードでは統一エラー型を許容

## 参考文献

- [Error Handling in Rust](https://doc.rust-lang.org/book/ch09-00-error-handling.html)
- [thiserror](https://github.com/dtolnay/thiserror)
- [anyhow](https://github.com/dtolnay/anyhow)
- [suzunari-error](https://github.com/ryotan/suzunari-error)
- [Visitor Pattern](https://refactoring.guru/design-patterns/visitor)
- Coding Practices: [Error Handling](../.ai/coding-practices.md#error-handling)
