use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow, bail};
use serde::Deserialize;

use crate::cli::Cli;

#[derive(Debug, Clone)]
pub struct Config {
    pub repo_root: PathBuf,
    pub host: String,
    pub api_key: String,
    pub model: String,
    pub commit_limit: usize,
    pub max_diff_tokens: usize,
    pub max_instructions_tokens: usize,
    pub dry_run: bool,
}

impl Config {
    pub fn load(cli: Cli) -> Result<Self> {
        let current_dir =
            std::env::current_dir().context("failed to determine current directory")?;
        let repo_root = find_git_root(&current_dir)?;

        let config_path = config_file_path()?;
        let file_config = load_file_config(&config_path)?;

        let host = cli
            .host
            .or(file_config.host)
            .unwrap_or_else(|| "https://api.openai.com/v1".to_string());

        let api_key = cli.api_key.or(file_config.api_key).ok_or_else(|| {
            anyhow!(
                "missing API key; set `api_key` in {} or pass --api-key",
                config_path.display()
            )
        })?;

        if cli.commit_limit == 0 {
            bail!("--commit-limit must be greater than 0");
        }

        Ok(Self {
            repo_root,
            host: normalize_host(&host),
            api_key,
            model: cli
                .model
                .or(file_config.model)
                .unwrap_or_else(|| "gpt-4o".to_string()),
            commit_limit: cli.commit_limit,
            max_diff_tokens: cli.max_diff_tokens,
            max_instructions_tokens: cli.max_instructions_tokens,
            dry_run: cli.dry_run,
        })
    }
}

#[derive(Debug, Default, Deserialize)]
struct FileConfig {
    #[serde(alias = "openai_host")]
    host: Option<String>,
    #[serde(alias = "openai_key")]
    api_key: Option<String>,
    #[serde(alias = "openai_model")]
    model: Option<String>,
}

fn load_file_config(config_path: &Path) -> Result<FileConfig> {
    if !config_path.is_file() {
        return Ok(FileConfig::default());
    }

    let content = std::fs::read_to_string(config_path)
        .with_context(|| format!("failed to read {}", config_path.display()))?;

    serde_yaml::from_str(&content)
        .with_context(|| format!("failed to parse {}", config_path.display()))
}

pub(crate) fn config_file_path() -> Result<PathBuf> {
    let config_dir =
        dirs::config_dir().ok_or_else(|| anyhow!("failed to determine OS config directory"))?;
    Ok(config_file_path_from_dir(&config_dir))
}

pub(crate) fn config_file_path_from_dir(config_dir: &Path) -> PathBuf {
    config_dir.join("ai-commit").join("config.yml")
}

pub(crate) fn config_dir_from_file_path(config_path: &Path) -> Option<PathBuf> {
    config_path.parent().map(Path::to_path_buf)
}

pub(crate) fn find_git_root(start: &Path) -> Result<PathBuf> {
    let mut current = start;

    loop {
        if current.join(".git").exists() {
            return Ok(current.to_path_buf());
        }

        current = current
            .parent()
            .ok_or_else(|| anyhow!("current directory is not inside a git repository"))?;
    }
}

fn normalize_host(host: &str) -> String {
    host.trim_end_matches('/').to_string()
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::{config_file_path_from_dir, find_git_root, load_file_config, normalize_host};
    use crate::cli::Cli;

    #[test]
    fn trims_trailing_slash_from_host() {
        assert_eq!(
            normalize_host("https://example.com/v1/"),
            "https://example.com/v1"
        );
    }

    #[test]
    fn finds_git_root_from_nested_directory() {
        let temp = tempdir().unwrap();
        let repo_root = temp.path().join("repo");
        let nested = repo_root.join("a").join("b");

        fs::create_dir_all(repo_root.join(".git")).unwrap();
        fs::create_dir_all(&nested).unwrap();

        assert_eq!(find_git_root(&nested).unwrap(), repo_root);
    }

    #[test]
    fn builds_expected_config_path() {
        let temp = tempdir().unwrap();
        assert_eq!(
            config_file_path_from_dir(temp.path()),
            temp.path().join("ai-commit").join("config.yml")
        );
    }

    #[test]
    fn loads_yaml_config() {
        let temp = tempdir().unwrap();
        let config_path = temp.path().join("config.yml");
        fs::write(
            &config_path,
            "host: https://example.com/v1\napi_key: test-key\nmodel: test-model\n",
        )
        .unwrap();

        let config = load_file_config(&config_path).unwrap();

        assert_eq!(config.host.as_deref(), Some("https://example.com/v1"));
        assert_eq!(config.api_key.as_deref(), Some("test-key"));
        assert_eq!(config.model.as_deref(), Some("test-model"));
    }

    #[test]
    fn supports_openai_prefixed_yaml_keys() {
        let temp = tempdir().unwrap();
        let config_path = temp.path().join("config.yml");
        fs::write(
            &config_path,
            "openai_host: https://example.com/v1\nopenai_key: test-key\nopenai_model: test-model\n",
        )
        .unwrap();

        let config = load_file_config(&config_path).unwrap();

        assert_eq!(config.host.as_deref(), Some("https://example.com/v1"));
        assert_eq!(config.api_key.as_deref(), Some("test-key"));
        assert_eq!(config.model.as_deref(), Some("test-model"));
    }

    #[test]
    fn cli_defaults_include_dry_run_disabled() {
        let cli = Cli {
            host: None,
            api_key: None,
            model: None,
            commit_limit: 20,
            max_diff_tokens: 64_000,
            max_instructions_tokens: 10_000,
            dry_run: false,
        };

        assert!(!cli.dry_run);
    }
}
