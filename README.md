# ai-commit

`ai-commit` is a Rust CLI that generates a semantic commit message from the current git repository context.

The published crate name is `ai-commits`, while the installed binary is `ai-commit`.

It gathers:

- the last 30 commits by the current git user
- staged and unstaged tracked changes
- `CLAUDE.md` at the repository root, or `AGENTS.md` if `CLAUDE.md` is absent

The model is instructed to return only the commit message, for example:

```text
refactor: Reformat code
```

## Configuration

Configuration is loaded from `config.yml` in the OS config directory.

Examples:

- Windows: `%APPDATA%\ai-commit\config.yml`
- macOS: `~/Library/Application Support/ai-commit/config.yml`
- Linux: `~/.config/ai-commit/config.yml`

Example file:

```yaml
host: https://api.cerebras.ai/v1
api_key: your-api-key
model: qwen-3-235b-a22b-instruct-2507
```

Supported keys:

- `host`
- `api_key`
- `model`

The legacy `openai_host`, `openai_key`, and `openai_model` keys are also accepted.

CLI flags override the config file:

- `--host`
- `--api-key`
- `--model`
- `--commit-limit`
- `--max-diff-chars`
- `--max-instructions-chars`

## Usage

```bash
cargo run --release
```

Or build the binary first:

```bash
cargo build --release
target/release/ai-commit
```

After publishing to crates.io:

```bash
cargo install ai-commits
ai-commit
```

## Development

```bash
cargo fmt
cargo test
cargo build
```

## Release Automation

The repository uses GitHub Actions and `release-plz` for automated releases.

- `CI` runs formatting, linting, tests, release builds, and `cargo publish --dry-run`
- `Release` opens release PRs, updates `CHANGELOG.md`, creates tags, creates GitHub releases, and publishes to crates.io

## Notes

- The tool uses the current git repository and git user identity from local git config.
- The output is a single semantic commit message line.
- The AI provider must expose an OpenAI-compatible `chat/completions` endpoint.
