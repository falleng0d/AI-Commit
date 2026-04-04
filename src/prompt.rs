use crate::git::RepositoryContext;

pub fn build_prompt(context: &RepositoryContext) -> String {
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

    format!(
        concat!(
            "You are generating a git commit message.\n",
            "Return only the commit message text and nothing else.\n",
            "The message must use semantic commit format: <type>: <summary>.\n",
            "Use a single line. Keep it concise and specific.\n",
            "Prefer matching the user's recent commit style when it fits the current changes.\n",
            "Do not wrap the response in quotes or markdown.\n\n",
            "Repository root: {repo_root}\n\n",
            "Recent commits by this user (up to 30):\n{commit_history}\n\n",
            "Repository instructions from CLAUDE.md or AGENTS.md:\n{instructions}\n\n",
            "Tracked changes:\n{tracked_changes}\n"
        ),
        repo_root = context.repo_root,
        commit_history = commit_history,
        instructions = instructions,
        tracked_changes = context.tracked_changes,
    )
}

#[cfg(test)]
mod tests {
    use super::build_prompt;
    use crate::git::RepositoryContext;

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
}
