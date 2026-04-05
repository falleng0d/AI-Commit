use std::path::Path;
use std::process::Command;
use std::sync::OnceLock;

use anyhow::{Context, Result, anyhow};
use tiktoken_rs::{CoreBPE, cl100k_base};

use crate::config::Config;

#[derive(Debug, Clone)]
pub struct RepositoryContext {
    pub repo_root: String,
    pub recent_commits: Vec<String>,
    pub tracked_changes: String,
    pub instructions: Option<String>,
}

impl RepositoryContext {
    pub fn gather(config: &Config) -> Result<Self> {
        let user_name = git_output(&config.repo_root, ["config", "user.name"])
            .context("failed to read git user.name")?
            .trim()
            .to_string();

        let user_email = git_output(&config.repo_root, ["config", "user.email"])
            .context("failed to read git user.email")?
            .trim()
            .to_string();

        if user_name.is_empty() || user_email.is_empty() {
            return Err(anyhow!("git user.name and user.email must be configured"));
        }

        let recent_commits = recent_commits(
            &config.repo_root,
            &user_name,
            &user_email,
            config.commit_limit,
        )?;
        let tracked_changes = tracked_changes(&config.repo_root, config.max_diff_tokens)?;
        let instructions =
            load_root_instructions(&config.repo_root, config.max_instructions_tokens)?;

        Ok(Self {
            repo_root: config.repo_root.display().to_string(),
            recent_commits,
            tracked_changes,
            instructions,
        })
    }

    pub fn has_staged_changes(config: &Config) -> Result<bool> {
        let output = Command::new("git")
            .current_dir(&config.repo_root)
            .args(["diff", "--cached", "--quiet", "--", "."])
            .output()
            .context("failed to execute git command")?;

        Ok(!output.status.success())
    }
}

fn recent_commits(
    repo_root: &Path,
    user_name: &str,
    user_email: &str,
    limit: usize,
) -> Result<Vec<String>> {
    let log = git_output(
        repo_root,
        recent_commit_log_args(user_name, user_email, limit),
    )?;

    let commits = log
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToOwned::to_owned)
        .collect();

    Ok(commits)
}

fn recent_commit_log_args(user_name: &str, user_email: &str, limit: usize) -> Vec<String> {
    vec![
        "log".to_string(),
        format!("--author={user_name}"),
        format!("--author={user_email}"),
        "--regexp-ignore-case".to_string(),
        format!("-n{limit}"),
        "--pretty=format:%s".to_string(),
    ]
}

fn tracked_changes(repo_root: &Path, max_tokens: usize) -> Result<String> {
    let staged = git_output(repo_root, ["diff", "--cached", "--", "."])?;
    let unstaged = git_output(repo_root, ["diff", "--", "."])?;
    let summary = git_output(repo_root, ["status", "--short", "--untracked-files=no"])?;

    let combined =
        format!("STATUS\n{summary}\n\nSTAGED DIFF\n{staged}\n\nUNSTAGED DIFF\n{unstaged}");

    Ok(truncate_with_notice(&combined, max_tokens))
}

pub(crate) fn load_root_instructions(
    repo_root: &Path,
    max_tokens: usize,
) -> Result<Option<String>> {
    let claude = repo_root.join("CLAUDE.md");
    if claude.is_file() {
        let content = std::fs::read_to_string(&claude)
            .with_context(|| format!("failed to read {}", claude.display()))?;
        return Ok(Some(truncate_with_notice(&content, max_tokens)));
    }

    let agents = repo_root.join("AGENTS.md");
    if agents.is_file() {
        let content = std::fs::read_to_string(&agents)
            .with_context(|| format!("failed to read {}", agents.display()))?;
        return Ok(Some(truncate_with_notice(&content, max_tokens)));
    }

    Ok(None)
}

fn truncate_with_notice(value: &str, max_tokens: usize) -> String {
    let tokens = token_chunks(value);

    if tokens.len() <= max_tokens {
        return value.to_string();
    }

    let truncated: String = tokens.into_iter().take(max_tokens).collect();
    format!("{truncated}\n\n[truncated]")
}

fn token_chunks(value: &str) -> Vec<String> {
    tokenizer()
        .split_by_token_ordinary(value)
        .unwrap_or_else(|_| vec![value.to_string()])
}

fn tokenizer() -> &'static CoreBPE {
    static TOKENIZER: OnceLock<CoreBPE> = OnceLock::new();
    TOKENIZER.get_or_init(|| cl100k_base().expect("failed to initialize cl100k_base tokenizer"))
}

fn git_output<I, S>(repo_root: &Path, args: I) -> Result<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut command = Command::new("git");
    command.current_dir(repo_root);

    for arg in args {
        command.arg(arg.as_ref());
    }

    let output = command.output().context("failed to execute git command")?;
    if output.status.success() {
        return Ok(String::from_utf8_lossy(&output.stdout).into_owned());
    }

    Err(anyhow!(
        String::from_utf8_lossy(&output.stderr).trim().to_string()
    ))
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::process::Command;

    use tempfile::tempdir;

    use crate::config::Config;

    use super::{
        RepositoryContext, load_root_instructions, recent_commit_log_args, token_chunks,
        truncate_with_notice,
    };

    #[test]
    fn keeps_short_text_unchanged() {
        assert_eq!(truncate_with_notice("abc", 10), "abc");
    }

    #[test]
    fn truncates_by_tokens_not_characters() {
        assert_eq!(token_chunks("hello hello hello").len(), 3);
        assert_eq!(
            truncate_with_notice("hello hello hello", 2),
            "hello hello\n\n[truncated]"
        );
    }

    #[test]
    fn builds_recent_commit_log_args_with_separate_author_filters() {
        assert_eq!(
            recent_commit_log_args("Mateus Junior", "mateus@matj.dev", 3),
            vec![
                "log".to_string(),
                "--author=Mateus Junior".to_string(),
                "--author=mateus@matj.dev".to_string(),
                "--regexp-ignore-case".to_string(),
                "-n3".to_string(),
                "--pretty=format:%s".to_string(),
            ]
        );
    }

    #[test]
    fn detects_when_no_staged_changes_exist() {
        let temp = tempdir().unwrap();

        Command::new("git")
            .args(["init"])
            .current_dir(temp.path())
            .output()
            .unwrap();
        Command::new("git")
            .args(["config", "user.name", "Test User"])
            .current_dir(temp.path())
            .output()
            .unwrap();
        Command::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(temp.path())
            .output()
            .unwrap();
        fs::write(temp.path().join("file.txt"), "hello").unwrap();

        let config = Config {
            repo_root: temp.path().to_path_buf(),
            host: String::new(),
            api_key: String::new(),
            model: String::new(),
            commit_limit: 1,
            max_diff_tokens: 1,
            max_instructions_tokens: 1,
            dry_run: false,
        };

        assert!(!RepositoryContext::has_staged_changes(&config).unwrap());
    }

    #[test]
    fn prefers_claude_over_agents() {
        let temp = tempdir().unwrap();
        fs::write(temp.path().join("CLAUDE.md"), "claude rules").unwrap();
        fs::write(temp.path().join("AGENTS.md"), "agents rules").unwrap();

        assert_eq!(
            load_root_instructions(temp.path(), 100).unwrap(),
            Some("claude rules".to_string())
        );
    }

    #[test]
    fn falls_back_to_agents_when_claude_missing() {
        let temp = tempdir().unwrap();
        fs::write(temp.path().join("AGENTS.md"), "agents rules").unwrap();

        assert_eq!(
            load_root_instructions(temp.path(), 100).unwrap(),
            Some("agents rules".to_string())
        );
    }
}
