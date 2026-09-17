# Repository Guidelines

## Branch and pull-request workflow

Keep `main` as a clean, reviewable integration branch. Unless the user explicitly asks otherwise:

1. Start feature, fix, refactor, and other substantive work from an up-to-date `main` on a short-lived branch.
2. Use a descriptive branch name such as `feat/addon-updates`, `fix/loadout-scan`, or `docs/install-guide`.
3. Keep commits focused and use concise, imperative messages. Conventional Commit prefixes (`feat:`, `fix:`, `docs:`, `refactor:`, `test:`, `chore:`) are preferred but not mandatory.
4. Push the branch and merge it through a pull request after checks pass. Do not push substantive work directly to `main` or force-push shared branches unless the user explicitly approves it.
5. Prefer **squash merging**. Format the squash commit as `<PR title> (#<PR number>)` so every feature-level commit on `main` links back to its pull request. Example: `feat: add loadout export (#42)`.
6. Delete the feature branch after merge when it is no longer needed.

Small documentation or metadata-only changes may go directly to `main` when the user requests that simpler path. Emergency fixes may also bypass the normal flow with explicit user approval; preserve a clear commit message and follow up with a pull request or issue reference when practical.

Before beginning work, check the current branch and working tree. If currently on `main` and the task is substantive, propose or create an appropriately named feature branch rather than silently committing to `main`. Never discard unrelated local changes.

## Pull requests

- Give each pull request a clear, outcome-oriented title suitable for its eventual squash commit.
- Summarize what changed, why it changed, and how it was tested.
- Keep each pull request scoped to one coherent change when practical.
- Resolve review feedback on the feature branch and keep required checks green before merging.
- Use GitHub's squash-merge defaults that append the pull request number to the commit title; verify the final title contains `(#<PR number>)` before merging.
