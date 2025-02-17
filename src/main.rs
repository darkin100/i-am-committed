use clap::Parser;
use std::io::{stdout, Write};
use std::process::Command;
use openai_api_rs::v1::api::OpenAIClient;
use openai_api_rs::v1::chat_completion::{self, ChatCompletionRequest};
use openai_api_rs::v1::common::GPT4_O_MINI;
use std::{env, io};
extern crate termion;

mod commit_formatter;
use commit_formatter::CommitFormatter;


#[derive(Parser)]
#[command(name = "iamcommitted")]
#[command(author = "Your Name")]
#[command(version = "1.0")]
#[command(about = "A command line application", long_about = None)]
struct Cli {
    
    

    
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>  {
    
    //TODO: Pull this out to be a parameter/environment variable
    let api_key = env::var("OPENAI_API_KEY").unwrap().to_string();
    
    //Get the git diff message
    let mut git = Command::new("git");
        
    let output = git.arg("diff").arg("--cached").arg("--diff-algorithm=minimal").output().expect("process failed to execute");
    
    if output.stdout.is_empty(){
        println!("No changes to commit");
        return Ok(());
    }

    // Convert stdout bytes to a String
    let stdout_str = String::from_utf8_lossy(&output.stdout);
    
    let client = OpenAIClient::builder().with_api_key(api_key).build()?;

    let mut chat_message = String::from(
        "Generate a commit message following the Conventional Commits specification. \
        Use one of these types: feat, fix, chore, docs, style, refactor, perf, test, build, ci, revert. \
        Include a scope in parentheses if relevant. \
        If this is a breaking change, include BREAKING CHANGE: in the footer. \
        Example format: \
        type(scope): description\n\n[optional body]\n\n[optional footer]\n\n\
        Here are the changes to commit:"
    );
    chat_message.push_str(stdout_str.as_ref());

    let req = ChatCompletionRequest::new(
        GPT4_O_MINI.to_string(),//TODO: Pull this out to be a parameter
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
    
    println!("\nGenerated Conventional Commit:");
    println!("{}", commit_message);
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
        println!("Please enter a valid number (1 or 2)");
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
                println!("Successfully committed changes!");
            } else {
                println!("Failed to commit changes. Exit status: {}", output.status);
            }


        } else {
            println!("\nCommit cancelled");
            return Ok(());
        }
    }

    Ok(())

}
