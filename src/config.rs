use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow, bail};

use crate::cli::Cli;

#[derive(Debug, Clone)]
pub struct Config {
    pub repo_root: PathBuf,
    pub host: String,
    pub api_key: String,
    pub model: String,
    pub commit_limit: usize,
    pub max_diff_chars: usize,
    pub max_instructions_chars: usize,
}

impl Config {
    pub fn load(cli: Cli) -> Result<Self> {
        let current_dir =
            std::env::current_dir().context("failed to determine current directory")?;
        let repo_root = find_git_root(&current_dir)?;

        let env_path = repo_root.join(".env");
        if env_path.is_file() {
            let _ = dotenvy::from_path(&env_path);
        }

        let host = cli
            .host
            .or_else(|| std::env::var("OPENAI_HOST").ok())
            .unwrap_or_else(|| "https://api.cerebras.ai/v1".to_string());

        let api_key = cli
            .api_key
            .or_else(|| std::env::var("OPENAI_KEY").ok())
            .or_else(|| std::env::var("OPENAI_API_KEY").ok())
            .ok_or_else(|| anyhow!("missing API key; set OPENAI_KEY or pass --api-key"))?;

        if cli.commit_limit == 0 {
            bail!("--commit-limit must be greater than 0");
        }

        Ok(Self {
            repo_root,
            host: normalize_host(&host),
            api_key,
            model: cli
                .model
                .unwrap_or_else(|| "qwen-3-235b-a22b-instruct-2507".to_string()),
            commit_limit: cli.commit_limit,
            max_diff_chars: cli.max_diff_chars,
            max_instructions_chars: cli.max_instructions_chars,
        })
    }
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

    use super::{find_git_root, normalize_host};

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
}
