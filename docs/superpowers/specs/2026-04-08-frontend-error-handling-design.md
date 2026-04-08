# Frontend Error Handling Design

**Date:** 2026-04-08
**Branch:** `feature/kdg-26-frontend-directory-structure-and-error-handling`
**Related:** KDG-26 Phase 2, ADR-0006

---

## Overview

Phase 2 of KDG-26: define the frontend error handling strategy. This covers error type
hierarchy, typed aggregate errors, refactoring of existing error display components, and
a minor adjustment to ADR-0006.

---

## Error Type Hierarchy

Defined in `shared/core/errors/`. Follows the `coding-practices.md` classification.

```
RootError extends Error
├─ ApplicationError
│  ├─ DomainError
│  ├─ UseCaseError
│  ├─ AdapterError
│  └─ FrameworkError
└─ PlatformError
```

One file per type. Each file exports the class and its type guard.

```
shared/core/errors/
  RootError.ts              — RootError + isRootError
  ApplicationError.ts       — ApplicationError + isApplicationError
  DomainError.ts            — DomainError + isDomainError
  UseCaseError.ts           — UseCaseError + isUseCaseError
  AdapterError.ts           — AdapterError + isAdapterError
  FrameworkError.ts         — FrameworkError + isFrameworkError
  PlatformError.ts          — PlatformError + isPlatformError
  TypedAggregateError.ts    — TypedAggregateError + isTypedAggregateError
  stackTrace.ts             — StackFrame type, parseFrame, formatOriginLocation
```

### Error Chaining: `cause`

All errors support single-cause chaining via native `ErrorOptions`:

```ts
throw new DomainError('Save failed', { cause: originalError });
```

`Error.cause` is ES2022 standard and supported across all target environments.

### Aggregate Errors: `cause: TypedAggregateError`

Multiple errors are expressed by placing a `TypedAggregateError` in `cause`. This mirrors
Python's `ExceptionGroup` pattern — the aggregate is itself an `Error` and captures its own
stack trace at the point of collection.

`cause` and `TypedAggregateError` are mutually exclusive as sub-error mechanisms:
a given `cause` is either a single error (chain) or a `TypedAggregateError` (aggregate).

```ts
// TypedAggregateError narrows the `any[]` of the standard AggregateError
class TypedAggregateError<E extends Error = Error> extends AggregateError {
  declare readonly errors: ReadonlyArray<E>;
}

// Usage
throw new DomainError('Validation failed', {
  cause: new TypedAggregateError<DomainError>([fieldError1, fieldError2]),
});
```

**Why `cause` rather than an `errors` property on the error class:**
- Semantic: "this error's cause is these multiple errors" — natural use of `cause`
- Stack trace: `TypedAggregateError` is an `Error` and captures a stack at the point of
  collection, giving three layers of context: where thrown, where collected, where each
  sub-error occurred
- Precedent: Python's `ExceptionGroup` can appear as `__cause__` of another exception

### Error Traversal

Display and logging code traverses the full tree by checking `cause`:

```ts
function traverse(e: Error) {
  // ... render e ...
  if (e.cause instanceof AggregateError) {
    for (const sub of e.cause.errors) traverse(sub); // each sub may have its own cause
  } else if (e.cause instanceof Error) {
    traverse(e.cause);
  }
}
```

### Notification Strategy

The caller decides how to surface each error. There is no centralized error-type →
notification-method mapping. Type guards (`isApplicationError`, etc.) give callers the
information needed to make that decision.

---

## Component Refactoring

### Problem

`AppErrorComponent` and `AppErrorPage` mix four distinct concerns:
1. Stack trace parsing — pure functions
2. Error display UI — design system component
3. TanStack Query reset integration — framework adapter
4. Tauri popover layout / window resize — platform-specific

### After Refactoring

#### `shared/core/errors/stackTrace.ts`

Move `StackFrame`, `parseFrame`, `formatOriginLocation` from `AppErrorComponent` here.
Pure functions with no dependencies.

#### `ui/ErrorDisplay.tsx`

New pure visual component. Accepts `error: Error` and `onReset?: () => void`. Renders
error name, message, origin frame, stack trace (expandable), and reset button. Uses
Radix UI only — no TanStack Query, no Tauri dependencies.

#### `features/popover/components/PopoverLayout.tsx`

Layout wrapper for the popover window: full-height flex container with overflow scroll.
Accepts `children` and `contentRef?: RefCallback<HTMLDivElement>`.

Replaces the duplicated inline `style` in `AppErrorPage` and the identical structure in
`PopoverPendingLayout`. `PopoverPendingLayout` is also moved here:

```
features/popover/
  components/
    PopoverLayout.tsx
    PopoverPendingLayout.tsx   ← moved from app/src/components/
```

#### `main.tsx` — local components

`PopoverAppErrorPage` and `PopoverErrorBoundary` are defined as module-local components
in `main.tsx`. They are used only there (as `defaultErrorComponent` and `ErrorBoundary`
fallback), so a separate file adds no value.

```tsx
// PopoverAppErrorPage: wires useAutoResize + useQueryErrorResetBoundary + ErrorDisplay
const PopoverAppErrorPage = ({ error, reset }: ErrorComponentProps) => {
  const { reset: resetQueries } = useQueryErrorResetBoundary();
  const { resizeCallbackRef } = useAutoResize();
  return (
    <PopoverLayout contentRef={resizeCallbackRef}>
      <ErrorDisplay error={error} onReset={() => { resetQueries(); reset(); }} />
    </PopoverLayout>
  );
};

// PopoverErrorBoundary: QueryErrorResetBoundary + ErrorBoundary for the popover window
// useQueryErrorResetBoundary hook (used inside PopoverAppErrorPage) handles query reset,
// so QueryErrorResetBoundary does not need the render-prop form.
const PopoverErrorBoundary = ({ children }: { children: ReactNode }) => (
  <QueryErrorResetBoundary>
    <ErrorBoundary
      fallbackRender={({ error, resetErrorBoundary }) => (
        <PopoverAppErrorPage
          error={error instanceof Error ? error : new Error(String(error))}
          reset={resetErrorBoundary}
        />
      )}
    >
      {children}
    </ErrorBoundary>
  </QueryErrorResetBoundary>
);
```

#### Deleted files

- `app/src/components/AppErrorComponent.tsx`
- `app/src/components/AppErrorComponent.module.css`
- `app/src/components/AppErrorComponent.stories.tsx`
- `app/src/components/AppErrorPage.tsx`
- `app/src/components/AppErrorPage.stories.tsx`
- `app/src/components/stackTrace.ts` (moved to `shared/core/errors/stackTrace.ts`)

---

## ADR-0006 Adjustment

The description of `features/` is broadened from "domain boundaries" to "cohesive,
isolated concerns." Not all features require all Clean Architecture layers.

**Change:**
```diff
- <bounded-context-name>/       # ドメイン境界ごとに分割
+ <bounded-context-name>/       # 凝集した独立した関心事ごとに分割
+                               # 全レイヤーが必須ではない（UIシェル等はcomponents/のみでよい）
```

**Example added to ADR-0006:**
```
features/
  popover/              # ポップオーバーウィンドウシェル（UIシェル — components/ のみ）
    components/
  tsunagi/              # ドメイン機能（全レイヤーを持つ）
    domain/
    usecases/
    adapters/
    components/
```

---

## Out of Scope

- **KDG-27**: Tauri IPC error mapping (converting raw IPC errors into typed `RootError`
  subclasses). `PlatformError` subclasses (e.g., `IpcError`) are deferred to that issue.
- **Notification UI**: Toast, inline error components. Added per-feature as needed.
- **Error mapping helpers**: Error type → notification method mapping is context-specific
  and left to each feature's UI layer.
- **Full migration of `components/` to new structure**: Other files in `components/` are
  out of scope for this issue.
