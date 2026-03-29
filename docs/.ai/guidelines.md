# AI Agent Guidelines

**Audience**: All AI coding assistants (Claude Code, Junie, JetBrains AI Assistant, GitHub Copilot, etc.)

This file serves as the index for all AI practice documents in this project.

## Available Practice and Context Files

You might get a ton of know-how from GitHub, and your use of individual algorithms and libraries is gonna be faster
than my implementation. But YOU MUST follow my practices.

### General Project Information

- [README.md](../../README.md) - Project overview, features, and setup introduction
- [CONTRIBUTING.md](../../CONTRIBUTING.md) - Detailed contribution guidelines, including project structure, development
  environment setup, and workflow

### Universal Principles (Language-agnostic)

- [**design-practices.md**](./design-practices.md) - Core design principles (DDD, boundaries, dependencies)
- [**coding-practices.md**](./coding-practices.md) - Cross-language implementation guidance
- [**technical-glossary.md**](./technical-glossary.md) - Domain terminology reference

### Language/Framework-Specific Practices

For Claude Code: these are loaded automatically via `.claude/rules/` when reading matching files.

For other AI agents: read the corresponding rule file directly:
- `.claude/rules/rust.md` - Rust patterns and best practices
- `.claude/rules/typescript.md` - TypeScript guidelines
- `.claude/rules/kotlin.md` - Kotlin idioms and conventions
- `.claude/rules/tauri-frontend.md` - Tauri frontend (TanStack Query/Router, Radix UI, Vitest)
- `.claude/rules/tauri-backend.md` - Tauri backend (commands, redb, state management)
- `.claude/rules/testing.md` - Testing principles across all levels

### Language-Specific Examples (Created when needed)

- `rust-examples.md` or files in `rust-examples`

## Domain-Specific Context

Each module has its own `.ai/context.md` with domain specifications:

- `kernel/codec/.ai/context.md` - Encoding/decoding library
- `kernel/crypto/.ai/context.md` - Cryptographic operations library

Load the relevant domain context when working within that module.

## Loading Strategy

Load only what's needed for your current task to optimize token usage.

### Loading Matrix

| Task                    | design | coding | {lang/fw} | domain |
|-------------------------|--------|--------|-----------|--------|
| New Feature (Design)    | ✓      | -      | -         | ✓      |
| New Feature (Implement) | ✓      | ✓      | ✓         | ✓      |
| Write Tests             | -      | -      | ✓         | ✓      |
| Refactor                | ✓      | ✓      | ✓         | -      |
| Code Review             | ✓      | -      | ✓         | ✓      |
| Bug Fix                 | -      | -      | ✓         | ✓      |
| Domain Modeling         | ✓      | -      | -         | ✓      |
| Infrastructure          | -      | ✓      | ✓         | -      |

**Legend:**

- design = [design-practices.md](./design-practices.md)
- coding = [coding-practices.md](./coding-practices.md)
- {lang/fw} = language/framework rule from `.claude/rules/`
- domain = kernel/{module}/.ai/context.md
