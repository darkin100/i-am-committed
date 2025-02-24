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

fn commit_changes(git_client: &GitClient, commit_message: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("\nCommitting changes to Git...");
    let output = git_client.commit(commit_message)?;

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
    Ok(())
}

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
    println!("2. Edit commit message");
    println!("3. Cancel");

    print!("\nEnter your choice (1-3): ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read line");

    let num_result = input.trim().parse::<u32>();

    if num_result.is_err() {
        println!("\n{} Please enter a valid number (1-3)\n", "❌".red());
        return Ok(());
    }

    let num = num_result.unwrap();
    match num {
        1 => {
            // Proceed with original commit message
            commit_changes(&git_client, &commit_message)?;
        }
        2 => {
            // Edit commit message using nano
            use std::fs;
            use std::process::Command;
            use tempfile::NamedTempFile;
            
            // Create a temporary file with the commit message
            let mut temp_file = NamedTempFile::new()?;
            write!(temp_file, "{}", commit_message)?;
            temp_file.flush()?;
            
            // Open nano to edit the message
            let status = Command::new("nano")
                .arg(temp_file.path())
                .status()
                .expect("Failed to open nano");
                
            if !status.success() {
                println!("\nFailed to edit commit message");
                return Ok(());
            }
            
            // Read back the edited message
            let edited_message = fs::read_to_string(temp_file.path())?;
            
            // Format the edited message
            let formatter = CommitFormatter::new(edited_message);
            let formatted_commit = formatter.format();
            let final_message = formatted_commit.to_string();
            
            println!("\n\nUpdated commit message:");
            println!("{}", final_message.color("grey"));
            
            // Commit with edited message
            commit_changes(&git_client, &final_message)?;
        }
        _ => {
            println!("\nCommit cancelled\n");
        }
    }

    Ok(())
}
