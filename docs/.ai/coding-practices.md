# Coding Practices

These practices apply across languages, though each language realizes them differently.

## Immutability

### Prefer Immutable Data

- Default to immutable data structures
- Mutability should be explicit and intentional
- Immutable data eliminates entire classes of bugs
- **Why**: Easier to reason about, thread-safe, prevents unintended side effects

### Make Mutation Explicit

- When mutation is necessary, make it visible
- Use clear naming (e.g., `update`, `modify`, `set`)
- Limit mutation scope (local variables, builder patterns)
- **Why**: Readers immediately know where state changes

## Null Safety

### Avoid Null When Possible

- Use language-specific null safety features
- Prefer Optional/Option/Maybe types over nullable references
- Make nullability explicit in type signatures
- **Why**: Eliminates null pointer exceptions at compile time

### Handle Absence Explicitly

- Don't return null for "not found" - use Option types
- Don't use null for "not initialized" - use lazy initialization or builders
- Don't use null for "not applicable" - model with sum types
- **Why**: Forces explicit handling, documents intent

## Error Handling

### Error Classification

**Domain Errors**: Business rule violations
- Represented as custom types
- Examples: OrderNotFound, InsufficientStock

**Infrastructure Errors**: External system failures
- Represented as custom types
- Examples: NetworkTimeout, DatabaseError

**Programming Errors**: Bugs in code (null pointer, index out of bounds, etc.)
- Not represented as custom types
- Detected via language assertions

### Design Principle

Model Domain/Infrastructure errors as a hierarchy:

```
ApplicationError
├─ DomainError
│  ├─ OrderNotFound
│  └─ InvalidInput
└─ InfrastructureError
   ├─ NetworkTimeout
   └─ DatabaseError
```

Programming errors use language mechanisms (panic, check, etc.).

See language-specific practices for implementation.

### Make Errors Explicit

- Errors are part of your API contract
- Default to exceptions with a global error handling system for centralized management
- Use Result/Either types only when callers must handle errors differently based on the error type
- **Why**: Centralized error handling reduces boilerplate and ensures consistent error responses
- **Why**: Result types make error handling explicit when caller-specific logic is needed

### Fail Fast

- Validate inputs at boundaries immediately
- Don't propagate invalid state
- Throw/return errors as soon as detected
- **Why**: Bugs are caught close to their source

### Provide Context

- Error messages should explain what went wrong
- Include relevant data in error types
- Chain errors to preserve context
- **Why**: Debugging is faster and easier

## Function Design

### Keep Functions Small

- One function, one responsibility
- Aim for < 20 lines for most functions
- Extract complex logic into well-named helpers
- **Why**: Easier to understand, test, and reuse

### Minimize Side Effects

- Pure functions when possible
- Side effects should be explicit (return types, naming)
- Isolate I/O and mutation
- **Why**: Easier to reason about and test

### Use Meaningful Names

- Function names describe what they do
- Parameter names describe what they represent
- Avoid abbreviations unless universally understood
- **Why**: Self-documenting code

## Data Modeling

### Make Illegal States Unrepresentable

- Use types to encode business rules
- Prefer enums/sum types over booleans
- Don't allow impossible combinations
- **Why**: Compiler enforces correctness

### Encapsulate Invariants

- Keep data private, expose operations
- Validate in constructors/factory functions
- Never expose setters that can break invariants
- **Why**: Guarantees are maintained throughout object lifetime

### Use Meaningful Types

- Don't use primitives for domain concepts
- Create value objects (Email, UserId, Money)
- Use type aliases/newtype for clarity
- **Why**: Type system helps catch mistakes

## Concurrency

### Minimize Shared Mutable State

- Prefer message passing over shared memory
- Use immutable data structures
- Isolate mutation to specific components
- **Why**: Reduces race conditions and complexity

### Use Language Idioms

- Kotlin: coroutines and flows
- Rust: ownership and async/await
- TypeScript: Promises and async/await
- **Why**: Language-specific tools are optimized and idiomatic

## Resource Management

### Clean Up Resources

- Use language RAII/defer/try-with-resources patterns
- Never rely on finalization/garbage collection for cleanup
- Make resource lifecycles explicit
- **Why**: Prevents leaks and ensures deterministic cleanup

### Limit Resource Scope

- Acquire late, release early
- Use scoped resource patterns
- Make ownership explicit
- **Why**: Reduces resource contention and memory usage

## Code Organization

### Group Related Functionality

- High cohesion within modules
- Low coupling between modules
- Related types and functions stay together
- **Why**: Easier to find and modify related code

### Minimize Dependencies

- Depend on abstractions, not concretions
- Keep dependency graphs acyclic
- Make dependencies explicit
- **Why**: Easier to test, modify, and understand

## Performance Considerations

### Optimize for Readability First

- Clear code is fast enough most of the time
- Measure before optimizing
- Document why optimizations are needed
- **Why**: Premature optimization is the root of all evil

### Be Allocation-Aware

- Avoid unnecessary allocations in hot paths
- Reuse buffers when appropriate
- Profile before optimizing
- **Why**: Memory allocation can be expensive

### Choose Appropriate Data Structures

- Understand time/space complexity
- Use the right tool for the job (List vs Set vs Map)
- Consider access patterns
- **Why**: Wrong data structure can kill performance

## Testing Support

### Design for Testability

- Inject dependencies
- Avoid static state
- Keep functions pure when possible
- **Why**: Testable code is good code

### Make Test Doubles Easy

- Use interfaces/traits for dependencies
- Avoid final/sealed unless intentional
- Support dependency injection
- **Why**: Tests can replace production dependencies

## Documentation

### Document Why, Not What

- Code shows what, comments explain why
- Document non-obvious decisions
- Explain trade-offs and constraints
- **Why**: Future developers need context

### Keep Documentation Close

- Doc comments on public APIs
- Inline comments for complex logic
- Architecture decisions in ADRs
- **Why**: Documentation that's far from code gets stale

## Async/Await Patterns

### Async All the Way

- Don't block on async operations
- Use async/await consistently
- Avoid mixing sync and async
- **Why**: Blocking defeats the purpose of async

### Handle Cancellation

- Support graceful cancellation
- Clean up on cancellation
- Propagate cancellation tokens
- **Why**: Long-running operations need to be stoppable

## API Design

### Favor Narrow Interfaces

- Expose only what's necessary
- Many small interfaces over few large ones
- Make it hard to misuse
- **Why**: Smaller surface area, easier to maintain

### Version Breaking Changes

- Semantic versioning
- Deprecate before removing
- Provide migration paths
- **Why**: Respects downstream consumers

## Security

### Validate All Inputs

- Never trust external data
- Sanitize at boundaries
- Use allowlists over denylists
- **Why**: Prevents injection attacks and crashes

### Minimize Attack Surface

- Principle of least privilege
- Fail closed, not open
- Don't expose internals
- **Why**: Security by design

## Logging and Observability

### Log at Appropriate Levels

- ERROR: something failed
- WARN: something unexpected
- INFO: important events
- DEBUG: detailed diagnostics
- **Why**: Logs are useful for debugging and monitoring

### Include Context

- Request IDs, user IDs, correlation data
- Structured logging when possible
- Don't log sensitive data
- **Why**: Makes troubleshooting possible

## Implementation Workflow

### Recommended Development Sequence

A structured approach to implementation that prioritizes type safety, testability, and separation of concerns.

### 1. Type Design First

- Define types before implementation
- Represent domain concepts with types
- Make illegal states unrepresentable
- **Why**: Type-driven design catches errors early

### 2. Start with Pure Functions

- Implement functions without external dependencies first
- Write tests before implementation (TDD)
- Focus on business logic in isolation
- **Why**: Pure functions are easiest to test and reason about

### 3. Separate Side Effects

- Push I/O operations to function boundaries
- Keep side effects explicit in signatures
- Isolate mutation from pure logic
- **Why**: Makes code testable and maintainable

### 4. Implement Adapters

- Abstract external services and databases
- Define interfaces in domain layer
- Create test doubles for testing
- **Why**: Enables testing without external dependencies

### 5. Ensure Quality

- Run linters and fix all warnings
- Run tests and ensure they pass
- Review test coverage
- **Why**: Catches issues before they reach production

## Language-Specific Realization

Each language realizes these principles differently:

**Kotlin**: Data classes, null safety, coroutines, sealed classes
**Rust**: Ownership, Result<T,E>, traits, async/await
**TypeScript**: Readonly, union types, Promises, strict null checks

See language-specific practice files for details.
