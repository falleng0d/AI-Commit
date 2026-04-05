mod cli;
mod config;
mod git;
mod prompt;
mod provider;

use anyhow::{Result, anyhow};

use crate::cli::Cli;

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse_args();
    let config = config::Config::load(cli)?;
    if !git::RepositoryContext::has_staged_changes(&config)? {
        return Err(anyhow!("no staged changes to commit"));
    }

    let repo = git::RepositoryContext::gather(&config)?;
    let prompt_text = prompt::build_prompt(&repo);

    if config.dry_run {
        let (_, summary) = prompt::build_prompt_with_summary(&repo);
        println!("{}", prompt::format_dry_run_output(&prompt_text, &summary));
        return Ok(());
    }

    let client = provider::AiClient::new(&config)?;
    let message = client.generate_commit_message(&prompt_text)?;
    println!("{}", message.trim());
    Ok(())
}
