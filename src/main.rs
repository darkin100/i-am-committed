use clap::Parser;
use std::io::{stdout, Write};
use std::process::Command;
use openai_api_rs::v1::api::OpenAIClient;
use openai_api_rs::v1::chat_completion::{self, ChatCompletionRequest};
use openai_api_rs::v1::common::GPT4_O_MINI;
use std::{env, io};
extern crate termion;


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
    }else{
        //DEBUG Print
        println!("DEBUG:{:?}", &output.stdout);
    }

    // Convert stdout bytes to a String
    let stdout_str = String::from_utf8_lossy(&output.stdout);
    
    let client = OpenAIClient::builder().with_api_key(api_key).build()?;

    let mut chat_message = String::from("Please provide a commit message for these changes:");
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
    let commit_message = content.unwrap();
    println!("{}", commit_message );
    println!("Please select an option:");
    println!("1. Commit Changes with message");
    // println!("2. Edit message");
    println!("2. Cancel");    

    // Read user input
    print!("Enter your choice (1-3): ");
    io::stdout().flush().unwrap(); // Ensure the prompt is displayed before reading input
    
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");

        // Parse and validate the input
      let num_result=  input.trim().parse::<u32>();
    
    if num_result.is_err() {
        println!("Please enter a valid number between 1 and 3");
        return Ok(());
    
    }else{
        let num = num_result.unwrap();
        
        if num == 1{
        
            println!("Committing message to Git");
            let output = git
                        .arg("commit")
                        .arg("-m")
                        .arg(commit_message)
                        .output()
                        .expect("process failed to execute");

            // Check if stdout has any content
            if output.stdout.is_empty() {
                println!("No output in stdout");
            } else {
                println!("Stdout contains {} bytes", output.stdout.len());
                // Convert and print the output
                match String::from_utf8(output.stdout) {
                    Ok(stdout_str) => println!("Output: {}", stdout_str),
                    Err(_) => println!("Output contains non-UTF8 characters"),
                }
            }

            // Also check the status
            println!("Exit status: {}", output.status);


        }else if num == 2{
            
            println!("Commit cancelled");
            return Ok(());

            // println!("{}", commit_message);
            // println!("\r");
            // let mut input = String::new();
            // io::stdin()
            //     .read_line(&mut input)
            //     .expect("Failed to read line");

            // println!("{0}",input.trim());

            
        }else{
            println!("Commit cancelled");
            return Ok(());
        }
    }

    Ok(())

}
