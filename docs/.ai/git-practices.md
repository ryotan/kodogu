# Git Practices

> [!Note]
> When users refer to "changes" or "uncommitted changes", they typically mean
> ALL uncommitted work (staged, unstaged, and untracked). AI agents should
> consider all these when working with changes unless the user specifically
> mentions "staged changes only" or similar qualifications.

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

Commits should be granular and focused. Make commits at logical task boundaries.

### Automatic Commit Guidelines for AI Agents

AI agents MAY commit automatically when ALL of the following conditions are met:
- On `feature/*` or `hotfix/*` branches (NOT on `main` or `develop`)
- User explicitly requested a commit or task completion
- All automated checks passing (tests, linters, type checks, formatters)

AI agents MUST NOT commit automatically when:
- On `main` or `develop` branches
- Automated checks are failing or not run yet
- User hasn't reviewed the changes
- Multiple unrelated changes are mixed together
- User explicitly requested "do not commit"

When committing automatically, AI agents MUST:
- Inform the user about the intention to commit
- Describe what will be committed
- Wait for user confirmation (AI cannot reliably judge if changes form a single logical unit)

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
