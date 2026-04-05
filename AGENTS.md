# AGENTS.md

This repository contains `ai-commit`, a Rust CLI that generates a semantic git commit message from the current repository context using an AI provider.

## Keep In Mind

- The published crate is `ai-commits`; the binary is `ai-commit`.
- Config is loaded from the OS config directory as `config.yml`, not from `.env`.
- The tool should output a single commit message line.
- Keep changes small and prefer the existing structure.
- Release automation uses `release-plz` and GitHub Actions.

## Local Commands

- Format: `just fmt`
- Check: `just check`
- Test: `just test` (runs with `check`) 
- Build: `just build`
- Build release binary: `just build-release`
- Package check: `just package`
- Dry-run publish: `just publish-dry-run`

## Versioning and Release

- Bump version locally: `just version`
- Create/update release PR: `just release-pr`
- Dry-run release flow: `just release-dry-run`
- Run real release flow: `just release`

If `release-plz` is not on `PATH`, set `RELEASE_PLZ` or place the binary at `tools/release-plz.exe`.

## GitHub Actions

- Run CI workflow: `just gh-ci`
- Run release workflow dry-run: `just gh-release-dry-run`
- Run release workflow: `just gh-release`
- List recent runs: `just gh-runs`
- Wait for a run: `just gh-wait <run-id>`

## Before Finishing

- Run `just check` and `just fmt` as needed.
- If release-related files changed, also run `just publish-dry-run`.
