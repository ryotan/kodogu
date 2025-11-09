# Git Practices

> [!Note]
> Users are indifferent to Git status, so you can treat both staged and unstaged hunks in the same way.
> When a user refers to "uncommitted changes", consider it to include all "uncommitted" changes in Git, including
> staged/unstaged/untracked changes.

## Git Branching Conventions

- `main` branch is the main branch
- `develop` branch is the development branch
- `feature/*` branches are used for feature development
- `hotfix/*` branches are used for hotfixes

## Git Tag Conventions

- `v*` tags are used for releases

## Git Workflow Conventions

- Always use `feature/*` branches or `hotfix/*` branches for development
- DO NOT commit directly to `main` or `develop` branches
- Always use `pull request` to merge changes to `main` or `develop` branches

## Git Commit Granularity Conventions

Commits should be granular and focused. Make commits at logical task boundaries. Also, when an AI Agent
attempts to commit automatically, it should inform the user about the intention to commit and its contents.

DO NOT commit automatically if the branch is `main` or `develop`. If the worktree is for features, then commit automatically.

## Git Commit Message Conventions

Commit messages comply with Conventional Commits, using [GitMoji](https://gitmoji.dev/) with Unicode Emoji input

### Format

```
`<emoji>: <summary>`

* <description 1 (optional)>
* ...
```

### GitMoji meanings

- 🎨: Improve structure / format of the code.
- ⚡️: Improve performance.
- 🔥: Remove code or files.
- 🐛: Fix a bug.
- 🚑️: Critical hotfix.
- ✨: Introduce new features.
- 📝: Add or update documentation.
- 🚀: Deploy stuff.
- 💄: Add or update the UI and style files.
- 🎉: Begin a project.
- ✅: Add, update, or pass tests.
- 🔒️: Fix security or privacy issues.
- 🔐: Add or update secrets.
- 🔖: Release / Version tags.
- 🚨: Fix compiler / linter warnings.
- 🚧: Work in progress.
- 💚: Fix CI Build.
- ⬇️: Downgrade dependencies.
- ⬆️: Upgrade dependencies.
- 📌: Pin dependencies to specific versions.
- 👷: Add or update CI build system.
- 📈: Add or update analytics or track code.
- ♻️: Refactor code.
- ➕: Add a dependency.
- ➖: Remove a dependency.
- 🔧: Add or update configuration files.
- 🔨: Add or update development scripts.
- 🌐: Internationalization and localization.
- ✏️: Fix typos.
- 💩: Write bad code that needs to be improved.
- ⏪️: Revert changes.
- 🔀: Merge branches.
- 📦️: Add or update compiled files or packages.
- 👽️: Update code due to external API changes.
- 🚚: Move or rename resources (e.g.: files, paths, routes).
- 📄: Add or update license.
- 💥: Introduce breaking changes.
- 🍱: Add or update assets.
- ♿️: Improve accessibility.
- 💡: Add or update comments in source code.
- 🍻: Write code drunkenly.
- 💬: Add or update text and literals.
- 🗃️: Perform database related changes.
- 🔊: Add or update logs.
- 🔇: Remove logs.
- 👥: Add or update contributor(s).
- 🚸: Improve user experience / usability.
- 🏗️: Make architectural changes.
- 📱: Work on responsive design.
- 🤡: Mock things.
- 🥚: Add or update an easter egg.
- 🙈: Add or update a .gitignore file.
- 📸: Add or update snapshots.
- ⚗️: Perform experiments.
- 🔍️: Improve SEO.
- 🏷️: Add or update types.
- 🌱: Add or update seed files.
- 🚩: Add, update, or remove feature flags.
- 🥅: Catch errors.
- 💫: Add or update animations and transitions.
- 🗑️: Deprecate code that needs to be cleaned up.
- 🛂: Work on code related to authorization, roles and permissions.
- 🩹: Simple fix for a non-critical issue.
- 🧐: Data exploration/inspection.
- ⚰️: Remove dead code.
- 🧪: Add a failing test.
- 👔: Add or update business logic.
- 🩺: Add or update healthcheck.
- 🧱: Infrastructure related changes.
- 🧑‍💻: Improve developer experience.
- 💸: Add sponsorships or money related infrastructure.
- 🧵: Add or update code related to multithreading or concurrency.
- 🦺: Add or update code related to validation.
- ✈️: Improve offline support.
