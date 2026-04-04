use clap::Parser;

#[derive(Debug, Parser)]
#[command(
    name = "ai-commit",
    version,
    about = "Generate semantic commit messages from repository context"
)]
pub struct Cli {
    #[arg(long, env = "OPENAI_HOST")]
    pub host: Option<String>,

    #[arg(long, env = "OPENAI_KEY")]
    pub api_key: Option<String>,

    #[arg(long, env = "OPENAI_MODEL")]
    pub model: Option<String>,

    #[arg(long, default_value_t = 30)]
    pub commit_limit: usize,

    #[arg(long, default_value_t = 20_000)]
    pub max_diff_chars: usize,

    #[arg(long, default_value_t = 8_000)]
    pub max_instructions_chars: usize,
}

impl Cli {
    pub fn parse_args() -> Self {
        Self::parse()
    }
}
