use crate::{config, git::RepositoryContext};

const DEFAULT_PROMPT_TEMPLATE: &str = include_str!("prompt.md");

pub fn build_prompt(context: &RepositoryContext) -> String {
    let template = load_prompt_template();
    let commit_history = if context.recent_commits.is_empty() {
        "- No recent commits found for the current git user.".to_string()
    } else {
        context
            .recent_commits
            .iter()
            .map(|commit| format!("- {commit}"))
            .collect::<Vec<_>>()
            .join("\n")
    };

    let instructions = context
        .instructions
        .as_deref()
        .unwrap_or("No CLAUDE.md or AGENTS.md file found at repository root.");

    render_prompt(
        &template,
        &context.repo_root,
        &commit_history,
        instructions,
        &context.tracked_changes,
    )
}

fn load_prompt_template() -> String {
    if let Some(template) = config::config_file_path()
        .ok()
        .and_then(|path| config::config_dir_from_file_path(&path))
        .and_then(|path| load_prompt_template_from_config_dir(&path))
    {
        return template;
    }

    DEFAULT_PROMPT_TEMPLATE.to_string()
}

fn load_prompt_template_from_config_dir(config_dir: &std::path::Path) -> Option<String> {
    let prompt_path = config_dir.join("PROMPT.md");

    if !prompt_path.is_file() {
        return None;
    }

    std::fs::read_to_string(&prompt_path).ok()
}

fn render_prompt(
    template: &str,
    repo_root: &str,
    commit_history: &str,
    instructions: &str,
    tracked_changes: &str,
) -> String {
    template
        .replace("{repo_root}", repo_root)
        .replace("{commit_history}", commit_history)
        .replace("{instructions}", instructions)
        .replace("{tracked_changes}", tracked_changes)
}

#[cfg(test)]
mod tests {
    use super::build_prompt;
    use crate::git::RepositoryContext;
    use std::fs;

    use tempfile::tempdir;

    use super::{DEFAULT_PROMPT_TEMPLATE, load_prompt_template_from_config_dir, render_prompt};

    #[test]
    fn includes_expected_sections() {
        let prompt = build_prompt(&RepositoryContext {
            repo_root: "repo".to_string(),
            recent_commits: vec!["feat: add cli".to_string()],
            tracked_changes: "diff body".to_string(),
            instructions: Some("follow rules".to_string()),
        });

        assert!(prompt.contains("Recent commits by this user"));
        assert!(prompt.contains("feat: add cli"));
        assert!(prompt.contains("follow rules"));
        assert!(prompt.contains("diff body"));
    }

    #[test]
    fn uses_embedded_template_when_override_is_missing() {
        let temp = tempdir().unwrap();

        assert_eq!(load_prompt_template_from_config_dir(temp.path()), None);
        assert!(DEFAULT_PROMPT_TEMPLATE.contains("You are generating a git commit message"));
    }

    #[test]
    fn loads_prompt_override_from_config_dir() {
        let temp = tempdir().unwrap();
        fs::create_dir_all(temp.path().join("ai-commit")).unwrap();
        fs::write(
            temp.path().join("ai-commit").join("PROMPT.md"),
            "custom prompt {repo_root}",
        )
        .unwrap();

        assert_eq!(
            load_prompt_template_from_config_dir(&temp.path().join("ai-commit")),
            Some("custom prompt {repo_root}".to_string())
        );
    }

    #[test]
    fn renders_prompt_with_template_values() {
        let prompt = render_prompt(
            DEFAULT_PROMPT_TEMPLATE,
            "repo",
            "feat: add cli",
            "follow rules",
            "diff body",
        );

        assert!(prompt.contains("Repository root: repo"));
        assert!(prompt.contains("feat: add cli"));
        assert!(prompt.contains("follow rules"));
        assert!(prompt.contains("diff body"));
    }
}
