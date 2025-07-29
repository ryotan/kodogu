# KoDoGu Development Guide

This document provides guidelines for contributors to the KoDoGu project. It includes important information you should
know when participating in the project, such as how to write code, commit rules, how to create PRs, and more.

## Directory Structure

```
kodogu/
├── app/                      # Frontend source code
│   ├── src/                  # React components, styles, routing, etc.
│   ├── public/               # Static assets
│   ├── index.html            # HTML entry point
│   ├── package.json          # npm dependencies and scripts
│   └── vite.config.ts        # Vite configuration
├── app/src-tauri/            # Tauri backend source code
│   ├── src/                  # Rust source code
│   ├── Cargo.toml            # Rust dependencies
│   └── tauri.conf.json       # Tauri configuration
├── CONTRIBUTING.md           # Contribution guidelines
└── README.md                 # Project overview and setup instructions
```

## Development Environment Setup

1. Clone the repository:
   ```
   git clone https://github.com/ryotan/kodogu.git
   cd kodogu
   ```

2. Install dependencies:
   ```
   cd app
   bun install
   ```

3. Start the development server:
   ```
   bun tauri:dev
   ```

4. Start the Storybook server:
   ```
   bun storybook
   ```

5. Run static analysis:
   ```
   bun lint
   ```

6. Run tests:
   ```
   bun run test
   ```

## Coding Conventions

### TypeScript / JavaScript

- Use [ESLint](https://eslint.org/) and [Biome](https://biomejs.dev/) to maintain code quality and consistency.
- Use camelCase for variable and function names.
- Use PascalCase for component names.
- Do not prefix interfaces with `I`.
- Use PascalCase for types.
- Use function components for React components.
- Use TanStack Store for state management.

### Rust

- Use `cargo fmt` and `cargo clippy` to maintain code quality.
- Use snake_case for variable and function names.
- Use PascalCase for structs and enums.
- Handle errors appropriately and use the `Result` type whenever possible.
- Call Tauri APIs asynchronously.

## Stereotypical Structure of a Tauri App

A Tauri app typically consists of the following directories and files.

```
src-tauri/
├── src/                      # Rust backend source code
│   ├── main.rs               # Application entry point
│   ├── lib.rs                # Application logic
│   └── commands.rs           # Commands called from the frontend
├── Cargo.toml                # Rust dependencies
└── tauri.conf.json           # Tauri configuration
```

- `src/main.rs`: The application entry point that initializes the Tauri application and creates windows.
- `src/lib.rs`: Contains application logic.
- `src/commands.rs`: Contains commands called from the frontend. These commands provide system-level functionality using
  the Tauri API.
- `Cargo.toml`: Describes Rust dependencies.
- `tauri.conf.json`: Describes Tauri application configuration.

The typical workflow of a Tauri app is as follows.

1. Build the UI in the frontend.
2. Write backend logic in Rust.
3. Call backend commands from the frontend.
4. Build the application using Tauri.

## Application Layer Structure

### Frontend (app/)

The frontend application consists of the following layers.

- **Component Layer (src/components/)**: Contains reusable React components that make up UI elements.
- **Style Layer (src/main.css)**: Contains CSS files that define styles for the entire application.
- **Routing Layer (src/routes/)**: Contains React Router components that define application routing.
- **API Layer (src/api/)**: Contains functions that handle communication with the backend API.
- **Asset Layer (src/assets/)**: Contains static assets such as images, fonts, and icons.

### Backend (src-tauri/src/)

The backend application consists of the following layers.

- **Main Layer (src/main.rs)**: The application entry point that initializes the Tauri application and creates windows.
- **Library Layer (src/lib.rs)**: Contains application logic.
- **Command Layer (src/commands.rs)**: Contains commands called from the frontend. These commands provide system-level
  functionality using the Tauri API.
- **Model Layer (src/models.rs)**: Contains data structures used for interacting with the database.
- **Utility Layer (src/utils.rs)**: Contains utility functions used throughout the application.

## Commit Message Conventions

以下の形式でコミットメッセージを書いてください:
Write commit messages in the following format:

```
<type>: <summary>

<description>
```

Example:

```
✨: Add Git diff display feature

- Display uncommitted changes
- Display staged changes
- Display untracked files
```

### Types

タイプは[Gitmoji](https://gitmoji.dev/)を参考にしてください。以下は一部の例です:
Refer to [Gitmoji](https://gitmoji.dev/) for types. Here are some examples:

- ✨: New feature
- 🐛: Bug fix
- 📝: Documentation only changes
- 🎨: Changes that do not affect code behavior (such as formatting)
- ♻️: Code changes that are neither bug fixes nor feature additions
- ⚡️: Changes for performance improvement
- ✅: Add or fix tests
- etc.

## プルリクエスト（PR）の作成

## Creating Pull Requests (PRs)

1. Create a new branch (include feature name or bug fix name):
   ```
   git checkout -b feat/git-diff-viewer
   ```

2. Implement changes and commit:
   ```
   git add .
   git commit -m "✨: Add Git diff display feature"
   ```

3. Push to remote branch:
   ```
   git push -u origin feat/git-diff-viewer
   ```

4. Create a pull request on GitHub. Include the following in the PR description:

- Summary of changes
- Related issue number
- How to test
- Screenshots (if there are UI changes)

## Testing

- Use [Vitest](https://vitest.dev/) for frontend testing.
  - Navigate to the `app` directory and run `bun test`.
- Run Rust tests with `cargo test`.
  - Navigate to the `src-tauri` directory and run `cargo test`.
- Use `bun lint` for code quality checks.
  - Navigate to the `app` directory and run `bun lint`.
  - This command runs ESLint, TypeScript type checking, Biome, and Rust's `cargo fmt` and `cargo clippy`.
- Before submitting a PR, make sure all tests pass and there are no lint errors.
- To ensure code quality, be sure to run `bun lint` and `bun run test`. These commands run code quality checks and tests
  to ensure there are no issues.

## Notes on Tauri

- This project uses Tauri v2.
- Refer to the [Tauri v2 official documentation](https://tauri.app/start/) for API references.
- Use the [Tauri plugin system](https://tauri.app/guides/plugins/) for plugin development.

## Release Process

1. The `main` branch is always kept in a stable state.
2. Releases are made by tagging: `v1.0.0`, `v1.0.1`, etc.
3. Follow semantic versioning:
  - Major: Incompatible changes
  - Minor: Backward-compatible feature additions
  - Patch: Backward-compatible bug fixes

## Help

If you have questions or need assistance, create an Issue or contact the project administrator.

---

Thank you for your contribution!
