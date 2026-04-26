---
paths:
  - "**/*.ts"
  - "**/*.tsx"
---

# TypeScript Practices

TypeScript-specific practices for realizing universal principles in this language.

## Type System

### Enable Strict Mode

- Use `"strict": true` in tsconfig.json
- Enables all strict type checks
- Catches errors at compile time
- **Why**: Maximum type safety

### Use Explicit Types for Public APIs

- Type function parameters and return values
- Type class properties
- Let inference work for local variables
- **Why**: Clear contracts, better tooling

### Prefer Interfaces for Objects

- `interface` for object shapes
- `type` for unions, intersections, primitives
- Extend interfaces, don't use intersections
- **Why**: Better error messages, more extensible

## Null and Undefined

### Handle Null and Undefined Explicitly

- Enable `strictNullChecks`
- Use optional chaining `?.` for safe access
- Use nullish coalescing `??` for defaults
- **Why**: Prevents runtime null/undefined errors

### Prefer Undefined Over Null

- Use `undefined` for "not present"
- Reserve `null` for explicit "no value"
- Be consistent across codebase
- **Why**: Simpler mental model

### Avoid Non-Null Assertions

- Don't use `!` except when absolutely certain
- Prefer type guards or optional chaining
- Comment why when using `!`
- **Why**: Defeats type safety

## Union Types

### Use Discriminated Unions

- Add literal type field for discrimination
- Exhaustive checking with `never`
- Type-safe state machines
- **Why**: Compiler ensures all cases handled

### Prefer Union Types Over Enums

- Union of string literals over string enums
- Type safety without runtime overhead
- Easier to refactor
- **Why**: More type-safe, simpler

## Type Guards

### Use Type Guards for Narrowing

- `typeof` for primitives
- `instanceof` for classes
- User-defined type guards with `is`
- **Why**: Type-safe runtime checks

### Create Custom Type Guards

- `function isUser(x: unknown): x is User`
- Encapsulate complex checks
- Reusable type narrowing logic
- **Why**: DRY principle for type checking

## Generics

### Use Generics for Reusable Code

- Generic functions and classes
- Constraint with `extends`
- Default type parameters
- **Why**: Type-safe reusability

### Avoid Over-Generic Code

- Too many type parameters is a code smell
- Consider splitting into specific functions
- Use generics when truly reusable
- **Why**: Simpler is better

## Readonly and Immutability

### Use Readonly for Immutable Data

- `readonly` for properties that don't change
- `Readonly<T>` for entire objects
- `ReadonlyArray<T>` for arrays
- **Why**: Prevents accidental mutations

### Prefer Const Assertions

- `as const` for literal types
- Deeply readonly objects
- Narrower types
- **Why**: Maximum type safety for constants

## Async/Await

### Always Use Async/Await

- Prefer async/await over `.then()`
- Handle errors with try/catch
- Return promises from async functions
- **Why**: More readable, easier error handling

### Handle Promise Rejection

- Always catch errors in async functions
- Use `Promise.allSettled()` for parallel operations
- Don't ignore unhandled rejection warnings
- **Why**: Prevents silent failures

## Error Handling

For error classification philosophy, invoke the `coding-practices` skill.

### Implement Error Hierarchy with Custom Error Classes

- Extend `Error` for Application and Platform errors (domain, use case, infrastructure)
- Create error hierarchy: `RootError` → `ApplicationError` / `PlatformError` → specific errors
- Pass descriptive messages to `super()` constructor
- Include relevant context as readonly properties
- Use centralized error handling (Express middleware, etc.)
- **Why**: Type-safe error classification, consistent error responses

### Handle Programming Errors with Type System and Assertions

- **Prevent at compile time**: `strictNullChecks`, `noUncheckedIndexedAccess`, `never` for exhaustiveness
- **Runtime assertions**: Use `assert()` or throw for "should never happen" conditions
- **Don't catch Programming Errors**: Let them crash and fix the bug
- **Why**: Programming errors indicate bugs, should fail fast

## Functions

### Use Arrow Functions Appropriately

- Arrow functions for callbacks and inline functions
- Regular functions for methods that need `this`
- Consistent `this` binding
- **Why**: Avoid `this` confusion

### Prefer Function Overloads

- Multiple signatures for different argument types
- Type-safe variant handling
- Better than union types for parameters
- **Why**: Precise type inference

## Classes

### Use Access Modifiers

- `private` for internal implementation
- `protected` for inheritance
- `public` as default
- **Why**: Encapsulation

### Prefer Composition Over Inheritance

- Use interfaces for contracts
- Implement multiple interfaces
- Delegate instead of extend
- **Why**: More flexible, easier to test

### Use Parameter Properties

- `constructor(private readonly id: string)`
- Declares and initializes in one line
- Less boilerplate
- **Why**: Concise class definitions

## Modules

### Use ES Modules

- `import`/`export` not `require`
- Named exports for multiple items
- Default export for primary export
- **Why**: Standard, tree-shakeable

### Avoid Barrel Exports

- Don't re-export everything in index.ts
- Explicit imports are clearer
- Better for tree-shaking
- **Why**: Performance and clarity

## Utility Types

### Leverage Built-in Utility Types

- `Partial<T>`, `Required<T>`, `Pick<T>`, `Omit<T>`
- `Record<K, V>` for objects
- `ReturnType<T>`, `Parameters<T>` for function types
- **Why**: Standard, well-understood

### Create Custom Mapped Types

- Transform object types systematically
- Use template literal types
- Conditional types for complex logic
- **Why**: Type-level programming

## Type Assertions

### Avoid Type Assertions

- `as` assertions bypass type checking
- Use type guards instead
- Only when you know more than compiler
- **Why**: Maintains type safety

### Use Type Assertions Sparingly

- DOM element types: `as HTMLInputElement`
- Unknown to specific type after validation
- Document why assertion is safe
- **Why**: Explicit escape hatch

## Declaration Files

### Create .d.ts for Untyped Libraries

- Declare types for JavaScript libraries
- Use `@types/` packages when available
- Minimal types better than `any`
- **Why**: Type safety for entire codebase

### Use Ambient Declarations

- `declare` for global types
- Keep declarations minimal
- Organize in `types/` directory
- **Why**: Clean type definitions

## Testing

### Type Test Data Properly

- Use `as const` for test fixtures
- Type test doubles
- Use `jest.MockedFunction<typeof fn>`
- **Why**: Tests benefit from types too

### Test Types with tsd

- Use `tsd` for type-level testing
- Verify complex type transformations
- Catch type regressions
- **Why**: Types are part of API

## Configuration

### Configure TypeScript Strictly

```json
{
  "compilerOptions": {
    "strict": true,
    "noUncheckedIndexedAccess": true,
    "noImplicitOverride": true,
    "exactOptionalPropertyTypes": true
  }
}
```

- **Why**: Catch more errors at compile time

### Use Path Mapping

- `@/` for src root
- Avoid `../../../` imports
- Configure in tsconfig.json
- **Why**: Cleaner imports

## Formatting and Linting

### Use Prettier for Formatting

- Autoformat on save
- No debates about style
- Consistent across team
- **Why**: Focus on code, not formatting

### Use ESLint with TypeScript

- `@typescript-eslint` plugin
- Catches common mistakes
- Enforces best practices
- **Why**: Better code quality

## React-Specific (if applicable)

### Type React Components Properly

- Use plain function components with explicit props types
- Avoid `React.FC` (discouraged pattern; use plain function components instead)
- Type props interfaces explicitly
- Type state with `useState<T>`
- Explicitly type `children` when needed: `children: React.ReactNode`
- **Why**: Type-safe components without implicit behavior

### Use Discriminated Unions for State

- Model component states explicitly
- Use union types for status
- Exhaustive checking in render
- **Why**: Prevents invalid states

## Node.js-Specific

### Use Appropriate Types

- `@types/node` for Node APIs
- Use `process.env` with type safety
- Type Express handlers properly
- **Why**: Type safety in backend code

### Handle Async Properly

- Always await database calls
- Use `Promise.all()` for parallel operations
- Handle errors in async routes
- **Why**: Prevents unhandled rejections

## Common Pitfalls

### Avoid These Patterns

- `any` type (use `unknown` instead)
- Type assertions without justification
- Ignoring compiler errors
- Non-strict mode
- Empty interfaces
- **Why**: Defeats TypeScript's purpose

## Build and Tooling

### Use Modern Build Tools

- Vite for frontend
- esbuild or swc for speed
- tsc for type checking only
- **Why**: Fast development feedback

### Separate Type Checking from Build

- Run `tsc --noEmit` for type check
- Use faster tools for transpilation
- Type check in CI/CD
- **Why**: Faster builds, reliable checks

## Documentation

### Use JSDoc for Documentation

- JSDoc comments for public APIs
- Show in IDE tooltips
- Generate documentation
- **Why**: Self-documenting code

### Type JSDoc Annotations

- `@param {string} name` with types
- `@returns {Promise<User>}` for return types
- Enhances IDE experience
- **Why**: Documentation and types together

## Project Libraries

### State Management

- React hooks (`useState`, `useReducer`) for component-level state
- TanStack Store for application-wide state
- `immer` for complex immutable state updates
- **Why**: Keeps state management predictable and type-safe

### Data Fetching

- TanStack Query for server state management and caching
- Handles loading, error, and success states
- Automatic refetching and cache invalidation
- **Why**: Simplifies async data management

#### TanStack Query v5: `queryOptions()` Factory Pattern

- Use `queryOptions()` to co-locate `queryKey`, `queryFn`, and options in a single factory
- Do NOT scatter queryKey / queryFn / staleTime across separate files
- Set `staleTime` / `gcTime` per-query, not as global defaults
- Consumers use the factory via `buildXxxQuery({select})` or spread `{...xxxQuery, select}`
- Define `queryKey` directly inside `queryOptions()` — no separate `QueryKeys` module needed

```typescript
export const buildAppConfigQuery = <T = AppConfig>(opts?: AppConfigQueryOptions<T>) =>
  queryOptions({
    queryKey: ['app-config'] as const,
    queryFn: getConfig,
    staleTime: Number.POSITIVE_INFINITY,
    gcTime: Number.POSITIVE_INFINITY,
    ...opts,
  });
export const appConfigQuery = buildAppConfigQuery();
```

- **Why**: Type-safe `getQueryData()` inference via DataTag, single source of truth

#### Optimistic Update Checklist

Mutations with optimistic updates must follow this lifecycle:

1. `onMutate`: Cancel queries → save previous → set optimistic data → return `{previous}`
2. `onError`: Log error + rollback with `context?.previous`
3. `onSettled`: Invalidate queries (covers both success and error)
   - **Exception**: Skip `onSettled` invalidation when `staleTime: Infinity` — `invalidateQueries` triggers a refetch on active observers regardless of staleTime, causing an unnecessary round-trip. The optimistic data from `onMutate` is already correct.

Additional requirements:
- Set `scope: {id: 'xxx'}` to serialize parallel mutations on the same resource
- Always log errors in `onError` — never silently swallow

#### Tauri Desktop App Defaults

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

### UI Components

- Radix Themes as the design system
- Import from `@radix-ui/themes`
- Use theme tokens for colors, spacing, typography
- **Why**: Consistent, accessible component library

### Testing

- Vitest for unit and component testing
- Fast test execution with Vite integration
- Compatible with Jest API
- **Why**: Modern testing with great TypeScript support

### Tauri Integration

- Use Tauri API types for type safety
- Create typed wrappers around Tauri commands
- Handle serialization/deserialization properly
- **Why**: Type-safe communication between frontend and backend

#### WebKit Prefix for CSS

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

#### Dev-Only Code with `__DEV_TOOLS__`

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

### Error Boundaries

#### QueryErrorResetBoundary + ErrorBoundary

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

#### Error Component Design

- Accept `error: Error` and optional `reset?: () => void` props
- Use `useMemo` for expensive derivations (e.g., stack trace parsing)
- Normalize unknown errors at the boundary, not inside the component
- **Why**: Reusable across TanStack Router's `defaultErrorComponent` and React ErrorBoundary
