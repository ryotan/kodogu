# ADR-0002: データ変換に Translator を使用する

## ステータス

承認済み

## 背景

Clean Architecture および Hexagonal Architecture では、レイヤー境界でのデータ変換が必要とされる。この変換を実現するパターンとして、以下の2つが存在する：

1. **Adapter Interface**（Hexagonal Architecture では「Port」と呼ばれる）: 振舞いを定義する抽象契約
2. **Translator（具象クラス）**: データ変換ロジック

疑問: データ変換は interface を通じて抽象化すべきか、それとも具象クラスとして実装すべきか？

### 業界の現状

- **Clean Architecture** (Robert C. Martin): DTO で境界を跨ぐことに言及するが、誰が変換するかは明示されていない
- **Hexagonal Architecture** (Alistair Cockburn): Port/Adapter パターンは振舞いに焦点を当て、変換は Adapter 内で暗黙的
- **DDD** (Eric Evans): Anti-Corruption Layer の文脈で「Translator」を明示的に言及するが、外部システム境界のみ

### 問題点

データ変換に interface (Port) を使用すると：
- 過剰抽象化: 単一実装に対する interface（YAGNI 違反）
- 責務の混乱: 振舞いの抽象化とデータ変換の混在
- 不要な間接参照: 純粋関数に対する DI

### 用語の整理: Port について

**Port = Adapter が実装する Interface**（Hexagonal Architecture の用語）

Hexagonal Architecture では：
- Primary Port (Driving): Use Case など、アプリケーションを駆動するインターフェース
- Secondary Port (Driven): Repository, External Service など、アプリケーションが利用するインターフェース

実装上は単なる interface だが、「アプリケーションの入出口」という概念的意味を持つ。

**このプロジェクトでの方針:**
- Port という抽象用語を**避ける**
- 具体名を使用: Repository interface, ExternalService interface など
- **理由**: 実装 = interface、概念的価値 < 教育コスト

## 決定

**レイヤー境界でのデータ変換には Translator（具象クラス）を使用する。外部システム境界での振舞い抽象化には interface（Port とも呼ばれる）を使用する。**

### Translator の特性

- **形式**: 具象クラス（interface なし）
- **目的**: レイヤー間でのデータ構造変換
- **配置**: 外側のレイヤー（Dependency Rule に従う）
- **DI**: 依存がある場合は DI 管理される場合もある（フレームワーク依存）
- **複数実装**: 技術スタックごとに異なる Translator（例: JpaOrderTranslator, MongoOrderTranslator）

### Adapter Interface（Port）の特性

- **形式**: Interface
- **目的**: 外部システム境界での振舞い抽象化
- **置換可能性**: 共通契約の背後に複数実装
- **例**: Repository, ExternalServiceClient, MessageQueue

### 本質的区別

**Adapter Interface は振舞いを抽象化（何をするか）、Translator はデータを変換（どう変換するか）**

## 例

### Use Cases 層

```kotlin
// Translator: Command/DTO → Domain Model
object OrderCommandTranslator {
    fun toDomain(cmd: CreateOrderCommand): Order {
        return Order(
            OrderId.generate(),
            UserId(cmd.userId),
            cmd.items.map { OrderLine(...) }
        )
    }

    fun toResponse(order: Order): OrderResponse {
        return OrderResponse(
            orderId = order.id.value,
            status = order.status.name
        )
    }
}

// Use Case は Translator を直接使用（interface なし）
class CreateOrderUseCase(
    private val repository: OrderRepository  // Adapter Interface
) {
    fun execute(cmd: CreateOrderCommand): OrderResponse {
        val order = OrderCommandTranslator.toDomain(cmd)
        repository.save(order)
        return OrderCommandTranslator.toResponse(order)
    }
}
```

### Adapters 層

```kotlin
// Translator: Domain Model → Infrastructure Model
object JpaOrderTranslator {
    fun toEntity(order: Order): OrderEntity {
        return OrderEntity(
            id = order.id.value,
            userId = order.userId.value,
            // ...
        )
    }

    fun toDomain(entity: OrderEntity): Order {
        return Order(
            OrderId(entity.id),
            UserId(entity.userId),
            // ...
        )
    }
}

// Adapter は内部で Translator を使用
class JpaOrderRepository(
    private val jpaRepository: JpaRepository<OrderEntity, UUID>
) : OrderRepository {  // Adapter Interface を実装
    override fun save(order: Order) {
        val entity = JpaOrderTranslator.toEntity(order)
        jpaRepository.save(entity)
    }
}
```

### 依存がある場合（Spring Boot）

```kotlin
@Component
class JpaOrderTranslator(
    private val objectMapper: ObjectMapper  // DI コンポーネント
) {
    fun toEntity(order: Order): OrderEntity {
        // objectMapper を使った複雑な変換
    }
}

@Repository
class JpaOrderRepository(
    private val jpaRepository: JpaRepository<OrderEntity, UUID>,
    private val translator: JpaOrderTranslator  // DI 管理
) : OrderRepository
```

## 根拠

### 1. 関心の分離（SRP）

**Translator なし:**
```kotlin
class JpaOrderRepository : OrderRepository {
    override fun save(order: Order) {
        // 振舞い: DB 接続、トランザクション、エラー処理
        // データ: Order → OrderEntity 変換
        // ❌ 2つの責務が1つのクラスに
    }
}
```

**Translator あり:**
```kotlin
class JpaOrderRepository : OrderRepository {
    override fun save(order: Order) {
        val entity = JpaOrderTranslator.toEntity(order)  // データ変換
        jpaRepository.save(entity)  // 振舞い
        // ✅ 単一責務: 永続化の調整
    }
}
```

### 2. テスタビリティ

Translator は純粋関数（またはステートレスコンポーネント）であり、単独で容易にテスト可能：

```kotlin
@Test
fun `Order を OrderEntity に正しく変換できること`() {
    val order = Order(...)
    val entity = JpaOrderTranslator.toEntity(order)

    assertEquals(order.id.value, entity.id)
    // モック不要、DI セットアップ不要
}
```

### 3. 再利用性

Translator は複数コンポーネントで再利用可能：

```kotlin
// Repository で使用
class JpaOrderRepository {
    fun save(order: Order) = jpaRepository.save(JpaOrderTranslator.toEntity(order))
}

// テストフィクスチャで使用
object OrderFixtures {
    fun createOrderEntity(order: Order) = JpaOrderTranslator.toEntity(order)
}
```

### 4. YAGNI 準拠

単一実装に対する過剰抽象化を回避：

```kotlin
// ❌ 過剰抽象化
interface OrderTranslator {
    fun toEntity(order: Order): OrderEntity
}
class OrderTranslatorImpl : OrderTranslator { ... }

// ✅ シンプルで十分
object JpaOrderTranslator {
    fun toEntity(order: Order): OrderEntity { ... }
}
```

### 5. 原則との整合性

- **Clean Architecture**: Dependency Rule 遵守（外側のレイヤーが内側に依存）
- **DDD**: Anti-Corruption Layer 概念のすべての境界への拡張
- **SOLID**:
  - SRP: 変換を振舞いから分離
  - DIP: 違反なし（Translator と Domain Model は同じ抽象レベル）
  - ISP: 単一実装に対する不要な interface を回避

## 決定ガイドライン

### Translator を導入すべき場合

✅ **使用する:**
- 構造的変換が必要（Command → Domain Model, Domain Model → Entity）
- 複数フィールドのマッピングが必要
- 変換ロジックが非自明

❌ **使用しない:**
- 単純な値のラッピング・アンラッピング（例: `OrderId(string)`）
- 変換なしの直接的なパススルー

### Adapter Interface を使用すべき場合

✅ **使用する:**
- 外部システム境界（Repository, External Service）
- 複数実装が想定される（JPA, MongoDB, InMemory）
- テスタビリティのためモック・スタブが必要

❌ **使用しない:**
- 単一実装の内部境界
- 純粋なデータ変換

## 結果

### 良い影響

- **明確性**: データ変換の責務が明示的
- **テスタビリティ**: 純粋関数は容易にテスト可能
- **保守性**: 変換ロジックの変更が局所化
- **シンプルさ**: 不要な interface や DI の複雑さがない
- **SRP 準拠**: Adapter は振舞いに集中、Translator はデータに集中

### 悪い影響

- **追加ファイル**: コードベースにより多くのクラス・オブジェクト
- **間接参照**: データフローに1段階の迂回
- **学習曲線**: チームは Adapter Interface vs Translator の区別を理解する必要

### 軽減策

- デザインプラクティスと用語集でパターンを文書化
- 各言語（Kotlin, Rust, TypeScript）で例を提供
- 一貫した命名: `*Translator` サフィックス

## 参考文献

- Clean Architecture (Robert C. Martin, Chapter 22: The Clean Architecture)
- Domain-Driven Design (Eric Evans, Chapter 14: Maintaining Model Integrity - Anti-Corruption Layer)
- Implementing Domain-Driven Design (Vaughn Vernon, Chapter 4: Architecture - Application Services)
- The Hexagonal Architecture (Alistair Cockburn) - Port の概念的定義
