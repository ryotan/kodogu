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
├── cli-app/<app-name>/       # CLI app source code
│   ├── src/                  # Rust source code
│   ├── tests/                # Rust test code (mainly for integration tests)
│   ├── Cargo.toml            # Rust dependencies
│   └── README.md             # README for CLI app
├── CONTRIBUTING.md           # Contribution guidelines
└── README.md                 # Project overview and setup instructions
```

## Development Environment Setup

See `README.md` for each CLI apps. The following steps are for the Tauri app.

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

## General Development Guidelines

This section outlines the high-level development guidelines for the KoDoGu project.
These guidelines apply to all aspects of the project, regardless of the programming language or technology used.

### 1. Application Basic Policy

- Designed as a collection of personal utility tools
- Keep each feature capable of operating independently
- Maintain a simple and intuitive UI

### 2. UI/UX Guidelines

- Unified design system based on Radix UI
- Full support for keyboard operation
- Error messages should be specific and practical

### 3. Security Policy

- Request only the minimum necessary system permissions
- Store sensitive data securely
- Operate offline by default

### 4. Testing Policy

- Unit tests are mandatory
- E2E tests are mandatory only for important features
- Target test coverage of 80% or higher for meaningful coverage
  - Focus coverage on:
    - Domain logic (aim for 90%+)
    - Critical business rules (100%)
    - Public APIs
  - Lower priority for:
    - UI presentation logic
    - Framework glue code
  - Quality over quantity: covering edge cases matters more than raw percentage

### 5. Documentation Conventions

- Prepare README.md for each feature
- Generate API documentation automatically
- Include concrete examples for error cases

### 6. Versioning

- Follow semantic versioning

### 7. Performance Requirements

- Target memory usage below 100MB
- Response time for each operation within 500ms

### 8. Cross-Platform Support

- Provide equivalent functionality on Windows/macOS/Linux
- Establish an abstraction layer for OS-specific features
- Separate platform-specific settings

## Coding Conventions

See `docs/.ai/` for comprehensive development practices:

- [Design Practices](./docs/.ai/design-practices.md) - DDD and Clean Architecture principles
- [Testing Practices](./docs/.ai/testing-practices.md) - Testing strategy and patterns
- [Coding Practices](./docs/.ai/coding-practices.md) - Cross-language implementation guidance
- [Git Practices](./docs/.ai/git-practices.md) - Commit conventions and workflow
- [Kotlin Practices](./docs/.ai/kotlin-practices.md) - For backend development
- [Rust Practices](./docs/.ai/rust-practices.md) - For CLI apps and Tauri backend
- [TypeScript Practices](./docs/.ai/typescript-practices.md) - For frontend development
- [AI Agent Guidelines](./docs/.ai/guidelines.md) - Additional instructions for AI coding assistants

See each document for detailed guidance on implementing features, writing tests, and maintaining code quality.

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

## Notes on Tauri

- This project uses Tauri v2.
- Refer to the [Tauri v2 official documentation](https://tauri.app/start/) for API references.
- Use the [Tauri plugin system](https://tauri.app/guides/plugins/) for plugin development.

## Git Workflow

### Branching Strategy

- `main` - Production-ready code
- `develop` - Integration branch for ongoing work
- `feature/*` - New features (e.g., `feature/add-dark-mode`)
- `hotfix/*` - Production hotfixes (e.g., `hotfix/fix-crash`)

**Rules**:
- Never commit directly to `main` or `develop`
- Always create pull requests for merging
- Use descriptive branch names

### Commit Message Format

We use [GitMoji](https://gitmoji.dev/) with conventional commits:

```
✨: Add dark mode toggle
🐛: Fix crash on startup
📝: Update installation guide
```

Common emojis:
- ✨ New feature
- 🐛 Bug fix
- 📝 Documentation
- ♻️ Refactor
- ✅ Tests
- 🚀 Deploy

See [Git Practices](./docs/.ai/git-practices.md) for complete emoji list, detailed workflow guidelines, and AI agent commit rules.

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
