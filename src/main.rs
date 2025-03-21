use clap::Parser;
use std::io::Write;
use std::{env, io, process::Command};
use colored::Colorize;
use log::{info, warn};

fn setup_logging() -> Result<(), Box<dyn std::error::Error>> {
    let home = env::var("HOME").expect("Failed to get HOME directory");
    let log_dir = format!("{}/.iamcommitted/logs", home);
    std::fs::create_dir_all(&log_dir)?;
    
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}] [{}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Info)
        .chain(fern::log_file(format!("{}/chatgpt_interactions.log", log_dir))?)
        .apply()?;
    Ok(())
}

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
    setup_logging()?;
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
    println!("\n{}", "🔍 Analyzing Changes...".blue());
    println!("-----------------------------------------");

    let api_key = env::var("OPENAI_API_KEY").map_err(|_| {
        "Error: OPENAI_API_KEY environment variable is not set. Please set this environment variable with your OpenAI API key to use this application."
    })?;

    let git_client = GitClient::new();
    let ai_client = AIClient::new(api_key)?;

    if !git_client.has_staged_changes()? {
        warn!("No staged changes found.");
        println!("\n{} No staged changes found.", "!".yellow());
        println!("\n  Please stage your changes using 'git add' before running this command.\n");
        return Ok(());
    }

    // Print the staged files
    println!("📂 Staged Files:");
    let staged_files = git_client.get_staged_files()?;
    for file in staged_files.lines() {
        println!("   - {}", file);
    }
    println!("-----------------------------------------");

    // Get the full diff for AI processing
    let diff = git_client.get_staged_changes()?;
    
    // Generate commit message using AI
    let raw_message = ai_client.generate_commit_message(&diff).await?;
    
    // Format the commit message
    let formatter = CommitFormatter::new(raw_message.clone());
    let formatted_commit = formatter.format();
    let commit_message = formatted_commit.to_string();

    // Generate alternative suggestions
    //TODO: There is not implementation here for alternative commit messages
    // let alt1 = CommitFormatter::new(format!("feat({}): {}", 
    //     commit_message.split('(').nth(1).unwrap_or("").split(')').next().unwrap_or(""),
    //     "remove backticks from commit messages")).format();
    // let alt2 = CommitFormatter::new(format!("refactor({}): standardize commit message formatting", 
    //     commit_message.split('(').nth(1).unwrap_or("").split(')').next().unwrap_or(""))).format();

    println!("\n📝 Suggested Commit Message:");
    println!("---------------------------------------------------");
    println!("{}", commit_message);
    println!("---------------------------------------------------");

    //TODO: This needs to be implemented
    // println!("\n💡 Alternative Suggestions:");
    // println!("1️⃣ {}", alt1.to_string());
    // println!("2️⃣ {}", alt2.to_string());
    // println!("3️⃣ custom: Edit the message manually");
    
    println!("\nPlease select an option:");
    println!("[1] Use the suggested message ✅ (default)");
    // println!("[2] Choose an alternative");
    println!("[2] Edit the message manually");
    println!("[3] Cancel");

    print!("\nEnter your choice (1-4): ⌨️  ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read line");

    let num_result = input.trim().parse::<u32>();

    if num_result.is_err() {
        println!("\n{} Please enter a valid number (1-3)\n", "❌".red());
        return Ok(());
    }

    let num = num_result.unwrap();
    let final_message = match num {
        1 => commit_message,
        // 2 => {
        //     println!("\nChoose an alternative (1-2):");
        //     println!("[1] {}", alt1.to_string());
        //     println!("[2] {}", alt2.to_string());
            
        //     print!("\nEnter your choice: ");
        //     io::stdout().flush().unwrap();
            
        //     let mut alt_input = String::new();
        //     io::stdin().read_line(&mut alt_input).expect("Failed to read line");
            
        //     match alt_input.trim().parse::<u32>() {
        //         Ok(1) => alt1.to_string(),
        //         Ok(2) => alt2.to_string(),
        //         _ => {
        //             println!("\nInvalid choice, using original message");
        //             commit_message
        //         }
        //     }
        // }
        2 => {
            // Edit commit message using nano
            use std::fs;
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
            formatted_commit.to_string()
        }
        _ => {
            println!("\nCommit cancelled\n");
            return Ok(());
        }
    };

    // Proceed with commit if a message was selected/edited
    if num <= 2 {
        git_client.commit_with_details(&final_message).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
    }

    Ok(())
}
