# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
## [0.1.3](https://github.com/falleng0d/AI-Commit/compare/v0.1.2...v0.1.3) - 2026-04-05

### Added

- switch from character-based to token-based truncation for diff and instructions with tiktoken integration

## [0.1.2](https://github.com/falleng0d/AI-Commit/compare/v0.1.1...v0.1.2) - 2026-04-05

### Added

- enhance prompt template with detailed semantic commit guidelines and formatting rules

## [0.1.1](https://github.com/falleng0d/AI-Commit/compare/v0.1.0...v0.1.1) - 2026-04-05

### Added

- support configurable prompt template via PROMPT.md in config directory


### Documentation

- support loading prompt template from PROMPT.md in config directory


### Maintenance

- remove unused serde_json dependency from Cargo.toml and Cargo.lock

## [0.1.0](https://github.com/falleng0d/AI-Commit/releases/tag/v0.1.0) - 2026-04-05

### Added

- add support for config file with YAML and env var fallbacks

- add AI commit message generator CLI


### Build

- add release automation workflows


### Documentation

- add README with detailed usage, configuration, and development instructions


### Maintenance

- add author information to Cargo.toml

- change default model to gpt-4o

- update default API host to OpenAI endpoint


### Other

- Init


### Tests

- add coverage for repo context and model fallback

