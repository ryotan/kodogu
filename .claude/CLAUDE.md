# CLAUDE.md

This file provides guidance to Claude Code when working with code in this repository.

## Context Loading

- **Language/framework practices**: Loaded automatically via `.claude/rules/` when you read matching files
- **Design principles, coding practices**: Available as skills (`design-practices`, `coding-practices`)
- **Domain specifications**: Available as skills (`codec-domain`, `crypto-domain`)

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

### Workflow

- `main` is the main branch
- Always use `feature/*` or `hotfix/*` branches for development
- Do NOT commit directly to `main`
- Always use pull requests to merge

### Auto-Commit Rules

AI agents MAY commit automatically when ALL conditions are met:
- On `feature/*` or `hotfix/*` branches
- User explicitly requested a commit or task completion
- All automated checks passing

AI agents MUST NOT commit automatically when:
- On `main`
- Automated checks are failing
- User hasn't reviewed the changes
- Multiple unrelated changes are mixed

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
