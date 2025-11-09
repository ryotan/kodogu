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

## Architectural Layers

### Layer Structure

This project follows Clean Architecture with DDD tactical patterns:

**Domain**
- [Entities](#entities), [Value Objects](#value-objects), [Aggregates](#aggregates)
- [Domain Behaviors](#domain-behaviors)
- [Domain Events](#domain-events)
- [Repository](#repositories) interfaces (abstractions)

**Use Cases**
- [Use Cases](#use-cases) (orchestration)
- Commands and Queries
- Application-specific workflows

**Adapters**
- Controllers, Presenters
- Repository implementations
- External service clients
- Message queue adapters

**Frameworks**
- Web frameworks
- Database drivers
- External libraries
- Infrastructure tools

### Mapping to Other Architectures

| This Project | Clean Architecture   | DDD Layered       | Onion Architecture   | Hexagonal Architecture |
|--------------|----------------------|-------------------|----------------------|------------------------|
| Domain       | Entities             | Domain Layer      | Domain Model         | Core/Hexagon           |
| Use Cases    | Use Cases            | Application Layer | Application Services | Core/Hexagon           |
| Adapters     | Interface Adapters   | Infrastructure    | Infrastructure       | Adapters               |
| Frameworks   | Frameworks & Drivers | Infrastructure    | Infrastructure       | Frameworks             |

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

- Business rules that must be consistent at a given point in time
- Enforce at creation and modification time
- Don't allow invalid states to exist
- **Why**: Eliminates whole classes of bugs

(Note: Invariants evolve over time—see "Temporal Validity" below)

### Temporal Validity: Invariants Change Over Time

- Data created in 2023: valid under 2023 rules
- Data created in 2024: valid under 2024 rules
- System must track: "which invariants applied when"
- **Why**: Business rules are not eternal; systems must support evolution

### Entity Lifecycles: Creation vs Instantiation

**Two different lifecycles must be distinguished:**

- **Creation (Business lifecycle)**: Bringing a business entity into existence for the first time
  - Example: User places an order → NEW Order entity is born
  - Represents a business event: "Order was created"

- **Instantiation (In-memory lifecycle)**: Technical act of creating an object in memory
  - Example: Loading an existing Order from database, reconstructing it in memory
  - Also happens during Creation (Creation requires instantiation)
  - Represents technical operation: "Object constructed in RAM"

- **Why**: Understanding this distinction prevents incorrect invariant enforcement during data loading. All Creations involve Instantiation, but not all Instantiations are Creations.

### When & How to Apply Invariants

Apply invariants according to entity lifecycle:

- **Creation, Modification**: ✅ Enforce ALL current invariants
- **Instantiation**: ❌ Bypass invariants (temporal validity)

**Fail Fast principle:**
- Validate immediately, don't defer
- Reject invalid data at system boundaries
- Make invalid states unrepresentable

**Why**:
- Lifecycle rules enable system evolution
- Fail Fast catches errors early and close to source

### Evolution Strategy

When adding new invariants to existing systems:

- **Document temporal validity**: Mark when invariants were introduced
- **Gradual enforcement**: Use feature flags or version markers for progressive rollout
- **Provide migration paths**:
  - Migrate on first modification (lazy migration)
  - Batch migration jobs for bulk updates
  - Accept old format with deprecation warnings
- **Set deprecation timeline**: Communicate when old data formats will no longer be supported
- **Why**: Balances system evolution with operational continuity

## Value Objects

- Defined by their attributes, not identity
- Immutable
- Equality based on values
- Examples: Email, Money, DateRange
- **Why**: Reduce bugs through immutability, express domain concepts clearly

## Entities

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

## Domain Behaviors

### Operations and Rules Without a Natural Home

- Operations and rules that don't naturally belong to an entity or value object
- Stateless operations and rules on domain concepts
- Contain business logic (not just orchestration)
- Can coordinate multiple aggregates when applying business rules
- Examples: CalculateShipping, ValidateCrossAggregateInvariant
- **Why**: Keeps entities focused, expresses complex domain logic clearly

### What Domain Behaviors Don't Do

- Don't handle transactions or infrastructure (that's Use Cases)
- Don't orchestrate without business logic (that's also Use Cases)
- **Why**: Maintains clean separation between domain logic and application layer

> [!Note]
> In DDD literature, this is called **"Domain Services"**. This project uses
> "Domain Behaviors" to emphasize the distinction from Application Services
> (Use Cases), which handle orchestration rather than domain logic.
> **Domain Services** and **Domain Behaviors** refer to the same concept.

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

## Use Cases

### Application Orchestration

- Coordinate domain objects and services
- Implement application-specific workflows
- Handle transactions and cross-cutting concerns
- Examples: CreateOrderUseCase, ProcessPaymentCommand, GetOrderDetailsQuery
- **Why**: Separates coordination from domain logic

### Commands and Queries

- **Commands**: Operations that change state (CreateOrder, UpdatePayment)
- **Queries**: Operations that read state (GetOrderById, ListOrders)
- Commands may return success/failure, queries return data
- **Why**: Clear distinction between reads and writes

### Use Case Responsibilities

- Validate input (application-level validation)
- Load aggregates via repositories
- Execute domain logic via aggregates and domain behaviors
- Save changes via repositories
- Publish domain events (if needed)
- **Why**: Defines clear application workflow

### What Use Cases Don't Do

- Don't contain business logic (that's in domain)
- Don't access infrastructure directly (use adapters)
- Don't know about HTTP, databases, or frameworks
- **Why**: Keeps application layer focused and testable

## Adapters

### Port and Adapter Pattern

- Domain defines interfaces (ports)
- Infrastructure provides implementations (adapters)
- Domain depends on abstractions, not implementations
- **Why**: Protects domain from infrastructure changes

### Adapter Responsibilities

- Convert between external formats and domain models
- Handle technical concerns (serialization, protocols, etc.)
- Implement domain-defined interfaces
- **Why**: Clean separation of concerns

### Anti-Corruption Layer

- Protect domain from external system changes
- Translate external models to domain models
- Don't leak external concepts into domain
- **Why**: Domain integrity and independence

### Common Adapter Types

- **Framework Adapters**: Controllers, Presenters
- **Persistence Adapters**: Database, file system, cache
- **Message Adapters**: Queues, event buses, pub/sub
- **External Service Adapters**: REST, GraphQL, gRPC clients of third-party APIs or legacy systems
- **Why**: Each handles different integration concerns

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
