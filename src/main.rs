mod flow;
mod nodes;
mod state;
mod utils;

use pocketflow_rs::Context;
use flow::create_graph_flow;
use state::SharedState;
use serde_json::json;
use std::env;
use std::fs;
use std::io::{self, Read};
use state::{UserSession, UserTier, ChatInput, InputType, AiResponse};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load .env file if present
    let _ = dotenvy::dotenv();
    
    // Initialize SharedState from CLI/environment for CLI demo
    // Arguments:
    //   --user <id> (default: "test")
    //   --tier <free|pro> (default: free)
    //   --credits <u32> (default: 100)
    //   --input-file <path> (optional)
    let args: Vec<String> = env::args().collect();
    let mut user_id = env::var("GF_USER").unwrap_or_else(|_| "test".to_string());
    let mut tier = UserTier::Free;
    let mut credits_remaining: u32 = 100;
    let mut input_file: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--user" if i + 1 < args.len() => { user_id = args[i+1].clone(); i += 2; }
            "--tier" if i + 1 < args.len() => { 
                let t = args[i+1].to_lowercase();
                tier = if t == "pro" { UserTier::Pro } else { UserTier::Free }; 
                i += 2; 
            }
            "--credits" if i + 1 < args.len() => { 
                credits_remaining = args[i+1].parse().unwrap_or(100); 
                i += 2; 
            }
            "--input-file" if i + 1 < args.len() => { input_file = Some(args[i+1].clone()); i += 2; }
            _ => { i += 1; }
        }
    }

    // Read chat input from file or stdin
    let chat_content = if let Some(path) = input_file {
        fs::read_to_string(path).unwrap_or_default()
    } else {
        // Read entire stdin; if empty, prompt once
        let mut buffer = String::new();
        // Try non-blocking check: if stdin is a tty, prompt
        eprintln!("Enter your description or edges (e.g., 'A -> B, B -> C') then press Ctrl+D:");
        io::stdin().read_to_string(&mut buffer)?;
        buffer.trim().to_string()
    };

    let initial_state = SharedState {
        user_session: UserSession {
            user_id,
            is_authenticated: false, // will be set by AuthenticationNode
            tier,
            credits_remaining,
            last_activity: String::new(),
        },
        chat_input: ChatInput {
            input_type: InputType::Text,
            content: chat_content,
            timestamp: String::new(),
        },
        ai_response: AiResponse { status: state::AiStatus::Success, message: None, graph_data: None, credits_cost: 0 },
        current_graph: None,
        payment_info: None,
    };

    // Initialize context and insert SharedState
    let mut context = Context::new();
    context.set("shared_state", json!(initial_state.clone()));

    // Create and run the graph flow
    let graph_flow = create_graph_flow();
    let final_context = graph_flow.run(context).await?;

    // Retrieve the final SharedState from the context
    let final_shared_state: SharedState = final_context.get("shared_state")
        .cloned()
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default();

    // Print the final state
    println!("\n=== Final SharedState ===");
    println!("{:#?}", final_shared_state);
    
    Ok(())
}
