# KoDoGu App

The KoDoGu desktop app. It lives in the macOS menu bar.

## Tech Stack

- Frontend: React + TypeScript (TanStack Router, TanStack Query)
- Backend: Rust (Tauri v2)
- Package manager: Bun

## Development

With [mise](https://mise.jdx.dev/getting-started.html) and the
[Tauri prerequisites](https://tauri.app/start/prerequisites/) installed, run these from the repository root:

```bash
mise install
mise exec -- bun install --frozen-lockfile
mise run app:tauri -- dev   # Start the app
mise run app:storybook      # Start Storybook
mise run app:tauri:build    # Build for distribution
```

Run checks and automatic fixes for the entire repository from the repository root.
Use `--continue-on-error` to finish all selected tasks and report failure if any task fails.
Without it, execution stops on failure:

```bash
mise run --continue-on-error lint
mise run --continue-on-error test
mise run --continue-on-error test:coverage
mise run --continue-on-error fix
```

Run only frontend tests, either once or in watch mode:

```bash
mise run app:test
mise run app:test:watch
```

Task definitions are grouped by [repository](../.config/mise/tasks/repository.toml),
[app](../.config/mise/tasks/app.toml), and [Cargo](../.config/mise/tasks/cargo.toml).

The shared `Tauri Frontend dev server` run configuration requires the mise plugin in JetBrains IDEs.
