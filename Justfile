#!/usr/bin/env just --justfile

set shell := ["pwsh", "-NoLogo", "-NoProfile", "-Command"]

crate_name := "ai-commits"
default_branch := "master"
ci_workflow := "ci.yml"
release_workflow := "release-plz.yml"

default:
  just --list

status:
  git status --short --branch

clean:
  cargo clean

fmt:
  cargo fmt

fmt-check:
  cargo fmt --check

lint:
  cargo clippy --all-targets --all-features -- -D warnings

test:
  cargo test

build:
  cargo build

build-release:
  cargo build --release

run *args:
  cargo run --release -- {{args}}

install:
  cargo install --path .

check: fmt-check lint test

ci: fmt-check lint test build-release publish-dry-run

package:
  cargo package --locked

publish-dry-run:
  cargo publish --dry-run --locked

release-plz-check:
  if (Test-Path "{{release_plz}}") { exit 0 }
  if (Get-Command "{{release_plz}}" -ErrorAction SilentlyContinue) { exit 0 }
  Write-Error "release-plz not found. Set RELEASE_PLZ to the binary path or place it at {{release_plz}}"
  exit 1

release-plz-version: release-plz-check
  {{release_plz}} --version

version: release-plz-check
  {{release_plz}} update --config release-plz.toml

version-from registry_manifest_path: release-plz-check
  {{release_plz}} update --config release-plz.toml --registry-manifest-path {{registry_manifest_path}}

release-pr: release-plz-check
  {{release_plz}} release-pr --config release-plz.toml

release-dry-run: release-plz-check
  {{release_plz}} release --config release-plz.toml --dry-run

release: release-plz-check
  {{release_plz}} release --config release-plz.toml

gh-ci:
  gh workflow run {{ci_workflow}} --ref {{default_branch}}

gh-release-pr dry_run="true":
  gh workflow run {{release_workflow}} --ref {{default_branch}} -f command=release-pr -f dry_run={{dry_run}}

gh-release-dry-run:
  gh workflow run {{release_workflow}} --ref {{default_branch}} -f command=release -f dry_run=true

gh-release:
  gh workflow run {{release_workflow}} --ref {{default_branch}} -f command=release -f dry_run=false

gh-release-both dry_run="true":
  gh workflow run {{release_workflow}} --ref {{default_branch}} -f command=both -f dry_run={{dry_run}}

gh-runs workflow=release_workflow limit="10":
  gh run list --workflow {{workflow}} --limit {{limit}}

gh-wait run_id poll_seconds="10":
  pwsh -NoProfile -File scripts/wait-gh-run.ps1 -RunId {{run_id}} -PollSeconds {{poll_seconds}}

crate-info:
  cargo info {{crate_name}}

crate-search:
  cargo search {{crate_name}} --limit 5
