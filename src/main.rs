use clap::Parser;
use std::io::Write;
use std::{env, io};
use colored::Colorize;

mod commit_formatter;
mod git;
mod ai;

use commit_formatter::CommitFormatter;
use git::GitClient;
use ai::AIClient;

#[derive(Parser)]
#[command(name = "iamcommitted")]
#[command(author = "Your Name")]
#[command(version = "1.0")]
#[command(about = "A small CLI used for generating Git commit messages", long_about = None)]
struct Cli {}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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

    let api_key = env::var("OPENAI_API_KEY").map_err(|_| {
        "Error: OPENAI_API_KEY environment variable is not set. Please set this environment variable with your OpenAI API key to use this application."
    })?;

    let git_client = GitClient::new();
    let ai_client = AIClient::new(api_key)?;

    if !git_client.has_staged_changes()? {
        println!("\n{} No staged changes found.", "!".yellow());
        println!("\n  Please stage your changes using 'git add' before running this command.\n");
        return Ok(());
    }

    // Print the staged files
    println!("\nStaged files:");
    println!("{}", git_client.get_staged_files()?);

    println!("\nGenerated Conventional Commit ...");

    // Get the full diff for AI processing
    let diff = git_client.get_staged_changes()?;
    
    // Generate commit message using AI
    let raw_message = ai_client.generate_commit_message(&diff).await?;
    
    // Format the commit message
    let formatter = CommitFormatter::new(raw_message);
    let formatted_commit = formatter.format();
    let commit_message = formatted_commit.to_string();

    println!("\n{}", commit_message.color("grey"));
    println!("\nPlease select an option:");
    println!("1. Commit changes with this message");
    println!("2. Cancel");

    print!("\nEnter your choice (1-2): ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read line");

    let num_result = input.trim().parse::<u32>();

    if num_result.is_err() {
        println!("\n{} Please enter a valid number (1-2)\n", "❌".red());
        return Ok(());
    }

    let num = num_result.unwrap();
    if num == 1 {
        println!("\nCommitting changes to Git...");
        let output = git_client.commit(&commit_message)?;

        if !output.stdout.is_empty() {
            match String::from_utf8(output.stdout) {
                Ok(stdout_str) => println!("{}", stdout_str.trim()),
                Err(_) => println!("Output contains non-UTF8 characters"),
            }
        }

        if !output.stderr.is_empty() {
            match String::from_utf8(output.stderr) {
                Ok(stderr_str) => println!("{}", stderr_str.trim()),
                Err(_) => println!("Error output contains non-UTF8 characters"),
            }
        }

        if output.status.success() {
            println!("\n{} You are successfully committed!\n", "✔".green());
        } else {
            println!(
                "{} {}",
                "Failed to commit changes. Exit status:".red(),
                output.status
            );
        }
    } else {
        println!("\nCommit cancelled\n");
    }

    Ok(())
}
