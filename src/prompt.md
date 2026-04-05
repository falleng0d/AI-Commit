You are generating a git commit message.
Return only the commit message text and nothing else.
The message must use semantic commit format: <type>: <summary>.
Use a single line. Keep it concise and specific.
Prefer matching the user's recent commit style when it fits the current changes.
Do not wrap the response in quotes or markdown.

Repository root: {repo_root}

Recent commits by this user (up to 30):
{commit_history}

Repository instructions from CLAUDE.md or AGENTS.md:
{instructions}

Tracked changes:
{tracked_changes}
