use clap::{Parser, Subcommand};
use colored::Colorize;
use log::{info, warn};
use std::fs;
use std::io::Write;
use std::{env, io, process::Command};

fn setup_logging(verbose: bool) -> Result<(), Box<dyn std::error::Error>> {
    let log_dir = Config::get_log_dir()?;
    std::fs::create_dir_all(&log_dir)?;

    let dispatch = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}] [{}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Info)
        .chain(fern::log_file(log_dir.join("chatgpt_interactions.log"))?);

    // If verbose mode is enabled, also log to stdout
    if verbose {
        dispatch.chain(std::io::stdout()).apply()?;
    } else {
        dispatch.apply()?;
    }

    Ok(())
}

mod ai;
mod commit_formatter;
mod config;
mod git;

use crate::ai::AIClient;
use crate::commit_formatter::CommitFormatter;
use crate::config::Config;
use crate::git::GitClient;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Enable verbose mode to print logs to console as well
    #[arg(long = "verbose", short = 'v')]
    verbose: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Prepares the commit message (for git prepare-commit-msg hook)
    PrepareCommitMsg {
        /// Path to the commit message file
        #[arg(index = 1)]
        commit_msg_file_path: String,

        /// Source of the commit message (e.g., message, template, merge, squash, commit)
        #[arg(index = 2, required = false)]
        commit_source: Option<String>,

        /// Commit SHA-1 (if applicable, e.g., for amends)
        #[arg(index = 3, required = false)]
        commit_sha1: Option<String>,
    },
}

async fn generate_formatted_commit_message(
    git_client: &GitClient,
    ai_client: &AIClient,
) -> Result<String, Box<dyn std::error::Error>> {
    // Get the full diff for AI processing
    let diff = git_client.get_staged_changes()?;
    info!(
        "Retrieved diff for AI processing (first 500 chars):\n{}",
        diff.chars().take(500).collect::<String>()
    );

    if diff.trim().is_empty() {
        let staged_files_list = git_client.get_staged_files()?;
        if staged_files_list.trim().is_empty() {
            warn!("Diff is empty and no staged files. AI will process an empty context.");
        } else {
            warn!("Diff is empty, but staged files are present (e.g. mode changes, new empty files). AI will process based on file list if prompt supports it.");
        }
    }

    // Generate commit message using AI
    let raw_message = ai_client.generate_commit_message(&diff).await?;
    info!("Raw AI-generated message: {}", raw_message);

    // Format the commit message
    let formatter = CommitFormatter::new(raw_message.clone());
    let formatted_commit = formatter.format();
    let final_message = formatted_commit.to_string();
    info!("Formatted commit message: {}", final_message);

    Ok(final_message)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    const VERSION: &str = env!("CARGO_PKG_VERSION");

    let cli = Cli::parse();

    // Set up logging with verbose flag if provided
    setup_logging(cli.verbose)?;

    if cli.verbose {
        println!("Verbose mode enabled. Logs will be printed to console.");
    }

    match cli.command {
        Some(Commands::PrepareCommitMsg {
            commit_msg_file_path,
            commit_source,
            commit_sha1,
        }) => {
            info!("Running in prepare-commit-msg hook mode.");
            info!("Commit message file: {}", commit_msg_file_path);
            if let Some(source) = &commit_source {
                info!("Commit source: {}", source);
                // If the user is providing a message via -m or -F, or using a template,
                // we should not overwrite it with an AI-generated one.
                if source == "message" || source == "template" {
                    info!(
                        "Commit source is '{}', skipping AI message generation.",
                        source
                    );
                    return Ok(());
                }
            }
            if let Some(sha1) = &commit_sha1 {
                info!("Commit SHA1: {}", sha1);
            }

            // Check for API key - IAC_OPENAI_API_KEY takes precedence over OPENAI_API_KEY
            let api_key = env::var("IAC_OPENAI_API_KEY")
                .or_else(|_| env::var("OPENAI_API_KEY"))
                .map_err(|_| {
                    "Error: Neither IAC_OPENAI_API_KEY nor OPENAI_API_KEY environment variable is set for prepare-commit-msg hook."
                })?;
            let git_client = GitClient::new();
            let config = Config::new()?;
            let ai_client = AIClient::new(api_key, config)?;

            // Check for staged changes. Even if none, AI might generate a message for an empty commit if allowed.
            if !git_client.has_staged_changes()? {
                warn!("No staged changes detected by git_client.has_staged_changes() in hook mode. Proceeding to generate message based on (likely empty) diff.");
            }

            match generate_formatted_commit_message(&git_client, &ai_client).await {
                Ok(commit_message_content) => {
                    fs::write(&commit_msg_file_path, &commit_message_content)?;
                    info!(
                        "Successfully wrote AI-generated commit message to {}",
                        commit_msg_file_path
                    );
                }
                Err(e) => {
                    eprintln!("Error generating commit message for hook: {}", e);
                    // Propagate the error to potentially halt the commit process
                    return Err(e);
                }
            }
            Ok(())
        }
        None => {
            // Interactive mode (original behavior)
            println!(
                "{}",
                r#"
    ____               _____                 _ __  __         __
    /  _/ ___ ___ _    / ___/__  __ _  __ _  (_) /_/ /____ ___/ /
   _/ /  / _ `/  ' \  / /__/ _ \/  ' \/  ' \/ / __/ __/ -_) _  /
  /___/  \_,_/_/_/_/  \___/\___/_/_/_/_/_/_/_/\__/\__/\__/\_,_/
                                                                 
      "#
                .green()
            );
            // Check for API key - IAC_OPENAI_API_KEY takes precedence over OPENAI_API_KEY
            let api_key = env::var("IAC_OPENAI_API_KEY")
                .or_else(|_| env::var("OPENAI_API_KEY"))
                .map_err(|_| {
                    "Error: Neither IAC_OPENAI_API_KEY nor OPENAI_API_KEY environment variable is set. Please set one of these environment variables with your OpenAI API key to use this application."
                })?;

            let git_client = GitClient::new();
            let config = Config::new()?;
            let ai_client = AIClient::new(api_key, config)?;

            println!("v{} | Model: {}", VERSION, ai_client.get_model());
            println!("\n{}", "🔍 Analysing Changes...".blue());
            println!("-----------------------------------------");

            if !git_client.has_staged_changes()? {
                warn!("No staged changes found.");
                println!("\n{} No staged changes found.", "!".yellow());
                println!(
                    "\n  Please stage your changes using 'git add' before running this command.\n"
                );
                return Ok(());
            }

            // Print the staged files
            println!("📂 Staged Files:");
            let staged_files = git_client.get_staged_files()?;
            for file in staged_files.lines() {
                println!("   - {}", file);
            }
            println!("-----------------------------------------");

            let commit_message = generate_formatted_commit_message(&git_client, &ai_client).await?;

            println!("\n📝 Suggested Commit Message:");
            println!("---------------------------------------------------");
            println!("{}", commit_message);
            println!("---------------------------------------------------");

            println!("\nPlease select an option:");
            println!("[1] Use the suggested message ✅ (default)");
            println!("[2] Edit the message manually");
            println!("[3] Cancel");

            print!("\nEnter your choice (1-3): ⌨️  ");
            io::stdout().flush().unwrap();

            let mut input = String::new();
            io::stdin()
                .read_line(&mut input)
                .expect("Failed to read line");

            let num_result = input.trim().parse::<u32>();

            if num_result.is_err() {
                println!("\n{} Please enter a valid number (1-3)\n", "❌".red());
                return Ok(());
            }

            let num = num_result.unwrap();
            let final_message = match num {
                1 => commit_message,
                2 => {
                    // Edit commit message using nano
                    // Note: std::fs is already imported at the top level
                    use tempfile::NamedTempFile; // Keep this local as it's specific to this block

                    let mut temp_file = NamedTempFile::new()?;
                    write!(temp_file, "{}", commit_message)?;
                    temp_file.flush()?;

                    let status = Command::new("nano")
                        .arg(temp_file.path())
                        .status()
                        .expect("Failed to open nano");

                    if !status.success() {
                        println!("\nFailed to edit commit message using nano");
                        return Ok(());
                    }

                    let edited_message = fs::read_to_string(temp_file.path())?;

                    let formatter = CommitFormatter::new(edited_message);
                    let formatted_commit = formatter.format();
                    formatted_commit.to_string()
                }
                _ => {
                    println!("\nCommit cancelled\n");
                    return Ok(());
                }
            };

            if num <= 2 {
                // Only commit if option 1 or 2 was chosen
                git_client
                    .commit_with_details(&final_message)
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
            }
            Ok(())
        }
    }
}
