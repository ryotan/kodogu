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

## Working Style

### Progress Tracking

When executing multi-step plans or analyses:

- Clearly indicate current progress (e.g., "Step 2 of 5: Analyzing domain boundaries")
- Complete one step at a time
- Wait for user confirmation before proceeding to the next step
- Provide a brief summary of what was accomplished after each step

### Incremental Delivery

- Break work into visible checkpoints
- Show intermediate results whenever possible
- Allow for course correction at each stage
- Make it easy to pause and resume work

## Communication Protocol

### When Seeking User Confirmation

When you need user input, decisions, or validation:

**One Question at a Time**

- Present only one question or decision point per message
- Do not dump multiple questions at once
- Wait for user response before presenting the next question

**Question Format**

- Keep each question concise: **20-30 lines maximum** (user should grasp the full context without scrolling)
- Include necessary context, options, and implications
- Make the decision point clear and actionable
- If you have multiple questions, show progress: "Question 1 of 3:", "Question 2 of 3:", etc.

**Progressive Disclosure**
If you have multiple questions:

1. Ask the first question
2. Wait for response
3. Then ask the second question
4. Continue until all questions are answered

### Examples

**Good - Single Focused Question:**

```
Question 1/3: 集約の境界について

Paymentは、Order集約の一部とすべきでしょうか、それとも独立した集約にすべきでしょうか？

選択肢A: Order集約の一部とする
- メリット: シンプル、即座の整合性、単一トランザクション
- デメリット: Orderが肥大化、結合度が高くなる

選択肢B: 独立した集約にする
- メリット: 関心の分離、独立したライフサイクル
- デメリット: 結果整合性が必要、複雑性が増す

背景: 支払いは注文完了後も返金や調整が発生します。

どちらが良いと思いますか？
```

**Bad - Multiple Questions at Once:**

```
❌ いくつか質問があります：
1. Paymentは Order集約の一部にすべきですか？
2. 返金処理はどう扱いますか？
3. リポジトリパターンはどうしますか？
4. ドメインイベントを使いますか？
5. 整合性モデルはどれが良いですか？
```

All questions dumped at once - overwhelming

**Bad - Too Long:**

```
集約設計についての理論全体、すべての可能なパターン、各アプローチの詳細な説明、歴史的背景、他のシステムからの10個の異なる例、それぞれの詳細な長所と短所、そしてこの1つの質問の中に5つのネストされたサブ質問があります...

❌ Here's everything about aggregate design including all the theory, all possible patterns, detailed explanations of each approach, historical context, 10 different examples from other systems, pros and cons of each with extensive detail, and here are 5 nested sub-questions within this one question...
```

50+ lines - requires scrolling to understand

### Automatic Execution is Fine

You can execute multiple steps automatically **without confirmation** as long as no user decisions are needed. Only
pause and ask when user input is actually required.

**Example Flow:**

```
[Auto-executes: Analyzes domain, identifies aggregates, maps relationships]
[Auto-executes: Creates initial design recommendations]
[Pauses: Needs user decision on consistency model]

Question: Order→Paymentの関係について、即座の整合性と結果整合性のどちらを使うべきでしょうか？
[Waits for response]

[Auto-executes: Applies decision, completes design]
[Pauses: Needs user decision on repository pattern]

Question: リポジトリは...
[Continues this pattern]
```

### Key Principle

**When asking = concise and focused. When executing = work efficiently without unnecessary pauses.**

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
