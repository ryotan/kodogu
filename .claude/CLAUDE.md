@../docs/.ai/guidelines.md
@../docs/.ai/design-practices.md

# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Language Rules

- Always respond in Japanese, even when asked in English
- Code comments: English
- Commit messages, PR titles, PR bodies: English
- Variable and function names: English

## Git Conventions

GitMoji + Conventional Commits format: `<emoji>: <summary>`

Emojis: ✨ feature, 🐛 bugfix, ♻️ refactor, ✅ test, 📝 docs, 🎨 structure, ⚡️ perf, 🔥 remove, 🚧 WIP

- Commit messages should describe the **purpose/intent** of the change, not what was changed
- PRs are squash-merged — the title becomes the commit subject and the body becomes the commit body
- PR title: follow the `<emoji>: <summary>` format
- PR body: explain the purpose and context of the change
- Do NOT add Co-Authored-By trailers
- Do NOT add "Generated with Claude Code" footers to PR bodies

### Branch Naming

Conventional Branch format: `<type>/<description>` (lowercase, hyphens only)

Types: `feature/`, `fix/`, `hotfix/`, `release/`, `chore/`

## Coding Philosophy

- **FP-first**: Prefer pure functions, immutable data, and iterator chains. Use structs/enums for data, functions for
  behavior. Isolate side effects at boundaries.
- **TDD**: Red-Green-Refactor cycle. Tests serve as specifications.
- **Implementation order**: Type design → Pure functions → Side-effect separation → Adapter implementation
- **Start small**: Avoid excessive abstraction. Focus on types rather than code. Extend incrementally.

## Error Handling Rules

- **Always use `#[suzunari_error]`** for defining error types. Do NOT define errors with raw `#[derive(Snafu)]` alone or
  hand-written `impl Error`.
- **Always propagate errors with `?` and `.context()`** (`snafu::ResultExt`). Do NOT use `unwrap()`, `expect()`,
  `.build()`, or `.fail()` in production code.
- **Test code**: `unwrap()` is acceptable. But prefer `.context()` propagation even in tests when it improves clarity.

## Error Design Principles

1. **Traceable** — Errors carry location context through the call stack
2. **Context-rich** — Include relevant parameters and state at each level
3. **Noise-free** — No duplicated information; focus on essentials for debugging
4. **Hierarchical** — Enum variants for categories, `source` chaining for the error trail
5. **Performance-conscious** — Defer formatting until display; minimize allocations in error paths
