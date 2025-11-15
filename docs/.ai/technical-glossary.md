# Glossary

Quick reference for DDD and architectural terms used in this project. For detailed explanations, follow the links to practice documents.

## DDD Tactical Patterns

### Aggregate

A cluster of entities and value objects treated as a unit for data changes. Has one entity as the aggregate root, and external references only go to the root.

→ See [design-practices.md](./design-practices.md#aggregates)

### Aggregate Root

The single entity within an aggregate that external objects reference. Controls access to all other entities and value objects in the aggregate.

→ See [design-practices.md](./design-practices.md#aggregate-boundaries)

### Domain Behavior (Domain Service)

Stateless operations containing business logic that don't naturally belong to an entity or value object. Can coordinate multiple aggregates when applying business rules.

→ See [design-practices.md](./design-practices.md#domain-behaviors)

### Domain Event

An immutable fact about something significant that happened in the domain. Named in past tense (e.g., "OrderPlaced"). Enables loose coupling and audit trails.

→ See [design-practices.md](./design-practices.md#domain-events)

### Entity

An object defined by unique identity rather than attributes. Has mutable state over time, and equality is based on ID.

→ See [design-practices.md](./design-practices.md#entities)

### Repository

A collection-like interface for loading and saving aggregates. Hides persistence details from the domain, with methods reflecting domain operations rather than database operations.

→ See [design-practices.md](./design-practices.md#repositories)

### Value Object

An object defined by its attributes rather than identity. Immutable, with equality based on values (e.g., Email, Money, DateRange).

→ See [design-practices.md](./design-practices.md#value-objects)

## DDD Strategic Patterns

### Anti-Corruption Layer (ACL)

A translation layer that protects the domain model from external system changes. Translates external models to domain models without leaking external concepts.

→ See [design-practices.md](./design-practices.md#anti-corruption-layer)

### Bounded Context

An explicit boundary within which a domain model applies. Each context has its own ubiquitous language, and models can differ across contexts.

→ See [design-practices.md](./design-practices.md#bounded-contexts)

### Conformist

A context mapping pattern where the downstream context conforms to the upstream model without negotiation. Used when upstream is external or non-negotiable.

### Customer-Supplier

A context mapping pattern with upstream/downstream relationship where downstream needs can influence upstream priorities through negotiation.

### Open Host Service

A context mapping pattern where the upstream context publishes a well-defined API for multiple downstream contexts to integrate with.

### Partnership

A context mapping pattern where both teams coordinate and collaborate closely, sharing goals and aligning schedules.

### Published Language

A well-defined shared language (e.g., JSON schema) used for inter-context communication.

### Shared Kernel

A small, carefully managed portion of code shared between bounded contexts. Requires strong coordination between teams.

### Ubiquitous Language

A shared vocabulary used consistently across requirements, design, and implementation within a bounded context. Bridges the gap between domain experts and developers.

→ See [design-practices.md](./design-practices.md#ubiquitous-language)

## Clean Architecture Concepts

### Adapter

An implementation of a port interface. Converts between external formats and domain models, handling technical concerns like serialization and protocols.

→ See [design-practices.md](./design-practices.md#adapters)

### Dependency Rule

The fundamental rule of Clean Architecture: dependencies point inward only. Outer layers depend on inner layers, never the reverse.

→ See [design-practices.md](./design-practices.md#dependency-direction)

### Port

A term from Hexagonal Architecture referring to interfaces that adapters implement. **Note: This project does not use this term in practice**, but it's explained here because you may encounter it in other literature. We call these "Adapter Interface" or use specific names like Repository, ExternalService, MessageQueue.

→ See [design-practices.md](./design-practices.md#adapter-interface-pattern) · [ADR-0002](../adr/0002-prefer-translator-for-data-transformation.md)

### Translator

A concrete class responsible for data transformation at layer boundaries. Converts between different data structures (Command → Domain Model, Domain Model → Entity). Does not use interfaces, technology-specific, placed in outer layers following Dependency Rule.

→ See [design-practices.md](./design-practices.md#translators) · [ADR-0002](../adr/0002-prefer-translator-for-data-transformation.md)

### Screaming Architecture

A principle stating that project structure should reveal the domain rather than frameworks. Top-level directories should be named after business concepts.

→ See [design-practices.md](./design-practices.md#screaming-architecture)

### Use Case

An application-layer component that orchestrates domain objects and services to implement application-specific workflows. Handles transactions and cross-cutting concerns but doesn't contain business logic.

→ See [design-practices.md](./design-practices.md#use-cases)

## Coupling Analysis (Khononov)

### Distance

A coupling dimension measuring how far apart components are. **Near**: same aggregate/context/module. **Far**: different contexts/services/applications.

→ See [design-practices.md](./design-practices.md#coupling-analysis-framework-khononov)

### Intrusive Coupling

A problematic type of integration that depends on concrete implementation classes, uses library-specific APIs, or requires knowledge of internal details. Cannot be replaced without changing dependent code.

→ See [design-practices.md](./design-practices.md#coupling-analysis-framework-khononov)

### Non-intrusive Coupling

An acceptable type of integration that depends on abstractions rather than implementations. Can be replaced without changing dependent code.

→ See [design-practices.md](./design-practices.md#coupling-analysis-framework-khononov)

### Volatility

A coupling dimension measuring how frequently a component changes. **High**: UI components, external API formats, evolving business rules. **Low**: core domain concepts, mathematical operations, established standards.

→ See [design-practices.md](./design-practices.md#coupling-analysis-framework-khononov)

## SOLID Principles

### Dependency Inversion Principle (DIP)

The principle most central to Clean Architecture: high-level modules should not depend on low-level modules; both should depend on abstractions. Abstractions should not depend on details; details should depend on abstractions.

→ See [design-practices.md](./design-practices.md#solid-principles)

### Interface Segregation Principle (ISP)

Clients shouldn't depend on methods they don't use. Interfaces should be focused and specific to client needs.

→ See [design-practices.md](./design-practices.md#solid-principles)

### Liskov Substitution Principle (LSP)

Subtypes must be substitutable for their base types without altering program correctness.

→ See [design-practices.md](./design-practices.md#solid-principles)

### Open/Closed Principle (OCP)

Software entities should be open for extension but closed for modification.

→ See [design-practices.md](./design-practices.md#solid-principles)

### Single Responsibility Principle (SRP)

Each module should have one reason to change, focusing on a single aspect of functionality.

→ See [design-practices.md](./design-practices.md#solid-principles)

## Consistency and Lifecycle

### Creation

The business lifecycle event of bringing an entity into existence for the first time (e.g., user places an order). Must enforce all current invariants.

→ See [design-practices.md](./design-practices.md#entity-lifecycles-creation-vs-instantiation)

### Eventual Consistency

A consistency model where changes propagate across boundaries asynchronously. Used between aggregates and between bounded contexts.

→ See [design-practices.md](./design-practices.md#consistency-boundaries)

### Immediate Consistency

A consistency model where changes are atomic within a transaction. Used within an aggregate boundary.

→ See [design-practices.md](./design-practices.md#consistency-boundaries)

### Instantiation

The technical act of creating an object in memory (e.g., loading from database). Should bypass invariant checks to support temporal validity.

→ See [design-practices.md](./design-practices.md#entity-lifecycles-creation-vs-instantiation)

### Invariant

A business rule that must be consistent at a given point in time. Enforced at creation and modification time, but bypassed during instantiation to support temporal validity.

→ See [design-practices.md](./design-practices.md#invariants)

### Temporal Validity

The principle that invariants evolve over time. Data created under old rules remains valid under those rules; new data must satisfy current rules.

→ See [design-practices.md](./design-practices.md#temporal-validity-invariants-change-over-time)

## Anti-Patterns

### Anemic Domain Model

An anti-pattern where entities have only getters/setters, with business logic placed in services instead. Loses benefits of object-oriented design.

→ See [design-practices.md](./design-practices.md#anemic-domain-model)

### Leaky Abstraction

An anti-pattern where infrastructure concerns appear in the domain layer, or domain code depends on databases, HTTP, etc. Creates tight coupling and makes evolution difficult.

→ See [design-practices.md](./design-practices.md#leaky-abstractions)

### Smart UI

An anti-pattern where business logic resides in the presentation layer with direct database access. Makes testing impossible and maintenance difficult.

→ See [design-practices.md](./design-practices.md#smart-ui)

---

## Alphabetical Index

A: [Adapter](#adapter) · [Aggregate](#aggregate) · [Aggregate Root](#aggregate-root) · [Anemic Domain Model](#anemic-domain-model) · [Anti-Corruption Layer (ACL)](#anti-corruption-layer-acl)

B: [Bounded Context](#bounded-context)

C: [Conformist](#conformist) · [Creation](#creation) · [Customer-Supplier](#customer-supplier)

D: [Dependency Inversion Principle (DIP)](#dependency-inversion-principle-dip) · [Dependency Rule](#dependency-rule) · [Distance](#distance) · [Domain Behavior](#domain-behavior-domain-service) · [Domain Event](#domain-event)

E: [Entity](#entity) · [Eventual Consistency](#eventual-consistency)

I: [Immediate Consistency](#immediate-consistency) · [Instantiation](#instantiation) · [Interface Segregation Principle (ISP)](#interface-segregation-principle-isp) · [Intrusive Coupling](#intrusive-coupling) · [Invariant](#invariant)

L: [Leaky Abstraction](#leaky-abstraction) · [Liskov Substitution Principle (LSP)](#liskov-substitution-principle-lsp)

N: [Non-intrusive Coupling](#non-intrusive-coupling)

O: [Open Host Service](#open-host-service) · [Open/Closed Principle (OCP)](#openclosed-principle-ocp)

P: [Partnership](#partnership) · [Port](#port) · [Published Language](#published-language)

R: [Repository](#repository)

S: [Screaming Architecture](#screaming-architecture) · [Shared Kernel](#shared-kernel) · [Single Responsibility Principle (SRP)](#single-responsibility-principle-srp) · [Smart UI](#smart-ui)

T: [Temporal Validity](#temporal-validity) · [Translator](#translator)

U: [Ubiquitous Language](#ubiquitous-language) · [Use Case](#use-case)

V: [Value Object](#value-object) · [Volatility](#volatility)
