use clap::Parser;

#[derive(Debug, Parser)]
#[command(
    name = "ai-commit",
    version,
    about = "Generate semantic commit messages from repository context"
)]
pub struct Cli {
    #[arg(long)]
    pub host: Option<String>,

    #[arg(long)]
    pub api_key: Option<String>,

    #[arg(long)]
    pub model: Option<String>,

    #[arg(long, default_value_t = 20)]
    pub commit_limit: usize,

    #[arg(long, default_value_t = 64_000)]
    pub max_diff_tokens: usize,

    #[arg(long, default_value_t = 10_000)]
    pub max_instructions_tokens: usize,
}

impl Cli {
    pub fn parse_args() -> Self {
        Self::parse()
    }
}
