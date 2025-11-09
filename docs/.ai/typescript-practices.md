# TypeScript Style Guide

This style guide outlines the conventions and best practices for TypeScript code in the KoDoGu project. Following these
guidelines will ensure code consistency, readability, and maintainability across the codebase.

## Code Formatting

- Use ESLint and Biome for code formatting and linting
- Run `bun run fix` before committing code to ensure consistent formatting
- Use 2 spaces for indentation (as specified in .editorconfig)
- Maximum line length is 120 characters
- Use LF line endings
- Always include a final newline in files
- Use single quotes for strings
- No trailing commas in function parameters
- Trailing commas in object and array literals

## Naming Conventions

- Use **camelCase** for variables, functions, methods, and properties
  ```typescript
  const userName = 'example';
  function calculateTotalPrice() { /* ... */ }
  ```
- Use **PascalCase** for classes, interfaces, types, enums, and React components
  ```typescript
  class UserProfile { /* ... */ }
  interface UserData { /* ... */ }
  type ConnectionState = 'connected' | 'disconnected';
  enum HttpStatus { OK, NotFound, ServerError }
  function UserAvatar() { /* ... */ }
  ```
- Use **SCREAMING_SNAKE_CASE** for constants and static variables
  ```typescript
  const MAX_CONNECTIONS = 100;
  const DEFAULT_TIMEOUT = 30000;
  ```
- Do not use "I" prefix for interfaces
- Use descriptive names that clearly convey purpose
- Prefer complete words over abbreviations unless the abbreviation is more widely recognized

## Code Organization

- Organize code into modules based on functionality
- Keep files focused on a single responsibility
- Follow a consistent directory structure:
  - `components/`: Reusable UI components
  - `api/`: API client code and data fetching
  - `routes/`: Route definitions and page components
  - `types/`: TypeScript type definitions
  - `utils/`: Utility functions
  - `hooks/`: Custom React hooks
  - `stores/`: State management

## React Components

- Use functional components with hooks instead of class components
  ```typescript jsx
  function UserProfile({ userId }: UserProfileProps) {
    const [user, setUser] = useState<User | null>(null);
    // ...
    return <div>...</div>;
  }
  ```
- Extract complex logic into custom hooks
- Keep components small and focused on a single responsibility
- Use proper prop typing with TypeScript interfaces
  ```typescript
  interface UserProfileProps {
    userId: string;
    showAvatar?: boolean;
  }
  ```
- Use destructuring for props and state
- Use React.Fragment or the shorthand `<>...</>` to avoid unnecessary divs

## State Management

- Use React hooks (`useState`, `useReducer`) for component-level state
- Use TanStack Store for application-wide state
- Prefer immutable state updates
- Use immer for complex state updates
- Keep state as close as possible to where it's used

## TypeScript Best Practices

- Enable strict mode in TypeScript configuration
- Explicitly type function parameters and return types
  ```typescript
  function fetchUser(userId: string): Promise<User> {
    // ...
  }
  ```
- Use type inference when the type is obvious
- Use interfaces for object shapes that will be implemented or extended
- Use type aliases for unions, intersections, and complex types
- Avoid using `any` type; use `unknown` instead when type is truly unknown
- Use generics for reusable components and functions
- Use discriminated unions for state management
  ```typescript
  type RequestState<T> =
    | { status: 'idle' }
    | { status: 'loading' }
    | { status: 'success'; data: T }
    | { status: 'error'; error: Error };
  ```

## Asynchronous Code

- Use async/await instead of Promise chains when possible
- Handle errors properly in async functions
- Use TanStack Query for data fetching and caching
- Implement proper loading and error states for async operations

## Tauri Integration

- Use Tauri API types for better type safety
- Create typed wrappers around Tauri commands
  ```typescript
  async function saveUserPreferences(prefs: UserPreferences): Promise<void> {
    return invoke('save_user_preferences', { prefs });
  }
  ```
- Handle errors from Tauri commands appropriately
- Use proper serialization/deserialization for data passed between TypeScript and Rust

## Testing

- Write unit tests for utility functions and hooks
- Write component tests for UI components
- Use Vitest for testing
- Use meaningful test descriptions
- Follow the Arrange-Act-Assert pattern in tests
- Mock external dependencies in tests

## Documentation

- Document complex functions and components with JSDoc comments
  ```typescript
  /**
   * Fetches user data from the API
   * @param userId - The ID of the user to fetch
   * @returns A promise that resolves to the user data
   * @throws {ApiError} If the user cannot be found
   */
  async function fetchUser(userId: string): Promise<User> {
    // ...
  }
  ```
- Include examples in documentation where appropriate
- Document non-obvious implementations with regular comments
- Use `// TODO:` or `// FIXME:` comments for temporary solutions or known issues

## Performance Considerations

- Use memoization for expensive calculations (`useMemo`, `useCallback`)
- Avoid unnecessary re-renders
- Use virtualization for long lists
- Be mindful of bundle size when adding dependencies
- Use code splitting for large applications

## Common Pitfalls to Avoid

- Excessive prop drilling (use context or state management instead)
- Overusing global state
- Not handling loading and error states
- Type casting with `as` without proper validation
- Complex conditional rendering (extract to helper functions)
- Mixing synchronous and asynchronous code without clear boundaries

## UI Components and Design System

- Use Radix Themes as the design system for UI components
- Import Radix Themes components from '@radix-ui/themes'
- Follow Radix Themes' component API and styling conventions
- Use the provided theme tokens for colors, spacing, typography, etc.
- Wrap your application with the Theme component to enable theming
- Use the 'rt-reset' class on the body element to reset browser styles
- Customize themes using the theme configuration options provided by Radix
- For components not available in Radix Themes, create custom components that follow the same design principles

## Dependencies

- Be selective about adding new dependencies
- Prefer well-maintained packages with good TypeScript support
- Consider the bundle size implications of dependencies
- Keep dependencies up to date

By following these guidelines, we can maintain a high-quality, consistent, and maintainable TypeScript codebase for the
KoDoGu project.
