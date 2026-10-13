# KoDoGu App

The KoDoGu desktop app. It lives in the macOS menu bar.

## Tech Stack

- Frontend: React + TypeScript (TanStack Router, TanStack Query)
- Backend: Rust (Tauri v2)
- Package manager: Bun

## Development

With the [Tauri prerequisites](https://tauri.app/start/prerequisites/) installed, run these in this directory:

```bash
bun install
bun run tauri dev     # Start the app
bun run storybook     # Start Storybook
bun run lint          # Run static analysis
bun run test          # Run the frontend and Rust tests
bun run tauri:build   # Build for distribution
```

See [CONTRIBUTING.md](../CONTRIBUTING.md) for the project's conventions.
