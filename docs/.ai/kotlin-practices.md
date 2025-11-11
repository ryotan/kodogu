# Kotlin Practices

Kotlin-specific practices for realizing universal principles in this language.

## Null Safety

### Use Nullable Types Explicitly

- `String?` when null is possible, `String` when it's not
- Avoid `!!` operator - it defeats null safety
- Use `?.let {}` for null-safe operations
- Use `?:` (Elvis) for default values
- **Why**: Compiler enforces null safety

### Prefer Non-Null Types

- Make types non-nullable by default
- Use `lateinit` sparingly, only for dependency injection
- Consider lazy initialization over lateinit
- **Why**: Fewer null checks, clearer intent

## Data Classes

### Use Data Classes for Value Objects

- Automatic equals/hashCode/copy
- Perfect for domain value objects
- Keep them immutable with `val` properties
- **Why**: Less boilerplate, value semantics

### Avoid Mutable Data Classes

- Use `val` not `var` in data classes
- Use `copy()` for modifications
- Data classes should represent values, not entities
- **Why**: Immutability prevents bugs

## Sealed Classes and When Expressions

### Model Domain States with Sealed Classes

- Use sealed classes for state machines
- Represent "one of several" with sealed classes
- Exhaustive `when` catches missing cases
- **Why**: Compiler ensures all cases handled

### Leverage Exhaustive When

- Don't use `else` in `when` on sealed classes
- Let compiler warn about missing branches
- Use `when` as an expression, not just statement
- **Why**: Compiler helps maintain code during refactoring

## Coroutines

### Use Structured Concurrency

- Launch coroutines in appropriate scopes
- Use `coroutineScope` for parallel work
- Don't use `GlobalScope` except for truly global work
- **Why**: Prevents leaks, enforces lifecycle

### Handle Cancellation

- Check `isActive` in long-running loops
- Use `ensureActive()` at cancellation points
- Clean up resources on cancellation
- **Why**: Responsive cancellation, no resource leaks

### Choose Right Dispatcher

- `Dispatchers.Main` for UI updates
- `Dispatchers.IO` for blocking I/O
- `Dispatchers.Default` for CPU-intensive work
- **Why**: Performance and responsiveness

## Extension Functions

### Use Extensions for Utility Functions

- Extension functions for cross-cutting concerns
- Don't extend types you don't own with domain logic
- Keep extensions focused and cohesive
- **Why**: Clean syntax without modifying original types

### Avoid Extension Function Abuse

- Don't replace proper OOP with extensions
- Core domain logic belongs in classes
- Extensions are for convenience, not architecture
- **Why**: Maintains clear responsibilities

## Scope Functions

### Use Appropriate Scope Function

- `let`: transformation and null safety
- `run`: object configuration and computation
- `with`: multiple calls on same object
- `apply`: object initialization
- `also`: side effects while chaining
- **Why**: Each has clear semantics

### Don't Nest Scope Functions

- Nested scope functions are hard to read
- Extract to separate functions instead
- Use meaningful variable names over `it`
- **Why**: Readability

## Collections

### Prefer Immutable Collections

- Use `List`, `Set`, `Map` not `MutableList`, etc.
- Return immutable collections from functions
- Use `toList()` to create defensive copies
- **Why**: Prevents unintended modifications

### Use Collection Operations

- `map`, `filter`, `fold` over manual loops
- `firstOrNull()` instead of `first()` when item might not exist
- `associate` for building maps
- **Why**: More expressive, less error-prone

### Be Sequence-Aware

- Use `Sequence` for large or infinite collections
- Use `asSequence()` for multiple operations
- Understand lazy vs eager evaluation
- **Why**: Performance optimization

## Type System

### Use Type Aliases for Clarity

- `typealias UserId = String` for domain concepts
- Don't overuse - value classes are better for type safety
- Use for complex generic types
- **Why**: Improves readability

### Prefer Value Classes for Type Safety

- `@JvmInline value class UserId(val value: String)`
- Zero runtime overhead with inline classes
- Type safety at compile time
- **Why**: Prevents mixing up similar types

## Object and Companion Objects

### Use Object for Singletons

- `object` for stateless utilities
- Thread-safe by default
- Lazy initialized on first access
- **Why**: Simple singleton pattern

### Use Companion Objects for Factory Methods

- Static factory methods in companion objects
- Named constructors via companion functions
- Keep companion objects focused
- **Why**: Clear object creation patterns

## Delegation

### Use Delegation When Appropriate

- Property delegation for cross-cutting concerns
- Class delegation for composition over inheritance
- Built-in delegates: `lazy`, `observable`, `vetoable`
- **Why**: Reduces boilerplate

### Create Custom Delegates Sparingly

- Only when reused across multiple properties
- Document behavior clearly
- Keep delegates simple
- **Why**: Complexity vs benefit trade-off

## Contracts

### Use Contracts for Smart Casts

- Contracts help compiler understand guarantees
- Standard library uses contracts extensively
- Custom contracts for domain-specific checks
- **Why**: Better type inference, fewer casts

## Inline Functions

### Use Inline for Higher-Order Functions

- `inline` for functions taking lambda parameters
- Prevents lambda allocation overhead
- Use `noinline` for parameters passed to other functions
- **Why**: Performance without sacrificing readability

### Don't Inline Large Functions

- Inline increases bytecode size
- Only inline small, frequently-called functions
- Measure performance impact
- **Why**: Code bloat vs performance trade-off

## Operators

### Implement Operators Semantically

- Only overload operators when meaning is clear
- `+` for addition/concatenation, not arbitrary operations
- Follow mathematical conventions
- **Why**: Meets user expectations

### Don't Abuse Operator Overloading

- Prefer named functions for non-obvious operations
- Operators should be intuitive
- Document non-standard operator meanings
- **Why**: Readability over cleverness

## Error Handling

See [coding-practices.md](./coding-practices.md#error-handling) for error classification philosophy.

### Implement Error Hierarchy with Custom Exception Classes

- Model Domain and Infrastructure errors as exception hierarchy
- For Programming errors (implementation bugs), use `require()` to validate function preconditions and `check()` to assert internal state
- For Domain/Infrastructure errors, always use custom exception classes (see below)

```kotlin
open class ApplicationError : Exception()
open class DomainError : ApplicationError()
open class InfrastructureError : ApplicationError()

class OrderNotFound(val id: String) : DomainError()
class NetworkTimeout(val url: String) : InfrastructureError()
```

- Use `@ControllerAdvice` or similar for centralized error handling
- **Why**: Type-safe error classification, open for extension

### Use runCatching for Functional Error Handling

- `runCatching` converts exceptions to `Result`
- Chain operations with `mapCatching`
- Use `getOrElse` for fallback values
- **Why**: Functional style with exception hierarchy

## String Templates

### Use String Templates

- `"User $name"` over `"User " + name`
- `"Result: ${computeValue()}"` for expressions
- Triple quotes `"""` for multiline strings
- **Why**: More readable, less error-prone

## Destructuring

### Use Destructuring for Clarity

- `val (name, age) = person` for data classes
- `for ((key, value) in map)` for iteration
- Don't destructure if only using some components
- **Why**: Concise code when appropriate

## Kotlin-Specific Testing

### Use Kotlin Test DSLs

- kotlinx.test for multiplatform
- Kotest for BDD-style tests
- MockK for mocking (not Mockito)
- **Why**: Better Kotlin integration

### Leverage Language Features in Tests

- Extension functions for test utilities
- Infix functions for readable assertions
- Sealed classes for test data variants
- **Why**: Tests should be as maintainable as production code

## Android-Specific (if applicable)

### Use Kotlin Android Extensions Carefully

- Prefer ViewBinding over synthetic properties
- Use `by viewModels()` delegate
- Leverage Kotlin coroutines for async
- **Why**: Modern Android development patterns

## Common Pitfalls

### Avoid These Patterns

- `!!` operator in production code
- `lateinit` for nullable types
- Mutable data class properties
- `GlobalScope.launch` in application code
- Swallowing exceptions silently
- **Why**: These patterns lead to bugs

## Code Style

### Follow Official Kotlin Style Guide

- 4-space indentation
- No wildcard imports
- Expression body for single-expression functions
- **Why**: Consistency across Kotlin ecosystem

### Use kotlinx Libraries

- kotlinx.coroutines for async
- kotlinx.serialization for JSON
- Arrow for functional programming
- **Why**: Official libraries, well-maintained
