use pocketflow_rs::{Node, ProcessResult, Context, ProcessState};
use anyhow::Result;
use async_trait::async_trait;
use std::io::{self, Write};
use crate::state::{AiStatus, SharedState, UserSession, UserTier, ChatInput, InputType, AiResponse, Graph, GraphData, PaymentInfo, PaymentStatus};
use crate::utils::{call_llm_ai_model, parse_media, db_save_graph, db_update_user_credits, process_payment, auth_authenticate, auth_validate_session, db_retrieve_graph};
use serde_json::json;
use chrono::Utc;

pub struct GetQuestionNode;

#[async_trait]
impl Node for GetQuestionNode {
    type State = SharedState;

    async fn execute(&self, _context: &Context) -> Result<serde_json::Value> {
        // Get question directly from user input
        print!("Enter your question: ");
        io::stdout().flush()?;
        
        let mut user_question = String::new();
        io::stdin()
            .read_line(&mut user_question)?;
        
        let question = user_question.trim().to_string();
        Ok(serde_json::json!({"question": question}))
    }

    async fn post_process(
        &self,
        context: &mut Context,
        result: &Result<serde_json::Value>,
    ) -> Result<ProcessResult<Self::State>> {
        let mut shared_state: SharedState = context.get("shared_state")
            .cloned()
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or_default();
        if let Ok(value) = result {
            if let Some(question) = value.get("question") {
                shared_state.chat_input.content = question.as_str().unwrap_or_default().to_string();
            }
            shared_state.ai_response.status = AiStatus::Success;
        } else {
            shared_state.ai_response.status = AiStatus::Failure;
            shared_state.ai_response.message = Some(format!("Error in GetQuestionNode: {:?}", result.as_ref().err()));
        }
        context.set("shared_state", json!(shared_state.clone()));
        Ok(ProcessResult::new(shared_state.clone(), shared_state.to_condition()))
    }
}

pub struct AnswerNode;

#[async_trait]
impl Node for AnswerNode {
    type State = SharedState;

    async fn execute(&self, context: &Context) -> Result<serde_json::Value> {
        let shared_state: SharedState = context.get("shared_state")
            .cloned()
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or_default();
        let question = shared_state.chat_input.content.clone();
        let tier = shared_state.user_session.tier.clone();
        
        // Call LLM to get the answer
        let answer = call_llm_ai_model(&question, &tier).await.map_err(|e| anyhow::anyhow!(e))?;
        
        Ok(json!({"answer": answer}))
    }

    async fn post_process(
        &self,
        context: &mut Context,
        result: &Result<serde_json::Value>,
    ) -> Result<ProcessResult<Self::State>> {
        let mut shared_state: SharedState = context.get("shared_state")
            .cloned()
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or_default();
        if let Ok(value) = result {
            if let Some(answer) = value.get("answer") {
                shared_state.ai_response.message = Some(answer.as_str().unwrap_or_default().to_string());
            }
            shared_state.ai_response.status = AiStatus::Success;
        } else {
            shared_state.ai_response.status = AiStatus::Failure;
            shared_state.ai_response.message = Some(format!("Error in AnswerNode: {:?}", result.as_ref().err()));
        }
        context.set("shared_state", json!(shared_state.clone()));
        Ok(ProcessResult::new(shared_state.clone(), shared_state.to_condition()))
    }
}

pub struct AuthenticationNode;

#[async_trait]
impl Node for AuthenticationNode {
    type State = SharedState;

    async fn execute(&self, context: &Context) -> Result<serde_json::Value> {
        let shared_state: SharedState = context.get("shared_state")
            .cloned()
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or_default();
        let username = shared_state.user_session.user_id.clone(); // Using user_id as username for simplicity
        let password = "password".to_string(); // Placeholder password

        let auth_result = auth_authenticate(&username, &password)
            .map_err(|e| anyhow::anyhow!(e))?;

        Ok(json!({"session_token": auth_result}))
    }

    async fn post_process(
        &self,
        context: &mut Context,
        result: &Result<serde_json::Value>,
    ) -> Result<ProcessResult<Self::State>> {
        let mut shared_state: SharedState = context.get("shared_state")
            .cloned()
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or_default();
        match result {
            Ok(value) => {
                if let Some(session_token) = value.get("session_token").and_then(|v| v.as_str()) {
                    let is_authenticated = auth_validate_session(session_token)
                        .map_err(|e| anyhow::anyhow!(e))?;
                    
                    shared_state.user_session = UserSession {
                        user_id: shared_state.user_session.user_id.clone(),
                        is_authenticated,
                        tier: UserTier::Free, // Placeholder
                        credits_remaining: 100, // Placeholder
                        last_activity: Utc::now().to_rfc3339(),
                    };
                    shared_state.ai_response.status = AiStatus::Success;
                    context.set("shared_state", json!(shared_state.clone()));
                    Ok(ProcessResult::new(shared_state.clone(), shared_state.to_condition()))
                } else {
                    shared_state.ai_response.status = AiStatus::Failure;
                    shared_state.ai_response.message = Some("Authentication failed: No session token".to_string());
                    context.set("shared_state", json!(shared_state.clone()));
                    Ok(ProcessResult::new(shared_state.clone(), shared_state.to_condition()))
                }
            },
            Err(e) => {
                shared_state.ai_response.status = AiStatus::Failure;
                shared_state.ai_response.message = Some(format!("Authentication error: {}", e));
                context.set("shared_state", json!(shared_state.clone()));
                Ok(ProcessResult::new(shared_state.clone(), shared_state.to_condition()))
            },
        }
    }
}

pub struct ChatInputNode;

#[async_trait]
impl Node for ChatInputNode {
    type State = SharedState;

    async fn execute(&self, context: &Context) -> Result<serde_json::Value> {
        let shared_state: SharedState = context.get("shared_state")
            .cloned()
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or_default();
        let raw_input = shared_state.chat_input.content.clone();
        let input_type = shared_state.chat_input.input_type.clone();

        let content = if matches!(input_type, InputType::Image | InputType::Link | InputType::Video) {
            parse_media(&raw_input).map_err(|e| anyhow::anyhow!(e))?
        } else {
            raw_input
        };

        let chat_input = ChatInput {
            input_type,
            content,
            timestamp: Utc::now().to_rfc3339(),
        };

        Ok(json!(chat_input))
    }

    async fn post_process(
        &self,
        context: &mut Context,
        result: &Result<serde_json::Value>,
    ) -> Result<ProcessResult<Self::State>> {
        let mut shared_state: SharedState = context.get("shared_state")
            .cloned()
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or_default();
        match result {
            Ok(value) => {
                shared_state.chat_input = serde_json::from_value(value.clone())
                    .map_err(|e| anyhow::anyhow!("Failed to deserialize ChatInput: {}", e))?;
                shared_state.ai_response.status = AiStatus::Success;
                context.set("shared_state", json!(shared_state.clone()));
                Ok(ProcessResult::new(shared_state.clone(), shared_state.to_condition()))
            },
            Err(e) => {
                shared_state.ai_response.status = AiStatus::Failure;
                shared_state.ai_response.message = Some(format!("Chat input error: {}", e));
                context.set("shared_state", json!(shared_state.clone()));
                Ok(ProcessResult::new(shared_state.clone(), shared_state.to_condition()))
            },
        }
    }
}

pub struct AIProcessingNode;

#[async_trait]
impl Node for AIProcessingNode {
    type State = SharedState;

    async fn execute(&self, context: &Context) -> Result<serde_json::Value> {
        let shared_state: SharedState = context.get("shared_state")
            .cloned()
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or_default();
        let chat_input = shared_state.chat_input.clone();
        let tier = shared_state.user_session.tier.clone();
        let user_session = shared_state.user_session.clone();

        let credits_cost = 5u32;

        if matches!(user_session.tier, UserTier::Free) && !matches!(chat_input.input_type, InputType::Text) {
            let ai_response = AiResponse {
                status: AiStatus::Failure,
                message: Some("Feature unavailable: upgrade to Pro for image/link/video inputs".to_string()),
                graph_data: None,
                credits_cost,
            };
            return Ok(json!(ai_response));
        }

        if user_session.credits_remaining < credits_cost {
            let ai_response = AiResponse {
                status: AiStatus::Failure,
                message: Some("Insufficient credits".to_string()),
                graph_data: None,
                credits_cost,
            };
            return Ok(json!(ai_response));
        }

        let prompt = format!("Process this chat input: {}", chat_input.content);
        let ai_response_str = call_llm_ai_model(&prompt, &tier).await.map_err(|e| anyhow::anyhow!(e))?;

        let mut nodes: std::collections::BTreeMap<String, crate::state::NodeData> = std::collections::BTreeMap::new();
        let mut edges: Vec<crate::state::EdgeData> = Vec::new();

        let content = chat_input.content.replace("\n", ",");
        let mut edge_counter = 0usize;
        for part in content.split(',') {
            let s = part.trim();
            if s.is_empty() { continue; }
            // Support chains: A -> B -> C
            let tokens: Vec<String> = s.split("->").map(|t| t.trim().to_string()).filter(|t| !t.is_empty()).collect();
            if tokens.len() >= 2 {
                for w in tokens.windows(2) {
                    let from = w[0].clone();
                    let to = w[1].clone();
                    if !nodes.contains_key(&from) {
                        nodes.insert(from.clone(), crate::state::NodeData { id: from.clone(), label: from.clone(), x: (nodes.len() as f32)*160.0, y: 0.0, style: crate::state::NodeStyle { shape: "rectangle".to_string(), color: "#4F46E5".to_string() } });
                    }
                    if !nodes.contains_key(&to) {
                        nodes.insert(to.clone(), crate::state::NodeData { id: to.clone(), label: to.clone(), x: (nodes.len() as f32)*160.0, y: 120.0, style: crate::state::NodeStyle { shape: "rectangle".to_string(), color: "#4F46E5".to_string() } });
                    }
                    let eid = format!("e{}", edge_counter);
                    edge_counter += 1;
                    edges.push(crate::state::EdgeData { id: eid, source: from, target: to, label: String::new(), style: crate::state::EdgeStyle { line: "smooth".to_string(), arrow: "end".to_string() } });
                }
            }
        }

        let graph_data = GraphData {
            nodes: nodes.into_values().collect(),
            edges,
            layout_hints: Some(crate::state::LayoutHints { direction: "TB".to_string(), algorithm: "dagre".to_string() }),
            global_style: Some(crate::state::GlobalStyle { font: "Inter".to_string(), background: "#ffffff".to_string() }),
        };

        let ai_response = AiResponse {
            status: AiStatus::Success,
            message: Some(ai_response_str),
            graph_data: Some(graph_data),
            credits_cost,
        };

        Ok(json!(ai_response))
    }

    async fn post_process(
        &self,
        context: &mut Context,
        result: &Result<serde_json::Value>,
    ) -> Result<ProcessResult<Self::State>> {
        let mut shared_state: SharedState = context.get("shared_state")
            .cloned()
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or_default();
        match result {
            Ok(value) => {
                shared_state.ai_response = serde_json::from_value(value.clone())
                    .map_err(|e| anyhow::anyhow!("Failed to deserialize AiResponse: {}", e))?;
                shared_state.ai_response.status = AiStatus::Success;
                context.set("shared_state", json!(shared_state.clone()));
                Ok(ProcessResult::new(shared_state.clone(), shared_state.to_condition()))
            },
            Err(e) => {
                shared_state.ai_response.status = AiStatus::Failure;
                shared_state.ai_response.message = Some(format!("AI processing error: {}", e));
                context.set("shared_state", json!(shared_state.clone()));
                Ok(ProcessResult::new(shared_state.clone(), shared_state.to_condition()))
            },
        }
    }
}

pub struct GraphRenderingNode;

#[async_trait]
impl Node for GraphRenderingNode {
    type State = SharedState;

    async fn execute(&self, context: &Context) -> Result<serde_json::Value> {
        let shared_state: SharedState = context.get("shared_state")
            .cloned()
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or_default();
        let ai_response = shared_state.ai_response.clone();

        let graph_data = ai_response.graph_data
            .ok_or_else(|| anyhow::anyhow!("Graph data not found in AI response"))?;

        // Placeholder for graph rendering logic
        println!("Rendering graph: {:?}", graph_data);

        Ok(json!({"rendering_status": "success", "rendered_graph_data": graph_data}))
    }

    async fn post_process(
        &self,
        context: &mut Context,
        result: &Result<serde_json::Value>,
    ) -> Result<ProcessResult<Self::State>> {
        let mut shared_state: SharedState = context.get("shared_state")
            .cloned()
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or_default();
        match result {
            Ok(value) => {
                // Assuming `rendered_graph_data` is the actual graph data
                if let Some(rendered_graph_data_value) = value.get("rendered_graph_data") {
                    shared_state.ai_response.graph_data = Some(serde_json::from_value(rendered_graph_data_value.clone())
                        .map_err(|e| anyhow::anyhow!("Failed to deserialize GraphData: {}", e))?);
                }
                shared_state.ai_response.status = AiStatus::Success;
                context.set("shared_state", json!(shared_state.clone()));
                Ok(ProcessResult::new(shared_state.clone(), shared_state.to_condition()))
            },
            Err(e) => {
                shared_state.ai_response.status = AiStatus::Failure;
                shared_state.ai_response.message = Some(format!("Graph rendering error: {}", e));
                context.set("shared_state", json!(shared_state.clone()));
                Ok(ProcessResult::new(shared_state.clone(), shared_state.to_condition()))
            },
        }
    }
}

pub struct GraphPersistenceNode;

#[async_trait]
impl Node for GraphPersistenceNode {
    type State = SharedState;

    async fn execute(&self, context: &Context) -> Result<serde_json::Value> {
        let shared_state: SharedState = context.get("shared_state")
            .cloned()
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or_default();
        let user_session = shared_state.user_session.clone();
        let graph_data_opt = shared_state.ai_response.graph_data.clone();

        if shared_state.chat_input.content.trim().starts_with(":retrieve") {
            let _ = db_retrieve_graph(&user_session.user_id).map_err(|e| anyhow::anyhow!(e))?;
            return Ok(json!({"persistence_status": "retrieved", "graph_id": "retrieved"}));
        }

        let graph_data = graph_data_opt.ok_or_else(|| anyhow::anyhow!("Graph data not found in AI response"))?;

        let graph_to_save = Graph {
            graph_id: "graph_id_123".to_string(), // Placeholder
            user_id: user_session.user_id.clone(),
            name: "My New Graph".to_string(), // Placeholder
            data: graph_data,
            last_edited: Utc::now().to_rfc3339(),
            created_at: Utc::now().to_rfc3339(),
        };

        let graph_json = serde_json::to_string(&graph_to_save)
            .map_err(|e| anyhow::anyhow!("Failed to serialize Graph: {}", e))?;

        db_save_graph(&user_session.user_id, &graph_json)
            .map_err(|e| anyhow::anyhow!(e))?;

        Ok(json!({"persistence_status": "success", "graph_id": graph_to_save.graph_id}))
    }

    async fn post_process(
        &self,
        context: &mut Context,
        result: &Result<serde_json::Value>,
    ) -> Result<ProcessResult<Self::State>> {
        let mut shared_state: SharedState = context.get("shared_state")
            .cloned()
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or_default();
        match result {
            Ok(value) => {
                if value.get("persistence_status").and_then(|v| v.as_str()) == Some("success") {
                    if let Some(graph) = shared_state.ai_response.graph_data.clone() {
                        shared_state.current_graph = Some(Graph {
                            graph_id: value.get("graph_id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                            user_id: shared_state.user_session.user_id.clone(),
                            name: "My New Graph".to_string(),
                            data: graph,
                            last_edited: Utc::now().to_rfc3339(),
                            created_at: Utc::now().to_rfc3339(),
                        });
                    }
                }
                shared_state.ai_response.status = AiStatus::Success;
                context.set("shared_state", json!(shared_state.clone()));
                Ok(ProcessResult::new(shared_state.clone(), shared_state.to_condition()))
            },
            Err(e) => {
                shared_state.ai_response.status = AiStatus::Failure;
                shared_state.ai_response.message = Some(format!("Graph persistence error: {}", e));
                context.set("shared_state", json!(shared_state.clone()));
                Ok(ProcessResult::new(shared_state.clone(), shared_state.to_condition()))
            },
        }
    }
}

pub struct CreditUpdateNode;

#[async_trait]
impl Node for CreditUpdateNode {
    type State = SharedState;

    async fn execute(&self, context: &Context) -> Result<serde_json::Value> {
        let shared_state: SharedState = context.get("shared_state")
            .cloned()
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or_default();
        let mut user_session = shared_state.user_session.clone();
        let ai_response = shared_state.ai_response.clone();

        let credits_cost = ai_response.credits_cost as i32;

        db_update_user_credits(&user_session.user_id, -credits_cost) // Deduct credits
            .map_err(|e| anyhow::anyhow!(e))?;

        user_session.credits_remaining = user_session.credits_remaining.saturating_sub(ai_response.credits_cost);

        Ok(json!({"new_credits_remaining": user_session.credits_remaining}))
    }

    async fn post_process(
        &self,
        context: &mut Context,
        result: &Result<serde_json::Value>,
    ) -> Result<ProcessResult<Self::State>> {
        let mut shared_state: SharedState = context.get("shared_state")
            .cloned()
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or_default();
        match result {
            Ok(value) => {
                if let Some(new_credits) = value.get("new_credits_remaining") {
                    shared_state.user_session.credits_remaining = new_credits.as_u64().unwrap_or_default() as u32;
                }
                shared_state.ai_response.status = AiStatus::Success;
                context.set("shared_state", json!(shared_state.clone()));
                Ok(ProcessResult::new(shared_state.clone(), shared_state.to_condition()))
            },
            Err(e) => {
                shared_state.ai_response.status = AiStatus::Failure;
                shared_state.ai_response.message = Some(format!("Credit update error: {}", e));
                context.set("shared_state", json!(shared_state.clone()));
                Ok(ProcessResult::new(shared_state.clone(), shared_state.to_condition()))
            },
        }
    }
}

pub struct PaymentProcessingNode;

#[async_trait]
impl Node for PaymentProcessingNode {
    type State = SharedState;

    async fn execute(&self, context: &Context) -> Result<serde_json::Value> {
        let shared_state: SharedState = context.get("shared_state")
            .cloned()
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or_default();
        let user_session = shared_state.user_session.clone();

        let amount = shared_state.payment_info.map_or(0.0, |pi| pi.amount as f64);

        process_payment(&user_session.user_id, amount)
            .map_err(|e| anyhow::anyhow!(e))?;

        let payment_info = PaymentInfo {
            transaction_id: "txn_12345".to_string(), // Placeholder
            amount: amount as f32,
            currency: "USD".to_string(),
            status: PaymentStatus::Completed,
            timestamp: Utc::now().to_rfc3339(),
        };

        Ok(json!(payment_info))
    }

    async fn post_process(
        &self,
        context: &mut Context,
        result: &Result<serde_json::Value>,
    ) -> Result<ProcessResult<Self::State>> {
        let mut shared_state: SharedState = context.get("shared_state")
            .cloned()
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or_default();
        match result {
            Ok(value) => {
                shared_state.payment_info = serde_json::from_value(value.clone())
                    .map_err(|e| anyhow::anyhow!("Failed to deserialize PaymentInfo: {}", e))?;
                shared_state.ai_response.status = AiStatus::Success;
                context.set("shared_state", json!(shared_state.clone()));
                Ok(ProcessResult::new(shared_state.clone(), shared_state.to_condition()))
            },
            Err(e) => {
                shared_state.ai_response.status = AiStatus::Failure;
                shared_state.ai_response.message = Some(format!("Payment processing error: {}", e));
                context.set("shared_state", json!(shared_state.clone()));
                Ok(ProcessResult::new(shared_state.clone(), shared_state.to_condition()))
            },
        }
    }
}

pub struct UserFeedbackNode;

#[async_trait]
impl Node for UserFeedbackNode {
    type State = SharedState;

    async fn execute(&self, context: &Context) -> Result<serde_json::Value> {
        let shared_state: SharedState = context.get("shared_state")
            .cloned()
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or_default();
        let user_session = shared_state.user_session.clone();
        let chat_input = shared_state.chat_input.clone();
        let ai_response = shared_state.ai_response.clone();

        // Simulate collecting user feedback
        println!("Collecting feedback for user: {}, input: {:?}, AI response: {:?}",
                 user_session.user_id, chat_input.content, ai_response.message);

        Ok(json!({"feedback_status": "collected", "user_id": user_session.user_id}))
    }

    async fn post_process(
        &self,
        context: &mut Context,
        result: &Result<serde_json::Value>,
    ) -> Result<ProcessResult<Self::State>> {
        let mut shared_state: SharedState = context.get("shared_state")
            .cloned()
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or_default();
        match result {
            Ok(value) => {
                // Assuming we want to store feedback status in shared_state
                // shared_state.user_feedback_status = value.get("feedback_status").map(|s| s.to_string());
                shared_state.ai_response.status = AiStatus::Success;
                context.set("shared_state", json!(shared_state.clone()));
                Ok(ProcessResult::new(shared_state.clone(), shared_state.to_condition()))
            },
            Err(e) => {
                shared_state.ai_response.status = AiStatus::Failure;
                shared_state.ai_response.message = Some(format!("Feedback collection error: {}", e));
                context.set("shared_state", json!(shared_state.clone()));
                Ok(ProcessResult::new(shared_state.clone(), shared_state.to_condition()))
            },
        }
    }
}
