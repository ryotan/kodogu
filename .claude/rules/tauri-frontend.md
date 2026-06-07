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
  routes/                        # TanStack Router (file-based routing, auto-generated tree)
  features/
    <cohesive-concern-name>/     # One directory per cohesive concern
      domain/                    # Entities, Value Objects (no external deps)
      usecases/                  # Application layer: Use Cases + Input Port types
                                 # (includes Form type definitions and validation rules)
      adapters/                  # Interface Adapters (Clean Architecture's adapters layer)
      components/                # UI components
  shared/
    core/                        # Foundation code referenced by multiple shared/* modules
      errors/
      messages/
      config/
      utils/
    commands/                    # Tauri IPC surface (backend-determined, like an OpenAPI client)
    notifications/
    logging/
    validation/
    date/
  ui/                            # Design system (Radix-based visual primitives)
  assets/
  (prototype)/                   # Prototype components (not shipped)
  (dev)/                         # Development-only utilities (excluded from production)
```

### Dependency Direction

```
routes/      → features/, ui/
components/  → usecases/, adapters/, domain/, ui/
usecases/    → domain/
adapters/    → usecases/, domain/
shared/xxx   → shared/core/
ui/          → (none)
```

`shared/*` may be referenced from any layer (it intentionally sits outside the layered dependency graph). Cross-references within `shared/` are allowed; code referenced by multiple `shared/*` modules is a candidate for `shared/core/` — move when it feels like "core", not as a strict rule.

This table mirrors `docs/adr/0006-frontend-application-directory-structure.md`. If the two ever drift, the ADR is canonical.

### adapters/ — Interface Adapters Layer

`adapters/` is Clean Architecture's **Interface Adapters** layer. It bridges both directions:

- **UI → UseCase**: `useForm()` wiring, controller hooks
- **UseCase → external**: Tauri IPC, REST, GraphQL

Do not split into stereotype subdirectories (`queries/`, `translators/`, `hooks/`).
One file = one connection target. Data translation, query definitions, and hooks live together in one file.

```typescript
// adapters/connections.ts
const connectionsQuery = queryOptions({
  queryFn: async () => {
    const raw = await getConnectionsCommand()
    return raw.map(toConnectionProfile)     // translation included
  },
})

export const useConnections = () => useQuery(connectionsQuery)   // hook included
```

### Form Responsibility

Form type definitions and validation rules belong in `usecases/` as the UseCase's Input Port.
`useForm()` wiring (connecting to React state) belongs in `adapters/`.

```typescript
// usecases/createConnection.ts — Input Port (what is valid)
export const CreateConnectionInput = type({
  name: "string > 0",
  port: "1 <= number.integer <= 65535",
})

// adapters/createConnection.ts — Interface Adapter (how to wire UseCase to React)
export const useCreateConnectionForm = () =>
  useForm({
    validators: {onChange: CreateConnectionInput},
    onSubmit: ({value}) => createConnectionUseCase(value),
  })
```

### Tauri IPC

- Tauri command wrappers go in `shared/commands/`
- The backend determines the API surface (potentially auto-generated in the future)
- Treat `shared/commands/` like an OpenAPI-generated client — do not scatter across features
- Each wrapper provides a typed function around `invoke()`
- Domain logic stays in Rust. The TS side only handles invocation and result mapping

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
| Story | target + `.story` | `ProfileCard.story.tsx` |

### Exports

| Target | Convention | Example |
|---|---|---|
| React component | PascalCase, named export | `export const ProfileCard = (...) => ...` |
| Hook | camelCase `use` prefix, named export | `export const useAutoResize = () => ...` |
| Pure function | camelCase, named export | `export const getMaxHeight = () => ...` |
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

> **Note**: Stereotypes will be extended as needed (Form, Service, etc.).

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

Guard development-only imports and components with `__DEV_TOOLS__` (declared in `app/src/vite-env.d.ts`,
defined per-environment in `app/vite.config.ts` and `.storybook/main.ts`):

```typescript
// Lazy-load dev tools only when __DEV_TOOLS__ is enabled
const TanStackTools = __DEV_TOOLS__
  ? lazy(() => import('@tanstack/router-devtools').then(/* ... */))
  : () => null;
```

- Vite replaces `__DEV_TOOLS__` literally at build time → dead-code elimination removes dev-only code from non-dev bundles (prod build, Storybook, browser test project)
- The `browser-devtools` Vitest project enables it explicitly to verify devtools-on behaviour (`*.devtools.spec.tsx`)
- Do NOT use `import.meta.env.DEV` for this guard — `DEV` is true in Storybook and tests too, which would make stories noisy and tie "render dev tools" to "Vite is in dev mode"
- Do NOT use runtime checks like `process.env.NODE_ENV` — Vite does not replace them in browser builds
- **Why**: One flag controls "should dev tools render"; each environment (dev server, prod, Storybook, devtools spec) sets it independently

## Error Boundaries

### PopoverErrorBoundary

Wrap `RouterProvider` (or top-level async UI) with `PopoverErrorBoundary`:

```tsx
<PopoverErrorBoundary>
  <RouterProvider router={router} />
</PopoverErrorBoundary>
```

- `QueryErrorResetBoundary` resets failed queries on retry so the user gets a fresh fetch
- Use `fallbackRender` (not `FallbackComponent`) for inline error normalization
- **Why**: Prevents stale error state after retry

### Error Component Design

- `ErrorDisplay` accepts only `error: Error` — no retry button. Display and action are separated.
- `PopoverAppErrorPage` internally calls `useQueryErrorResetBoundary` and owns the Try Again button.
- `PopoverAppErrorPage` is compatible with both `defaultErrorComponent` (TanStack Router) and `fallbackRender` in `PopoverErrorBoundary`.
- **Why**: Reusable across TanStack Router's `defaultErrorComponent` and React ErrorBoundary

## State Management

- React hooks (`useState`, `useReducer`) for component-level state
- TanStack Store for application-wide state
- `immer` for complex immutable state updates
- **Why**: Keeps state management predictable and type-safe
