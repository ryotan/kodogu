# Rust Practices

Rust-specific practices for realizing universal principles in this language.

## Ownership and Borrowing

### Understand Ownership Rules

- Each value has exactly one owner
- When owner goes out of scope, value is dropped
- Ownership can be transferred (moved)
- **Why**: Memory safety without garbage collection

### Use References Appropriately

- `&T` for immutable borrows
- `&mut T` for mutable borrows
- Only one mutable reference OR many immutable references
- **Why**: Prevents data races at compile time

### Prefer Borrowing Over Ownership Transfer

- Pass `&T` or `&mut T` instead of `T` when possible
- Transfer ownership only when necessary
- Clone only when you must
- **Why**: Avoids unnecessary moves and copies

## Error Handling

See [coding-practices.md](./coding-practices.md#error-handling) for error classification philosophy.

For the architectural rationale behind Rust's error handling strategy (Visitor Pattern vs other approaches), see [ADR-0004](../adr/0004-error-handling-strategy-in-rust.md).

### Use suzunari-error for Error Hierarchy

> **Note**: `suzunari-error` is a custom error handling crate developed specifically
> for this project. It is **not available on crates.io** and must be added from the
> GitHub repository:
>
> ```toml
> [dependencies]
> suzunari-error = { git = "https://github.com/ryotan/suzunari-error.git", version = "0.1.0" }
> ```

- Apply `#[suzunari_error]` macro to error enums
- Provides automatic stack trace propagation through error chains
- Implement the error hierarchy pattern:

```rust
// Rust enum (no language-specific root type)
// └─ RootError (for centralized error handling)
//    ├─ Application { ApplicationError }
//    └─ Platform { PlatformError }

#[suzunari_error]
pub enum RootError {
    #[snafu(display("Application error: {source}"))]
    Application { source: ApplicationError },
    #[snafu(display("Platform error: {source}"))]
    Platform { source: PlatformError },
}

// Application errors (our code: domain, use case, adapter, framework)
#[suzunari_error]
#[suzu(context(suffix(Error)))]
pub enum ApplicationError {
    #[suzu(display("Domain error: {source}"))]
    Domain { source: DomainError },
    #[suzu(display("Infrastructure error: {source}"))]
    Infrastructure { source: InfrastructureError },
  // UseCaseError, AdapterError, etc. can be added as needed
}

// Domain-specific errors
#[suzunari_error]
#[suzu(context(suffix(Error)))]
pub enum DomainError {
    #[suzu(display("Order not found: {id}"))]
    OrderNotFound { id: String },
}

// Platform errors (external systems)
#[suzunari_error]
pub enum PlatformError {
    #[snafu(display("Database error: {source}"))]
    Database { source: DatabaseError },
    // KeyValueStore, PubSub, etc. can be added as needed
}
```

- **Why**: Type-safe error hierarchy with full context
- **Example**: See [rust-examples/error-handling.md](./rust-examples/error-handling.md)

### Use Result for Application and Platform Errors

- Application and Platform errors are recoverable errors in Rust terms
- `Result<T, E>` for operations that can fail
- `?` operator for error propagation
- **Why**: Explicit, type-safe error handling

### Use panic! for Programming Errors

- Programming errors indicate bugs
- Use `panic!`, `assert!`, `unwrap()` in tests only
- **Why**: Bugs should crash immediately, not be handled

### Use Option for Absence

- `Option<T>` when value may not exist
- Avoid using `unwrap()` except in tests
- Use `unwrap_or`, `unwrap_or_else`, `ok_or` instead
- **Why**: Forces handling of None case

## Type System

### Use Type-Driven Design

- Make illegal states unrepresentable
- Use enums for sum types
- Use structs for product types
- **Why**: Compiler enforces correctness

### Prefer Newtype Pattern

- `struct UserId(String)` for domain concepts
- Zero-cost abstraction
- Prevents mixing up similar types
- **Why**: Type safety without runtime overhead

### Use Enums for State Machines

- Model states as enum variants
- Attach data to variants
- Match exhaustively
- **Why**: Compiler ensures all states handled

## Pattern Matching

### Use Match Exhaustively

- Handle all enum variants explicitly
- Avoid `_` wildcard unless intentional
- Compiler warns about non-exhaustive matches
- **Why**: Catches bugs during refactoring

### Leverage Pattern Matching Features

- Destructuring in match arms
- Guards with `if` conditions
- `@` bindings for naming matched values
- **Why**: Expressive and concise

## Traits

### Use Traits for Abstraction

- Traits define shared behavior
- Implement traits for your types
- Use trait bounds in generic functions
- **Why**: Polymorphism without inheritance

### Prefer Trait Objects Sparingly

- `dyn Trait` has runtime cost
- Use generics with trait bounds when possible
- Use trait objects for heterogeneous collections
- **Why**: Performance vs flexibility trade-off

### Implement Standard Traits

- `Debug`, `Clone`, `PartialEq`, `Eq`, `Hash` as needed
- Derive when possible
- Manual implementation when derivation insufficient
- **Why**: Integrates with Rust ecosystem

## Lifetimes

### Understand Lifetime Elision

- Compiler infers lifetimes in common cases
- Explicit lifetimes only when necessary
- Named lifetimes for clarity in complex cases
- **Why**: Less annotation clutter

### Keep Lifetimes Simple

- Avoid multiple lifetime parameters when possible
- Use `'static` sparingly
- Let compiler guide you
- **Why**: Complex lifetimes are hard to maintain

## Collections

### Choose Right Collection

- `Vec<T>` for dynamic arrays
- `HashMap<K, V>` for key-value pairs
- `BTreeMap<K, V>` for sorted maps
- `HashSet<T>` for unique values
- **Why**: Each has different performance characteristics

### Understand Ownership in Collections

- `Vec<T>` owns its elements
- Iteration can borrow or consume
- `into_iter()` takes ownership
- `iter()` borrows
- **Why**: Prevents use-after-move bugs

## Iterators

### Prefer Iterator Chains

- `map`, `filter`, `fold` over manual loops
- Lazy evaluation for efficiency
- `collect()` to materialize results
- **Why**: More expressive, often faster

### Implement Iterator When Appropriate

- Custom iterators for domain types
- Stateful iteration
- Use `IntoIterator` for natural conversion
- **Why**: Integrates with ecosystem

## Smart Pointers

### Use Appropriate Smart Pointer

- `Box<T>` for heap allocation
- `Rc<T>` for shared ownership (single-threaded)
- `Arc<T>` for shared ownership (multi-threaded)
- `RefCell<T>` for interior mutability
- **Why**: Each solves different problems

### Avoid Smart Pointer Overuse

- Prefer ownership and borrowing
- Smart pointers add overhead
- Use only when necessary
- **Why**: Simplicity and performance

## Modules and Visibility

### Structure Code with Modules

- Use modules for organization
- `pub` for public API
- Keep internals private by default
- **Why**: Clear API boundaries

### Use `pub(crate)` for Internal APIs

- Visible within crate, hidden from consumers
- Better than full `pub` for internal helpers
- Enables refactoring without breaking changes
- **Why**: Encapsulation within crate

## Async/Await

### Use Async for I/O-Bound Work

- `async fn` for asynchronous functions
- `.await` to wait for futures
- Choose runtime (tokio, async-std, smol)
- **Why**: Efficient concurrency

### Understand Send and Sync

- `Send` types can be moved between threads
- `Sync` types can be shared between threads
- Async functions require `Send` futures for tokio
- **Why**: Thread safety guarantees

### Handle Cancellation

- Futures are lazy and cancellable
- Clean up on drop
- Use `select!` for racing futures
- **Why**: Resource management

## Macros

### Use Macros Sparingly

- Prefer functions and generics
- Macros for code generation only when needed
- Procedural macros for complex cases
- **Why**: Macros harder to debug and understand

### Leverage Standard Macros

- `vec![]`, `format!()`, `println!()`
- `derive` macros for traits
- `cfg!()` for conditional compilation
- **Why**: Well-tested and familiar

## Cargo and Dependencies

### Manage Dependencies Carefully

- Use semantic versioning
- Pin versions in Cargo.lock
- Regular dependency updates
- **Why**: Reproducible builds, security

### Use Cargo Features

- Optional dependencies via features
- Feature flags for conditional code
- Default features for common use cases
- **Why**: Modular dependency management

## Testing

### Write Unit Tests in Same File

- `#[cfg(test)]` mod tests
- Tests close to implementation
- Private function testing enabled
- **Why**: Easy to maintain

### Use Integration Tests

- `tests/` directory for integration tests
- Test public API
- Each file is separate crate
- **Why**: Tests from consumer perspective

### Leverage Test Attributes

- `#[test]` for tests
- `#[should_panic]` for expected panics
- `#[ignore]` for slow tests
- **Why**: Flexible test control

### Use Standard Testing Tools

- Rust's built-in test framework covers most needs
- `cargo test` is the standard - no external framework required
- Add specialized tools only when specific needs arise
- **Why**: Standard tooling is well-integrated, maintained, and sufficient

## Documentation

### Write Rustdoc Comments

- `///` for item documentation
- `//!` for module documentation
- Use markdown in doc comments
- **Why**: Integrated documentation

### Include Examples in Docs

- Code blocks in doc comments are tests
- `# ` prefix for hidden lines
- Demonstrate usage in examples
- **Why**: Docs stay up to date

## Safety and Unsafe

### Avoid Unsafe When Possible

- Most code doesn't need `unsafe`
- Understand invariants before using unsafe
- Minimize unsafe boundaries
- **Why**: Safety is Rust's strength

### Document Unsafe Invariants

- Comment why unsafe is needed
- Document what invariants must hold
- Review unsafe code carefully
- **Why**: Future maintainers need context

## Performance

### Measure Before Optimizing

- Use `cargo bench` for benchmarks
- Profile with appropriate tools
- Don't assume, measure
- **Why**: Premature optimization wastes time

### Understand Zero-Cost Abstractions

- Generics monomorphized (no runtime cost)
- Iterators optimize to same as loops
- Newtype pattern has no overhead
- **Why**: Abstraction without performance penalty

### Use Release Builds for Performance

- Debug builds prioritize compile time
- Release builds enable optimizations
- Always benchmark release builds
- **Why**: Debug performance not representative

## Common Pitfalls

### Avoid These Patterns

- `.unwrap()` in production code
- Cloning everything to avoid borrow checker
- Fighting the borrow checker instead of learning
- Ignoring compiler warnings
- **Why**: These indicate misunderstanding

## Code Style

### Follow Rustfmt

- Use `cargo fmt` for formatting
- Consistent style across ecosystem
- Integrate into CI/CD
- **Why**: No debates about style

### Use Clippy

- Run `cargo clippy` regularly
- Fix warnings, don't suppress
- Teaches Rust idioms
- **Why**: Catches common mistakes

## Tauri Commands

### Sync vs Async Commands

- **Sync commands** (`#[tauri::command]`): Run on the main thread. Use only for trivial, non-blocking operations
- **Async commands** (`#[tauri::command] async fn`): Run on the tokio runtime. Use for any I/O (file, database, network)

```rust
// Sync — only for trivial operations
#[tauri::command]
fn get_version() -> String { env!("CARGO_PKG_VERSION").to_string() }

// Async — for anything involving I/O
#[tauri::command]
async fn get_config(store: State<'_, AppConfigStore>) -> Result<AppConfig, CommandError> {
    store.get().context(GetConfigFailed)
}
```

- **Why**: Sync commands block the main thread (UI freezes). Async commands run on the tokio thread pool

## Ecosystem Libraries

### Prefer Standard Library

- Use `std` when sufficient
- Battle-tested and stable
- Part of language guarantee
- **Why**: Minimal dependencies

### Use Well-Maintained Crates

- `serde` for serialization
- `tokio` or `async-std` for async runtime
- `suzunari-error` for errors
- `clap` for CLI parsing
- **Why**: Ecosystem best practices
