# Design Practices

## Core Principles

### Ubiquitous Language

- Use domain terms consistently across requirements, design, and implementation
- Avoid technical jargon when describing business concepts
- Bridge the gap between domain experts and developers through shared vocabulary
- **Why**: Reduces misunderstanding and keeps code aligned with business intent

### Bounded Contexts

- Define explicit boundaries where a model applies
- Each context has its own ubiquitous language
- Models can differ across contexts (e.g., "User" in Auth Context vs "Customer" in Sales Context)
- **Why**: Prevents model pollution and maintains clear ownership

### Explicit Boundaries

- Mark integration points between contexts clearly
- Use Anti-Corruption Layers when integrating external systems
- Don't leak implementation details across boundaries
- **Why**: Protects domain integrity and enables independent evolution

## Dependency Direction

### The Dependency Rule

- Dependencies point inward: Outer layers depend on inner layers, never the reverse
- Domain layer has no dependencies on infrastructure or application
- Application layer orchestrates but doesn't contain business logic
- **Why**: Keeps business logic pure and testable

### Dependency Inversion

- Depend on abstractions, not concretions
- Define interfaces in the layer that uses them
- Implement interfaces in outer layers
- **Why**: Enables flexibility and testability without coupling

## Cohesion and Coupling

### High Cohesion

- Group related concepts together
- A module should have a single, well-defined purpose
- Changes to related functionality happen in one place
- **Why**: Easier to understand, modify, and maintain

### Low Coupling

- Minimize dependencies between modules
- Communicate through well-defined interfaces
- Avoid shared mutable state
- **Why**: Changes in one module don't ripple across the system

## Invariants

### Protect Invariants

- Business rules that must always be true
- Enforce at creation and modification time
- Don't allow invalid states to exist
- **Why**: Eliminates whole classes of bugs

[//]: # (
TODO: 以下のような内容が誤解なく伝わるように、適切なファイルに記載する（このファイルに書くべきかどうかの判断から必要）
  - 「作成」とは実装における「インスタンスの生成」とは異なります。
  - 例えば、「過去には制限がなかったが、新しく追加された制限」を考えると自明です。
    - 過去に生成されたエンティティは、「新しく追加された制限」を満たさなくてもインスタンス化できないと業務を継続できません。
    - 一方で、新しくエンティティを作成するときには、「新しく追加された制限」が絶対に満たされている必要があります。
)

### Fail Fast

- Validate immediately, don't defer
- Reject invalid data at system boundaries
- Make invalid states unrepresentable
- **Why**: Errors are caught early and close to their source

## Value Objects vs Entities

### Value Objects

- Defined by their attributes, not identity
- Immutable
- Equality based on values
- Examples: Email, Money, DateRange
- **Why**: Reduce bugs through immutability, express domain concepts clearly

### Entities

- Have unique identity
- Mutable state over time
- Equality based on ID
- Own their invariants
- **Why**: Model real-world objects with lifecycles

## Aggregates

### Aggregate Boundaries

- Cluster of entities and value objects treated as a unit
- One entity is the aggregate root
- External references only to the root
- **Why**: Enforces consistency and simplifies transaction boundaries

### Consistency Boundaries

- Invariants apply within an aggregate
- Changes to multiple aggregates use eventual consistency
- Transactions don't span aggregates
- **Why**: Enables scalability and clear ownership

## Events

### Domain Events

- Use past tense: "OrderPlaced", "PaymentCompleted"
- Immutable facts about what happened
- Include aggregate ID, timestamp, and relevant data
- **Why**: Enables loose coupling and audit trails

### Event-Driven Design

- Aggregates publish events after state changes
- Other contexts can subscribe without tight coupling
- Events enable eventual consistency across boundaries
- **Why**: Decouples contexts while maintaining data consistency

## Repositories

### Repository Abstraction

- Collection-like interface for aggregates
- Hide persistence details from domain
- Methods reflect domain operations, not database operations
- **Why**: Domain logic stays pure and portable

### Aggregate Root Access

- Repositories only for aggregate roots
- Load entire aggregate, not individual entities
- **Why**: Maintains aggregate consistency guarantees

[//]: # (TODO: Add "Adapters" stereotype from Clean Architecture, Use "Use Cases" or "Command+Query" instead of Services)
## Services

### Domain Services

- Operations that don't naturally belong to an entity or value object
- Stateless
- Express domain concepts
- **Why**: Keeps entities focused, expresses domain operations clearly

### Application Services

- Orchestrate domain objects
- Handle transactions
- Don't contain business logic
- **Why**: Separates coordination from domain logic

## Anti-Patterns to Avoid

### Anemic Domain Model

- Entities with only getters/setters
- Logic in services instead of domain objects
- **Why it's bad**: Loses benefits of object-oriented design

### Smart UI

- Business logic in presentation layer
- Direct database access from UI
- **Why it's bad**: Impossible to test, hard to maintain

### Leaky Abstractions

- Infrastructure concerns in domain
- Domain knowing about databases, HTTP, etc.
- **Why it's bad**: Tight coupling, hard to evolve

## Design Evolution

### Refactoring Toward Deeper Insight

- Design emerges from understanding the domain
- Continuous refinement as knowledge grows
- Don't expect perfect design upfront
- **Why**: Deep models come from iteration and learning

### Model Integrity

- Keep the model consistent within a context
- Don't compromise model for technical convenience
- Refactor when the model no longer fits
- **Why**: Model quality determines code quality
