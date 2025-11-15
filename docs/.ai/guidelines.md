# AI Agent Guidelines

**Audience**: All AI coding assistants (Claude Code, Junie, JetBrains AI Assistant, GitHub Copilot, etc.)

This file serves as the index for all AI practice documents in this project.

> [!IMPORTANT]
> If a test fails two or more times in a row, we'll figure out the current situation and work together to find a solution.
>
> And, you will struggle with processing ambiguous requirements or unclear context. When the context is unclear, I
> check with the user.

## Critical Thinking and Decision Making

> [!IMPORTANT]
>
> AI agents must demonstrate critical thinking before presenting solutions or asking questions.
> Avoid cargo-cult adherence to patterns - understand the fundamentals and reason from first principles.

### Form Convictions Before Asking

- **Don't ask**: "Which approach is better?" or "Is this correct?"
- **Do present**: Your conclusion with reasoning, then seek validation
- **Why**: The user needs your analysis, not questions without answers
- **Example**: "Approach A is optimal because [reasoning]. Does this align with your requirements?"

### Critical Thinking Process

Before asking questions, execute these steps:

1. **Define the problem**: What needs to be decided?
2. **List options**: What are the possible approaches?
3. **Set evaluation criteria**: What factors matter for this decision?
4. **Evaluate each option**: Analyze against the criteria
5. **Form conclusion**: Which option is best and why?
6. **Present**: State your conclusion, then seek validation if needed

**DO NOT skip this process and ask without analysis.**

### Anti-Patterns (What NOT to Do)

❌ **Approval Dependency**:
- Repeatedly asking "Does this plan work for you?"
- Always seeking confirmation instead of making judgments
- Waiting for user approval for technical decisions

❌ **Thought Abandonment**:
- Adding user requests to plans without questioning why
- Not considering if the request is actually needed
- Accepting requirements at face value

❌ **Superficial Reasoning**:
- Using vague justifications like "because it's important" or "it's needed"
- Not providing concrete evidence or analysis
- Copying patterns without understanding fundamentals

### Critical Analysis Required

- Question assumptions, including those in documentation
- Consider multiple perspectives before concluding
- Verify your understanding against first principles
- Have conviction - defend your reasoning with evidence
- **Why**: Thoughtless application of patterns leads to poor solutions

## Working Style

### Incremental Delivery

- Break work into visible checkpoints
- Show intermediate results whenever possible
- Allow for course correction at each stage
- Make it easy to pause and resume work

## Communication Protocol

> [!IMPORTANT]
>
> All communication with the user must be concise and focused. Avoid overwhelming the user with long messages.

### Concise Communication

- **20-30 lines maximum** for messages to the user
  - **Excludes**: Code blocks, tool outputs, file contents, diffs
  - **Rationale**: User should grasp your message without scrolling
  - Messages where 90% is code with 2 lines of explanation are fine
- **Split long content**: Use "Part 1/3", "Part 2/3" format when needed
- **Wait for acknowledgment** before continuing to next part

### When Seeking User Confirmation

When you need user input, decisions, or validation:

**One Question at a Time**

- Present only one question or decision point per message
- Do not dump multiple questions at once
- Wait for user response before presenting the next question

**Progressive Disclosure**

If you have multiple questions:

1. Ask the first question
2. Wait for response
3. Then ask the second question
4. Continue until all questions are answered

**Show Progress Indicators**

- Always indicate total number and current position when asking multiple questions
- Format: "Question 1 of 3:", "Question 2 of 3:", etc.
- User needs to know how many more questions are coming
- **Why**: Prevents frustration from unknown question count

**Question Format**

- Keep each question concise (see Concise Communication above)
- Include necessary context, options, and implications
- Make the decision point clear and actionable

### Examples

**GOOD** - Single Focused Question:

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

**BAD** - Multiple Questions at Once:

```
❌ いくつか質問があります：
1. Paymentは Order集約の一部にすべきですか？
2. 返金処理はどう扱いますか？
3. リポジトリパターンはどうしますか？
4. ドメインイベントを使いますか？
5. 整合性モデルはどれが良いですか？
```

All questions dumped at once - overwhelming

**BAD** - Too Long:

```
集約設計についての理論全体、すべての可能なパターン、各アプローチの詳細な説明、歴史的背景、他のシステムからの10個の異なる例、それぞれの詳細な長所と短所、そしてこの1つの質問の中に5つのネストされたサブ質問があります...

❌ Here's everything about aggregate design including all the theory, all possible patterns, detailed explanations of each approach, historical context, 10 different examples from other systems, pros and cons of each with extensive detail, and here are 5 nested sub-questions within this one question...
```

50+ lines - requires scrolling to understand

### Automatic Execution is Fine

You can execute multiple steps automatically **without confirmation** as long as no user decisions are needed. Only
pause and ask when user input is actually required.

**Decision Boundary: When to Ask vs Auto-Execute**

**Ask when**:
- **Business logic decisions**: Domain model design, aggregate boundaries, consistency models
- **User preference matters**: Technology stack choices, architectural patterns with trade-offs
- **Multiple valid approaches exist**: No clear "best practice" applies

**Auto-execute when**:
- **Technical implementation**: Given a design, implement it without asking
- **Clear best practice exists**: Apply known patterns (e.g., dependency injection, error handling)
- **Reversible actions**: Changes that can be undone (Git-tracked files)

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

## Available Practice and Context Files

You might get a ton of know-how from GitHub, and your use of individual algorithms and libraries is gonna be faster
than my implementation. But YOU MUST follow my practices.

### General Project Information

- [README.md](../../README.md) - Project overview, features, and setup introduction
- [CONTRIBUTING.md](../../CONTRIBUTING.md) - Detailed contribution guidelines, including project structure, development
  environment setup, and workflow

### Universal Principles (Language-agnostic)

- [**design-practices.md**](./design-practices.md) - Core design principles (DDD, boundaries, dependencies)
- [**testing-practices.md**](./testing-practices.md) - Testing principles across all levels
- [**coding-practices.md**](./coding-practices.md) - Cross-language implementation guidance
- [**git-practices.md**](./git-practices.md) - Git workflow and commit message conventions

### Language-Specific Practices

- [**kotlin-practices.md**](./kotlin-practices.md) - Kotlin idioms and conventions
- [**rust-practices.md**](./rust-practices.md) - Rust patterns and best practices
- [**typescript-practices.md**](./typescript-practices.md) - TypeScript guidelines

### Language-Specific Examples (Created when needed)

- `kotlin-examples.md` or files in `kotlin-examples`
- `rust-examples.md` or files in `rust-examples`
- `typescript-examples.md` or files in `typescript-examples`

## Domain-Specific Context

Each bounded context has its own `.ai/context.md`:

- `src/user-context/.ai/context.md`
- `src/order-context/.ai/context.md`
- `src/shared-kernel/.ai/context.md`

Load the relevant domain context when working within that context.

## Loading Strategy

Load only what's needed for your current task to optimize token usage.

### Loading Matrix

| Task                    | design | testing | coding | {lang} | domain |
|-------------------------|--------|---------|--------|--------|--------|
| New Feature (Design)    | ✓      | ✓       | -      | -      | ✓      |
| New Feature (Implement) | ✓      | ✓       | ✓      | ✓      | ✓      |
| Write Tests             | -      | ✓       | -      | ✓      | ✓      |
| Refactor                | ✓      | -       | ✓      | ✓      | -      |
| Code Review             | ✓      | ✓       | -      | ✓      | ✓      |
| Bug Fix                 | -      | -       | -      | ✓      | ✓      |
| Domain Modeling         | ✓      | ✓       | -      | -      | ✓      |
| Infrastructure          | -      | -       | ✓      | ✓      | -      |

**Legend:**

- design = [design-practices.md](./design-practices.md)
- testing = [testing-practices.md](./testing-practices.md)
- coding = [coding-practices.md](./coding-practices.md)
- {lang} = [kotlin](./kotlin-practices.md), [rust](./rust-practices.md), or [typescript](./typescript-practices.md)
- domain = src/{context}/.ai/context.md
