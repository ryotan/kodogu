# AI Agent Guidelines

This file serves as the index for all AI practice documents in this project.

> [!Important]
> 
> The user is better at programming than you (AI Agents), but we ask you to code for us to save time.
> 
> If a test fails two or more times in a row, we'll figure out the current situation and work together to find a solution.
> 
> You might get a ton of know-how from GitHub, and her use of individual algorithms and libraries is gonna be faster than my
> implementation. you'll write the test code and check the operation, while explaining the code to the user.
> 
> On the other hand, you are not so great at processing according to the current context. When the context is unclear, I
> check with the user.

## Available Practice Files

### General Project Information

- [README.md](../../README.md) - Project overview, features, and setup introduction
- [CONTRIBUTING.md](../../CONTRIBUTING.md) - Detailed contribution guidelines, including project structure, development
  environment setup, and workflow

### Universal Principles (Language-agnostic)

- [**design-practices.md**](./design-practices.md) - Core design principles (DDD, boundaries, dependencies)
- [**testing-practices.md**](./testing-practices.md) - Testing principles across all levels
- [**coding-practices.md**](./coding-practices.md) - Cross-language implementation guidance
- [**git-practices.md**](./git-practices.md) - Git commit message conventions

### Language-Specific Practices

- [**kotlin-practices.md**](./kotlin-practices.md) - Kotlin idioms and conventions
- [**rust-practices.md**](./rust-practices.md) - Rust patterns and best practices
- [**typescript-practices.md**](./typescript-practices.md) - TypeScript guidelines

### Examples (Created when needed)

- `kotlin-examples.md` or files in `kotlin-examples`
- `rust-examples.md` or files in `rust-examples`
- `typescript-examples.md` or files in `typescript-examples`

## Loading Strategy

Load only what's needed for your current task to optimize token usage.

### Loading Matrix

| Task            | design | testing | coding | {lang} | domain |
|-----------------|--------|---------|--------|--------|--------|
| New Feature     | ✓      | ✓       | -      | -      | ✓      |
| Write Tests     | -      | ✓       | -      | ✓      | ✓      |
| Refactor        | ✓      | -       | ✓      | ✓      | -      |
| Code Review     | ✓      | ✓       | -      | ✓      | ✓      |
| Bug Fix         | -      | -       | -      | ✓      | ✓      |
| Domain Modeling | ✓      | ✓       | -      | -      | ✓      |
| Infrastructure  | -      | -       | ✓      | ✓      | -      |

**Legend:**

- design = [design-practices.md](./design-practices.md)
- testing = [testing-practices.md](./testing-practices.md)
- coding = [coding-practices.md](./coding-practices.md)
- {lang} = [kotlin](./kotlin-practices.md), [rust](./rust-practices.md), or [typescript](./typescript-practices.md)
- domain = src/{context}/.ai/context.md

## Domain-Specific Context

Each bounded context has its own `.ai/context.md`:

- `src/user-context/.ai/context.md`
- `src/order-context/.ai/context.md`
- `src/shared-kernel/.ai/context.md`

Load the relevant domain context when working within that context.

## Integration Examples

### Claude Code

Recommended minimal CLAUDE.md:

```markdown
@docs/.ai/guidelines.md

Multi-module DDD architecture. Each bounded context in src/*-context/ has .ai/context.md.
```

For task-specific agents:

```markdown
# .claude/agents/kotlin-feature.md
@docs/.ai/design-practices.md
@docs/.ai/kotlin-practices.md
```

### Junie

Link to this file in `.junie/guidelines.md`:

```markdown
See [AI Guidelines](../../docs/.ai/guidelines.md) for complete practices.
```

### JetBrains AI Assistant

Link to this file in `.aiassistant/rules/project.md`:

```markdown
See [AI Guidelines](../../docs/.ai/guidelines.md) for complete practices.
```

## Quick Reference by Task

> [!Important]
> This section is for JetBrains Junie and JetBrains AI Assistant. And NOT FOR Claude Code subagents.

**Starting new feature:**

1. Read design-practices.md
2. Read relevant domain's .ai/context.md
3. Read {language}-practices.md

**Writing tests:**

1. Read testing-practices.md
2. Read {language}-practices.md
3. Check domain's .ai/context.md for invariants

**Code review:**

1. Read design-practices.md
2. Read testing-practices.md
3. Read {language}-practices.md
4. Verify domain rules from .ai/context.md

**Refactoring:**

1. Read design-practices.md
2. Read coding-practices.md
3. Read {language}-practices.md
