---
paths:
  - "app/src/**/*.ts"
  - "app/src/**/*.tsx"
---

# Tauri App Frontend Practices

Framework-specific practices for the Tauri desktop app frontend (`app/src/`).

## Application Structure

### Directory Layout

```
app/src/
  routes/          # TanStack Router file-based routes (auto-generated tree)
  hooks/           # Custom React hooks (reusable across routes)
  components/      # Shared UI components
  commands/        # Tauri command wrappers (typed invoke() functions)
  (prototype)/     # Prototype / stub components (temporary, not shipped)
  (dev)/           # Development-only utilities (excluded from production)
```

- Routes define screens. Each route file is a self-contained entry point
- Hooks with independent contracts go in `hooks/`. Route-local hooks stay in the route file
- Components shared across 2+ routes go in `components/`. Single-route components stay in the route file or a colocated file

### Tauri IPC

- Tauri command wrappers go in `commands/`
- Each wrapper provides a typed function around `invoke()`
- Domain logic stays in Rust. The TS side only handles invocation and result mapping

> **Note**: This structure is minimal for M1. Consider introducing Features / Bases layers in M1.5 / M2.
> Ref: [Fintan Santoku Application Structure](https://fintan-contents.github.io/mobile-app-crib-notes/react-native/santoku/application-architecture/application-structure/overview/)

### Dependency Direction

- `routes/` → `hooks/`, `components/`, `commands/`
- `hooks/` → `commands/` (when calling Tauri APIs)
- `components/` → no dependencies on other directories (receive data via props)
- `(prototype)/` → never imported by others (only referenced by routes)

## File Organization

### 1 File = 1 Export

Each file represents a single logical module with one primary export.

- File name = primary export name (`ProfileCard.tsx` → `export const ProfileCard`)
- Closely related constants/types may co-exist in the same file (e.g., a hook and its constants)
- Do not mix unrelated exports in one file

Ref: [Fintan Santoku TypeScript Style Guide](https://fintan-contents.github.io/mobile-app-crib-notes/react-native/santoku/development/implement/style-guide/typescript-style-guide/)

## Naming Conventions

### Files

**File name = primary export name** as a rule.

| Target | Convention | Example |
|---|---|---|
| Directory | kebab-case | `hooks/`, `components/form/` |
| Route file | Follow TanStack Router convention | `popover.tsx`, `__root.tsx` |
| Component | PascalCase (= export name) | `ProfileCard.tsx` |
| Hook | camelCase `use` prefix (= export name) | `useAutoResize.ts` |
| Function | camelCase (= export name) | `createResizeHandler.ts` |
| Type-only | PascalCase (= export name) | `ConnectionProfile.ts` |
| Tauri command wrapper | camelCase (= export name) | `getConnections.ts` |
| Test (node) | target + `.test` | `useAutoResize.test.ts` |
| Test (browser) | target + `.spec` | `useAutoResize.spec.tsx` |
| Story | target + `.stories` | `ProfileCard.stories.tsx` |

### Exports

| Target | Convention | Example |
|---|---|---|
| React component | PascalCase, named export | `export const ProfileCard = ...` |
| Hook | camelCase `use` prefix, named export | `export function useAutoResize()` |
| Pure function | camelCase, named export | `export function getMaxHeight()` |
| Type / Interface | PascalCase, named export | `export interface ConnectionProfile` |
| Constant | UPPER_SNAKE_CASE | `const MAX_HEIGHT_RATIO = 0.85` |
| Module-local constant | camelCase or UPPER_SNAKE_CASE | context-dependent |

### Stereotype Suffixes

Use explicit suffixes to indicate responsibility. The file's role should be clear without reading its code.

| Stereotype | Suffix | Example |
|---|---|---|
| Hook | `use` prefix | `useAutoResize` |
| Function | none (starts with verb) | `createResizeHandler`, `getMaxHeight` |
| Context Provider | `Provider` | `ThemeProvider` |
| Error class | `Error` | `ConnectionError` |
| Layout component | `Layout` | `PopoverLayout` |

> **Note**: Stereotypes will be extended in M2+ as needed (Form, Service, etc.).

## TanStack Query

### `queryOptions()` Factory Pattern

- Use `queryOptions()` to co-locate `queryKey`, `queryFn`, and options in a single factory
- Do NOT scatter queryKey / queryFn / staleTime across separate files
- Set `staleTime` / `gcTime` per-query, not as global defaults
- Consumers use the factory via `buildXxxQuery({select})` or spread `{...xxxQuery, select}`
- `QueryKeys` object pattern is deprecated — `queryOptions()` owns the key

```typescript
export const buildAppConfigQuery = <T = AppConfig>(opts?: AppConfigQueryOptions<T>) =>
  queryOptions({
    queryKey: ['appConfig'],
    queryFn: getConfig,
    staleTime: Number.POSITIVE_INFINITY,
    gcTime: Number.POSITIVE_INFINITY,
    ...opts,
  });
export const appConfigQuery = buildAppConfigQuery();
```

- **Why**: Type-safe `getQueryData()` inference via DataTag, single source of truth

### Optimistic Update Checklist

Mutations with optimistic updates must follow this 3-step lifecycle:

1. `onMutate`: Cancel queries → save previous → set optimistic data → return `{previous}`
2. `onError`: Log error + rollback with `context?.previous`
3. `onSettled`: Invalidate queries (covers both success and error)

Additional requirements:
- Set `scope: {id: 'xxx'}` to serialize parallel mutations on the same resource
- Always log errors in `onError` — never silently swallow

### Tauri Desktop App Defaults

Set these on the global `QueryClient` — browser-centric defaults don't apply:

```typescript
const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      refetchOnWindowFocus: false, // Tauri popover show/hide ≠ browser focus
      refetchOnReconnect: false,   // Local IPC, no network
    },
  },
});
```

- **Why**: Tauri IPC is local. Browser online/offline and focus events are irrelevant

## TanStack Router

- File-based routing: route files in `routes/` are auto-discovered
- Use `createFileRoute` for type-safe route definitions
- Route-level code splitting via lazy routes

## Radix UI

- Radix Themes as the design system
- Import from `@radix-ui/themes`
- Use theme tokens for colors, spacing, typography
- **Why**: Consistent, accessible component library

## Vitest

- Vitest for unit and component testing
- Fast test execution with Vite integration
- Compatible with Jest API
- vitest-browser-react for hook/component tests (no jsdom/happy-dom)
- **Why**: Modern testing with great TypeScript support

## WebKit CSS

Tauri on macOS uses WKWebView (Safari engine). Always add `-webkit-` prefixed versions for:

- `backdrop-filter` → `-webkit-backdrop-filter`
- `mask-image` → `-webkit-mask-image`
- Other properties listed in [Can I Use](https://caniuse.com) as requiring `-webkit-`

```css
.glass {
  backdrop-filter: blur(24px);
  -webkit-backdrop-filter: blur(24px); /* Required for WKWebView */
}
```

- **Why**: WKWebView does not support unprefixed versions of some CSS properties

## Dev-Only Code

Guard development-only imports and components with `import.meta.env.DEV`:

```typescript
// Lazy-load dev tools only in development builds
const TanStackTools = import.meta.env.DEV
  ? lazy(() => import('@tanstack/router-devtools').then(/* ... */))
  : () => null;
```

- Vite statically replaces `import.meta.env.DEV` → dead-code elimination removes dev-only code from production
- Do NOT use runtime checks like `process.env.NODE_ENV` — Vite does not replace them in browser builds
- **Why**: Zero dev-tool overhead in production

## Error Boundaries

### QueryErrorResetBoundary + ErrorBoundary

Wrap `RouterProvider` (or top-level async UI) with both:

```typescript
<QueryErrorResetBoundary>
  {({reset}) => (
    <ErrorBoundary onReset={reset} fallbackRender={({error, resetErrorBoundary}) => (
      <AppErrorPage error={error} reset={resetErrorBoundary} />
    )}>
      <RouterProvider router={router} />
    </ErrorBoundary>
  )}
</QueryErrorResetBoundary>
```

- `QueryErrorResetBoundary` resets failed queries on retry so the user gets a fresh fetch
- Use `fallbackRender` (not `FallbackComponent`) for inline error normalization
- **Why**: Prevents stale error state after retry

### Error Component Design

- Accept `error: Error` and optional `reset?: () => void` props
- Use `useMemo` for expensive derivations (e.g., stack trace parsing)
- Normalize unknown errors at the boundary, not inside the component
- **Why**: Reusable across TanStack Router's `defaultErrorComponent` and React ErrorBoundary

## State Management

- React hooks (`useState`, `useReducer`) for component-level state
- TanStack Store for application-wide state
- `immer` for complex immutable state updates
- **Why**: Keeps state management predictable and type-safe
