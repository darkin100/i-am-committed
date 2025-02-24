use clap::Parser;
use std::io::Write;
use std::process::Command;
use openai_api_rs::v1::api::OpenAIClient;
use openai_api_rs::v1::chat_completion::{self, ChatCompletionRequest};
use openai_api_rs::v1::common::GPT4_O_MINI;
use std::{env, io};
extern crate termion;
use colored::Colorize;

mod commit_formatter;
use commit_formatter::CommitFormatter;


#[derive(Parser)]
#[command(name = "iamcommitted")]
#[command(author = "Your Name")]
#[command(version = "1.0")]
#[command(about = "A small CLI used for generating Git commit messages", long_about = None)]
struct Cli {
    
    

    
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>  {
    
    println!("{}", r#"
    ____               _____                 _ __  __         __
    /  _/ ___ ___ _    / ___/__  __ _  __ _  (_) /_/ /____ ___/ /
   _/ /  / _ `/  ' \  / /__/ _ \/  ' \/  ' \/ / __/ __/ -_) _  / 
  /___/  \_,_/_/_/_/  \___/\___/_/_/_/_/_/_/_/\__/\__/\__/\_,_/  
                                                                 
      "#.green());

    let api_key = env::var("OPENAI_API_KEY")
        .map_err(|_| "Error: OPENAI_API_KEY environment variable is not set. Please set this environment variable with your OpenAI API key to use this application.")?
        .to_string();
    
    let mut git = Command::new("git");
        
    let output = git.arg("diff").arg("--cached").arg("--diff-algorithm=minimal").output().expect("process failed to execute");
    
    if output.stdout.is_empty(){
        println!("\n{} No staged changes found.","!".yellow());
        println!("  Please stage your changes using 'git add' before running this command.\n");
        return Ok(());
    }

    println!("\nGenerated Conventional Commit ...");

    // Convert stdout bytes to a String
    let stdout_str = String::from_utf8_lossy(&output.stdout);
    
    let client = OpenAIClient::builder().with_api_key(api_key).build()?;

    let mut chat_message = String::from(
        "Generate a commit message following the Conventional Commits specification. \
        Use one of these types: feat, fix, chore, docs, style, refactor, perf, test, build, ci, revert. \
        Include a scope in parentheses if relevant. \
        Example format: \
        type(scope): description\n\n[optional body]
        Here are the changes to commit:"
    );
    chat_message.push_str(stdout_str.as_ref());

    let req = ChatCompletionRequest::new(
        GPT4_O_MINI.to_string(),
        vec![chat_completion::ChatCompletionMessage {
            role: chat_completion::MessageRole::user,
            content: chat_completion::Content::Text(chat_message),
            name: None,
            tool_calls: None,
            tool_call_id: None,
        }],
    );

    let result = client.chat_completion(req).await?;
    
    let content = result.choices[0].message.content.clone();
    let raw_message = content.unwrap();
    
    // Format the commit message using our CommitFormatter
    let formatter = CommitFormatter::new(raw_message);
    let formatted_commit = formatter.format();
    let commit_message = formatted_commit.to_string();
    
    println!("\n{}", commit_message.color("grey"));
    println!("\nPlease select an option:");
    println!("1. Commit changes with this message");
    println!("2. Cancel");    

    print!("\nEnter your choice (1-2): ");
    io::stdout().flush().unwrap(); // Ensure the prompt is displayed before reading input
    
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");

        // Parse and validate the input
      let num_result=  input.trim().parse::<u32>();
    
    if num_result.is_err() {
        println!("\n{} Please enter a valid number (1-2)\n","❌".red());
        return Ok(());
    } else {
        let num = num_result.unwrap();
        
        if num == 1 {
            println!("\nCommitting changes to Git...");
            // Create a new git command for committing
            let output = Command::new("git")
                        .arg("commit")
                        .arg("-m")
                        .arg(&commit_message)
                        .output()
                        .expect("process failed to execute");

            // Print both stdout and stderr
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
                println!("\n{} You are successfully committed!\n","✔".green());
            } else {
                println!("{} {}","Failed to commit changes. Exit status:".red(), output.status);
            }


        } else {
            println!("\nCommit cancelled\n");
            return Ok(());
        }
    }

    Ok(())

}
