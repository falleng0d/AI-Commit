use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, anyhow};

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
        let tracked_changes = tracked_changes(&config.repo_root, config.max_diff_chars)?;
        let instructions =
            load_root_instructions(&config.repo_root, config.max_instructions_chars)?;

        Ok(Self {
            repo_root: config.repo_root.display().to_string(),
            recent_commits,
            tracked_changes,
            instructions,
        })
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
        [
            "log",
            &format!("--author={user_name}|{user_email}"),
            "--regexp-ignore-case",
            &format!("-n{limit}"),
            "--pretty=format:%s",
        ],
    )?;

    let commits = log
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToOwned::to_owned)
        .collect();

    Ok(commits)
}

fn tracked_changes(repo_root: &Path, max_chars: usize) -> Result<String> {
    let staged = git_output(repo_root, ["diff", "--cached", "--", "."])?;
    let unstaged = git_output(repo_root, ["diff", "--", "."])?;
    let summary = git_output(repo_root, ["status", "--short", "--untracked-files=no"])?;

    let combined =
        format!("STATUS\n{summary}\n\nSTAGED DIFF\n{staged}\n\nUNSTAGED DIFF\n{unstaged}");

    Ok(truncate_with_notice(&combined, max_chars))
}

fn load_root_instructions(repo_root: &Path, max_chars: usize) -> Result<Option<String>> {
    let claude = repo_root.join("CLAUDE.md");
    if claude.is_file() {
        let content = std::fs::read_to_string(&claude)
            .with_context(|| format!("failed to read {}", claude.display()))?;
        return Ok(Some(truncate_with_notice(&content, max_chars)));
    }

    let agents = repo_root.join("AGENTS.md");
    if agents.is_file() {
        let content = std::fs::read_to_string(&agents)
            .with_context(|| format!("failed to read {}", agents.display()))?;
        return Ok(Some(truncate_with_notice(&content, max_chars)));
    }

    Ok(None)
}

fn truncate_with_notice(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        return value.to_string();
    }

    let truncated: String = value.chars().take(max_chars).collect();
    format!("{truncated}\n\n[truncated]")
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
    use super::truncate_with_notice;

    #[test]
    fn keeps_short_text_unchanged() {
        assert_eq!(truncate_with_notice("abc", 10), "abc");
    }

    #[test]
    fn truncates_long_text_with_marker() {
        assert_eq!(truncate_with_notice("abcdef", 4), "abcd\n\n[truncated]");
    }
}
