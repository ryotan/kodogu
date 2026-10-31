# AGENTS.md

Instructions for AI agents working in this repository.

## Git Conventions

- Scope examples: `app`, `codec`, `crypto`, `ci`, `deps`
- PRs are squash-merged — the title becomes the commit subject and the body becomes the commit body

### Branch Naming

Conventional Branch format: `<type>/<description>` (lowercase, hyphens only)

Types: `feature/`, `fix/`, `hotfix/`, `release/`, `chore/`

### Workflow

- `main` is the main branch
- Always use `feature/*` or `hotfix/*` branches for development
- Do NOT commit directly to `main`
- Always use pull requests to merge

## Coding Philosophy

- **FP-first**: Prefer pure functions, immutable data, and iterator chains. Use structs/enums for data, functions for
  behavior. Isolate side effects at boundaries.
- **TDD**: Red-Green-Refactor cycle. Tests serve as specifications.
- **Implementation order**: Type design → Pure functions → Side-effect separation → Adapter implementation
- **Start small**: Avoid excessive abstraction. Focus on types rather than code. Extend incrementally.

## Error Handling Rules

- **Always use `#[suzunari_error]`** for defining error types. Do NOT define errors with hand-written `impl Error`.
- **Always propagate errors with `?` and `.context()`** (`suzunari_error::ResultExt`). Do NOT use `unwrap()`, `expect()`,
  `.build()`, or `.fail()` in production code.
- **Test code**: `unwrap()` is acceptable. But prefer `.context()` propagation even in tests when it improves clarity.

## Error Design Principles

1. **Traceable** — Errors carry location context through the call stack
2. **Context-rich** — Include relevant parameters and state at each level
3. **Noise-free** — No duplicated information; focus on essentials for debugging
4. **Hierarchical** — Enum variants for categories, `source` chaining for the error trail
5. **Performance-conscious** — Defer formatting until display; minimize allocations in error paths
