---
paths:
  - "app/src-tauri/**/*.rs"
---

# KoDoGu App Backend Practices

Framework-specific practices for the Tauri desktop app backend (`app/src-tauri/`).

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

### State Management

- Use `State<'_, T>` to access managed state from commands
- Register state with `app.manage(T)` in `setup()`
- State types must be `Send + Sync`
- **Why**: Thread-safe shared state across async commands

### Error Handling in Commands

- Commands return `Result<T, CommandError>` for IPC serialization
- Use `CommandError` as the top-level error type for all commands
- Propagate inner errors with `.context(CommandErrorKind)` using suzunari_error
- **Why**: Consistent error format for frontend consumption

### Command Module Organization

```
app/src-tauri/src/
  commands/        # Tauri command handlers (one file per domain)
    mod.rs         # Re-exports all commands
    config.rs      # Config-related commands
  domain/          # Domain types and logic
  store/           # Persistence (redb, etc.)
```

- Commands are thin adapters: validate input → delegate to domain/store → return result
- Do NOT put business logic in command handlers

## redb (Embedded Database)

- Use `redb` for local key-value persistence
- Define table constants with `const TABLE: TableDefinition<K, V>`
- Wrap database access in a store type with `Arc<Database>`
- Read operations: `read_txn()` → `open_table()` → `get()`
- Write operations: `write_txn()` → `open_table()` → `insert()` → `commit()`
- **Why**: Embedded, zero-config, ACID-compliant local storage for desktop apps

### Atomicity

- Hold write lock across both database write and in-memory cache update
- Do NOT update cache before confirming database write success
- **Why**: Prevents stale reads when cache and database diverge
