mod cli;
mod config;
mod git;
mod prompt;
mod provider;

use anyhow::Result;

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
    let repo = git::RepositoryContext::gather(&config)?;
    let prompt = prompt::build_prompt(&repo);
    let client = provider::AiClient::new(&config)?;
    let message = client.generate_commit_message(&prompt)?;
    println!("{}", message.trim());
    Ok(())
}
