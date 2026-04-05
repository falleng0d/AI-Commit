# Release

This repository uses `release-plz` and GitHub Actions for normal releases.

## Recommended Order

1. Run local checks:

```powershell
just ci
```

2. Commit your changes using semantic commits such as `feat: ...` or `fix: ...`.
3. Push your branch.
4. Open and merge your pull request into `master`.
5. Wait for GitHub to open or update the release PR.
6. Review and merge the release PR.
7. GitHub will publish the crate to crates.io, create the git tag, and create the GitHub release.

## Important Notes

- Yes, you should build and test first. Use `just ci`.
- Yes, you should push first for the normal GitHub-managed release flow.
- No, you usually should not run `just release` for standard releases.

## What Each Command Does

```powershell
just ci
```

Runs formatting, linting, tests, release build, and publish dry-run locally.

```powershell
just version
```

Locally updates the version and changelog preview using `release-plz`.

```powershell
just release-pr
```

Creates or updates the release PR locally using `release-plz`.

```powershell
just release-dry-run
```

Simulates the release step locally without publishing.

```powershell
just release
```

Runs a real local release. This is usually only for manual or emergency releases.

## Version Bump Rules

- `feat:` -> minor bump
- `fix:` -> patch bump
- breaking change -> major bump

## Typical Developer Flow

```powershell
just ci
git add .
git commit -m "feat: add X"
git push
```

Then on GitHub:

1. Merge into `master`
2. Wait for the release PR
3. Merge the release PR

## Useful GitHub Helpers

```powershell
just gh-runs
just gh-release-dry-run
just gh-wait <run-id>
```
