// Utility functions for GraphFlow

use crate::state::UserTier;
use std::env;

// OpenAI SDK for Pro tier
use async_openai::{Client as OpenAIClient, config::OpenAIConfig};
use async_openai::types::{
    CreateChatCompletionRequestArgs, 
    ChatCompletionRequestMessage,
    ChatCompletionRequestSystemMessage,
    ChatCompletionRequestSystemMessageContent,
    ChatCompletionRequestUserMessage,
    ChatCompletionRequestUserMessageContent,
};

// Anthropic SDK for Free tier
use anthropic_sdk::{Anthropic, MessageCreateBuilder};

// AI processing with switchable providers (Anthropic for Free, OpenAI for Pro)
pub async fn call_llm_ai_model(prompt: &str, tier: &UserTier) -> Result<String, String> {
    match tier {
        UserTier::Free => {
            // Use Anthropic Claude for Free tier
            let api_key = env::var("ANTHROPIC_API_KEY").map_err(|_| "Missing ANTHROPIC_API_KEY".to_string())?;
            let model = env::var("ANTHROPIC_MODEL").unwrap_or_else(|_| "claude-3-5-haiku-latest".to_string());
            
            let client = Anthropic::new(&api_key).map_err(|e| format!("Anthropic client error: {}", e))?;
            
            let response = client.messages()
                .create(
                    MessageCreateBuilder::new(&model, 1024)
                        .system("You are a diagram generation engine. From any user input, infer the best diagram (flow, system architecture, sequence, or mindmap) and convert it into a clear, structured representation. If the input appears to be notes (bullets, numbered lists, paragraphs), summarize and organize them into the most helpful visual to accelerate understanding. Prefer JSON outputs that match the caller's requested schema. Use concise, readable naming, and pick layouts that minimize crossings. Keep responses compact and free of prose unless explicitly asked.")
                        .user(prompt)
                        .build()
                )
                .await
                .map_err(|e| format!("Anthropic error: {}", e))?;
            
            // Extract text from content blocks
            let text = response.content
                .iter()
                .filter_map(|block| {
                    if let anthropic_sdk::ContentBlock::Text { text } = block {
                        Some(text.as_str())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
                .join(" ");
            
            if text.is_empty() {
                Err("Anthropic empty response".to_string())
            } else {
                Ok(text)
            }
        }
        UserTier::Pro => {
            // Use OpenAI for Pro tier
            let api_key = env::var("OPENAI_API_KEY").map_err(|_| "Missing OPENAI_API_KEY".to_string())?;
            let model = env::var("OPENAI_MODEL_PRO").unwrap_or_else(|_| "gpt-4o".to_string());

            let config = OpenAIConfig::new().with_api_key(api_key);
            let client = OpenAIClient::with_config(config);

            let messages = vec![
                ChatCompletionRequestMessage::System(
                    ChatCompletionRequestSystemMessage {
                        content: ChatCompletionRequestSystemMessageContent::Text(
                            "You are a diagram generation engine. From any user input, infer the best diagram (flow, system architecture, sequence, or mindmap) and convert it into a clear, structured representation. If the input appears to be notes (bullets, numbered lists, paragraphs), summarize and organize them into the most helpful visual to accelerate understanding. Prefer JSON outputs that match the caller's requested schema. Use concise, readable naming, and pick layouts that minimize crossings. Keep responses compact and free of prose unless explicitly asked.".to_string()
                        ),
                        name: None,
                    }
                ),
                ChatCompletionRequestMessage::User(
                    ChatCompletionRequestUserMessage {
                        content: ChatCompletionRequestUserMessageContent::Text(prompt.to_string()),
                        name: None,
                    }
                ),
            ];

            let req = CreateChatCompletionRequestArgs::default()
                .model(model)
                .messages(messages)
                .temperature(0.2)
                .build()
                .map_err(|e| e.to_string())?;

            let resp = client.chat().create(req).await.map_err(|e| format!("OpenAI error: {}", e))?;
            let text = resp.choices.first()
                .and_then(|c| c.message.content.clone())
                .unwrap_or_default();
            
            if text.is_empty() { 
                Err("OpenAI empty response".to_string()) 
            } else { 
                Ok(text) 
            }
        }
    }
}

// Media parsing
pub fn parse_media(media_url: &str) -> Result<String, String> {
    println!("Parsing media from URL: {}", media_url);
    Ok(format!("Parsed content from {}", media_url))
}

// Database operations
pub fn db_save_graph(user_id: &str, graph_data: &str) -> Result<(), String> {
    println!("Saving graph for user {} with data: {}", user_id, graph_data);
    Ok(())
}

pub fn db_retrieve_graph(user_id: &str) -> Result<String, String> {
    println!("Retrieving graph for user {}", user_id);
    Ok(format!("Retrieved graph data for {}", user_id))
}

pub fn db_update_user_credits(user_id: &str, amount: i32) -> Result<(), String> {
    println!("Updating credits for user {} by amount: {}", user_id, amount);
    Ok(())
}

// Payment gateway integration
pub fn process_payment(user_id: &str, amount: f64) -> Result<(), String> {
    println!("Processing payment for user {} with amount: {}", user_id, amount);
    Ok(())
}

// Authentication and session management
pub fn auth_authenticate(username: &str, password: &str) -> Result<String, String> {
    println!("Authenticating user: {}", username);
    if username == "test" && password == "password" {
        Ok("session_token_123".to_string())
    } else {
        Err("Authentication failed".to_string())
    }
}

pub fn auth_validate_session(session_token: &str) -> Result<bool, String> {
    println!("Validating session token: {}", session_token);
    if session_token == "session_token_123" {
        Ok(true)
    } else {
        Ok(false)
    }
}
