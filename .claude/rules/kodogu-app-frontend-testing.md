---
paths:
  - "app/src/**/*.test.ts"
  - "app/src/**/*.test.tsx"
  - "app/src/**/*.spec.ts"
  - "app/src/**/*.spec.tsx"
  - "app/src/**/*.story.tsx"
---

# KoDoGu App Frontend Testing Practices

Test-strategy conventions for the Tauri app frontend (`app/src/`) — which test level owns which kind of behavior,
and the concrete patterns for each. Complements `kodogu-app-frontend.md` (general frontend conventions) and
`testing.md` (language-agnostic testing principles).

## Four Vitest Projects

| Project | Environment | File pattern | `__DEV_TOOLS__` | Purpose |
|---|---|---|---|---|
| unit | node | `*.test.ts` | false | Pure functions, error invariants |
| browser | browser (Playwright) | `*.spec.tsx` (excl. `*.devtools.spec.tsx`) | false | Hooks, build-flag=false guards |
| browser-devtools | browser (Playwright) | `*.devtools.spec.tsx` | true | Dev-tools UI presence |
| storybook | browser (storybookTest) | `*.story.tsx` (`play`) | false | Component behavior |

Configured in `vite.config.ts`'s `test.projects`. Plus: `cargo test` for Rust, manual checklist for tray-level UX.

## Choosing Between spec and story

Per Storybook's own position — "Unit tests validate logic; E2E tests are best for a few key flows. **Storybook Test
covers the middle: your components.**" — do not write both a `.spec.tsx` and a `.story.tsx` for the same subject.
Pick one based on what's under test:

| Subject | File | Why |
|---|---|---|
| Pure function (no DOM) | `*.test.ts` | Runs in the `unit` (node) project |
| Hook in isolation | `*.spec.tsx` | No component is the SUT — forcing it into a story distorts the story |
| Component behavior | `*.story.tsx` (`play`) | Storybook's home ground |
| `__DEV_TOOLS__` build-flag behavior | `*.spec.tsx` / `*.devtools.spec.tsx` | One Storybook run = one build-time value; can't toggle per-story |

- `play` functions verify more than visuals — side effects too (`vi.mock`, `expect(mockInvoke).toHaveBeenCalledWith(...)`, etc.). Don't treat Storybook tests as screenshot-only.

## Interaction: `userEvent`, Not `element.click()`

In a `.story.tsx` `play` function, click through `await userEvent.click(el)` (`storybook/test`), not `element.click()`.

- `userEvent.click` replays a realistic pointer/focus event sequence and awaits React's `act` flush, so the
  assertion after it sees settled DOM.
- `element.click()` fires a single synchronous `click` event — a low-fidelity shortcut, and type-aware ESLint flags
  `await element.click()` as `await-thenable` since it isn't a Promise.
- `expect(...)` from `storybook/test` returns a Promise — always `await` it (`no-floating-promises`).
- In `.spec.tsx`, vitest-browser's `page.locator.click()` is a different, already-async API — `await` it directly.
- Biome has no type-aware rules, so it won't catch a missed `await` here — always verify with both `biome ci .` and
  `eslint .`.

## Why vitest-browser-react, Not jsdom/happy-dom

See [ADR-0003](../../docs/adr/0003-use-vitest-browser-react-for-hook-testing.md).

- Runs against real browser APIs (`ResizeObserver`, `IntersectionObserver`, etc.) instead of DOM-shaped mocks
- `renderHook` is async — `await renderHook()`
- Shares the same Playwright infrastructure as the `browser`/`browser-devtools`/`storybook` projects

## Testability: Extract Pure Logic

Prefer testing pure functions extracted from hooks (`createResizeHandler`, `getMaxHeight`, etc.) over testing the
hook itself — same principle applies on the Rust side (`calculate_position`, etc.). Only fall back to a browser
`.spec.tsx` for the hook/IPC integration that can't be extracted.
