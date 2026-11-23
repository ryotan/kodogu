# KoDoGu App

The KoDoGu desktop app. It lives in the macOS menu bar.

## Packages

- `app-root/`: the startup: the entry of each window, Vite, and the Tauri application in `app-root/src-tauri/`
- `routing/`: the route definitions of TanStack Router without screens, and the router
- `foundation/`: everything else of the desktop app that tools may import, with its Tauri crate in
  `foundation/src-tauri/`

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
mise run app:tauri -- dev                       # Start the app
mise run //apps/desktop/foundation:storybook    # Start the Storybook of a package
mise run app:tauri:build                        # Build for distribution
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

Run the frontend tests of every package, or watch the tests of one package:

```bash
mise run app:test
mise run //apps/desktop/foundation:test:watch
```

Each package has its own tasks: inside its directory, `mise run test` runs its tests.

Task definitions are grouped by [repository](../../.config/mise/tasks/repository.toml),
[app](../../.config/mise/tasks/app.toml), and [Cargo](../../.config/mise/tasks/cargo.toml); the tasks of each package
are in [package-tasks](../../.config/mise/package-tasks/).

The shared `Tauri Frontend dev server` run configuration requires the mise plugin in JetBrains IDEs.
