use pocketflow_rs::{Node, ProcessResult, Context, ProcessState};
use anyhow::Result;
use async_trait::async_trait;
use std::io::{self, Write};
use std::fs;
use std::path::Path;
use std::process::Command;
use std::collections::{HashMap, BTreeMap, VecDeque};
use crate::state::{AiStatus, SharedState, UserSession, UserTier, ChatInput, InputType, AiResponse, Graph, GraphData, PaymentInfo, PaymentStatus};
use crate::utils::{call_llm_ai_model, parse_media, db_save_graph, db_update_user_credits, process_payment, auth_authenticate, auth_validate_session, db_retrieve_graph};
use serde_json::json;
use chrono::Utc;
// use crate::excalidraw::graphdata_to_excalidraw_scene; // not needed here

// Generate a filesystem-friendly suggested filename from user text
fn suggest_filename(text: &str) -> String {
    let mut s = String::new();
    for ch in text.chars() {
        if ch.is_ascii_alphanumeric() {
            s.push(ch.to_ascii_lowercase());
        } else if ch.is_whitespace() || ch == '-' || ch == '_' {
            if !s.ends_with('-') { s.push('-'); }
        }
        if s.len() >= 48 { break; }
    }
    let s = s.trim_matches('-');
    if s.is_empty() { "graph".to_string() } else { s.to_string() }
}

// --- Auto layout helpers ---

fn approx_node_size(label: &str) -> (f64, f64) {
    let w = (label.len() as f64 * 10.0 + 30.0).max(100.0);
    let h = 48.0;
    (w, h)
}

fn apply_auto_layout(g: &mut GraphData, node_gap: f64, rank_gap: f64, dir: &str, _max_per_rank: usize) {
    // Build adjacency and indegree
    let mut adj: HashMap<String, Vec<String>> = HashMap::new();
    let mut indeg: HashMap<String, usize> = HashMap::new();
    for n in &g.nodes { indeg.entry(n.id.clone()).or_insert(0); adj.entry(n.id.clone()).or_insert_with(Vec::new); }
    for e in &g.edges {
        if let (Some(_), Some(_)) = (indeg.get(&e.source), indeg.get(&e.target)) {
            adj.entry(e.source.clone()).or_default().push(e.target.clone());
            *indeg.entry(e.target.clone()).or_insert(0) += 1;
        }
    }

    // Kahn topo
    let mut q: VecDeque<String> = indeg.iter().filter(|(_, &d)| d==0).map(|(k,_)| k.clone()).collect();
    let mut order: Vec<String> = Vec::new();
    let mut indeg_mut = indeg.clone();
    while let Some(u) = q.pop_front() {
        order.push(u.clone());
        if let Some(vs) = adj.get(&u) {
            for v in vs {
                if let Some(d) = indeg_mut.get_mut(v) {
                    if *d > 0 { *d -= 1; if *d == 0 { q.push_back(v.clone()); } }
                }
            }
        }
    }
    if order.len() != g.nodes.len() {
        // Graph may have cycles; fall back to input order
        order = g.nodes.iter().map(|n| n.id.clone()).collect();
    }

    // Longest-path rank assignment
    let mut rank: HashMap<String, usize> = HashMap::new();
    for id in &order { rank.insert(id.clone(), 0); }
    for u in &order {
        let ru = *rank.get(u).unwrap_or(&0);
        if let Some(vs) = adj.get(u) {
            for v in vs {
                let entry = rank.entry(v.clone()).or_insert(0);
                if ru + 1 > *entry { *entry = ru + 1; }
            }
        }
    }

    // Group nodes by rank preserving relative order
    let mut by_rank: BTreeMap<usize, Vec<String>> = BTreeMap::new();
    for id in &order {
        let r = *rank.get(id).unwrap_or(&0);
        by_rank.entry(r).or_default().push(id.clone());
    }

    // (Reserved for future centering calculations)

    // Map id -> position with multi-row packing per rank
    let mut pos: HashMap<String, (f64, f64)> = HashMap::new();
    for (r, ids) in &by_rank {
        let r = *r as f64;
        let chunks: Vec<&[String]> = ids.chunks(_max_per_rank.max(1)).collect();
        // center rows/cols around 0
        let rows = chunks.len() as f64;
        let start_secondary = -((rows - 1.0) * rank_gap) / 2.0;
        for (row_idx, chunk) in chunks.iter().enumerate() {
            let count_primary = chunk.len() as f64;
            let start_primary = -((count_primary - 1.0) * rank_gap) / 2.0;
            for (col_idx, id) in chunk.iter().enumerate() {
                let _ = approx_node_size(&g.nodes.iter().find(|n| n.id == **id).map(|n| n.label.clone()).unwrap_or_default());
                let col = col_idx as f64;
                let row = row_idx as f64;
                let (x, y) = if dir == "TB" {
                    // ranks go down (y), primary spread is x (within a row), rows stacked vertically
                    let y = r * node_gap + (start_secondary + row * rank_gap);
                    let x = start_primary + col * rank_gap;
                    (x, y)
                } else {
                    // ranks go right (x), primary spread is y (within a column), rows stacked horizontally
                    let x = r * node_gap + (start_secondary + row * rank_gap);
                    let y = start_primary + col * rank_gap;
                    (x, y)
                };
                pos.insert((*id).clone(), (x, y));
            }
        }
    }

    // Apply positions back into nodes
    for n in &mut g.nodes {
        if let Some((x, y)) = pos.get(&n.id) {
            n.x = *x as f32;
            n.y = *y as f32;
        }
    }
}

// --- Intent helpers ---
fn infer_diagram_kind(text: &str) -> (&'static str, &'static str) {
    let lower = text.to_lowercase();
    // Explicit tags have priority
    if lower.contains(":sequence") || lower.contains("[mode: sequence]") || lower.contains("<sequence>") {
        return ("sequence", "LR");
    }
    if lower.contains(":mindmap") || lower.contains("[mode: mindmap]") || lower.contains("<mindmap>") {
        return ("mindmap", "TB");
    }
    if lower.contains(":system") || lower.contains("[mode: system]") || lower.contains("architecture") {
        return ("system", "LR");
    }
    if lower.contains(":flow") || lower.contains("[mode: flow]") || lower.contains("workflow") || lower.contains("process") {
        return ("flow", "TB");
    }
    // Heuristics
    if lower.contains("->") || lower.contains("then") || lower.contains("next") { return ("flow", "TB"); }
    if lower.contains("actor") || lower.contains("request") || lower.contains("service") { return ("system", "LR"); }
    if lower.contains("brainstorm") || lower.contains("topics") { return ("mindmap", "TB"); }
    if lower.contains(":") && lower.contains("->") { return ("sequence", "LR"); }
    // Notes detection: bullets, numbered lists, many short lines
    let lines: Vec<&str> = text.lines().collect();
    let bullet_like = lines.iter().filter(|l| {
        let s = l.trim_start();
        s.starts_with("-") || s.starts_with("*") || s.chars().take(3).all(|c| c.is_numeric()) && s.contains('.')
    }).count();
    if bullet_like >= 2 || lines.len() >= 6 {
        // Bias to mindmap for organization; TB is clearer for radial/branching
        return ("mindmap", "TB");
    }
    // Default generic
    ("auto", "TB")
}

pub struct GetQuestionNode;

#[async_trait]
impl Node for GetQuestionNode {
    // ... (rest of the code remains the same)
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

        // Ask the LLM to output ONLY valid JSON matching our GraphData schema.
        let (kind, default_dir) = infer_diagram_kind(&chat_input.content);
        let mut prompt = r##"
You are the Logic Engine of a two-stage diagram system. Focus ONLY on logic & structure. Output JSON ONLY (no prose, no markdown).

CONSTRAINTS
- Do NOT include SVG, images, or styling.
- Prefer DAGs unless cycles are explicit and labeled.
- Avoid ambiguity; design a balanced, readable structure.

WHEN TO INFER
- If not provided, infer the suitable diagram family.
- Fill minimal missing connections only when clearly implied.

GRAPHFLOW SCHEMA (exact keys)
{
  "nodes": [{"id":"string_snake_case","label":"string","x":0,"y":0,"style":{"shape":"rect","color":"#F3F4F6"}}],
  "edges": [{"id":"string_snake_case","source":"node_id","target":"node_id","label":"","style":{"line":"orthogonal","arrow":"end"}}],
  "layout_hints": {"direction":"LR"|"TB","algorithm":"longest_path"},
  "global_style": {"font":"Inter","background":"#FFFFFF","theme":"minimal"},
  "decorations": null | [{
     "type": "icon"|"note",
     "target": "node_id_or_edge_id"|null,
     "builtin": "database"|"model"|"search"|"email"|"salesperson"|null,
     "url": "",
     "size": {"w":number,"h":number}|null,
     "offset": {"dx":number,"dy":number}|null,
     "text": ""|null
  }],
  "containers": null | [{"id":"string_snake_case","label":"string","children":["node_id"],"style":{"bg":"#FFFFFF","border":"#D1D5DB","radius":12,"label_tag":"string"}}]
}

RULES
- IDs unique, snake_case; no dangling edges; no duplicate edges.
- Containers reference existing nodes only; limit decorations ≤ 3.
- Decisions use edge labels; only add gateway nodes when required.

DIAGRAM GUIDANCE
- Kind: KINDSLOT. If "auto", choose among flow, system, sequence, mindmap.
- Layout: set layout_hints.direction to "DIRSLOT" unless readability is better otherwise; algorithm "longest_path".
- Flowchart: clear start/end, labeled branches, balanced symmetry.
- System: group components in meaningful containers; orthogonal connectors.
- Sequence: actors left→right; messages as labeled edges; consider TB if clearer.
- Mindmap: central topic with branches; avoid cycles.

DECORATIONS (icons by the model)
- Only add when they materially improve comprehension (max 3).
- Prefer built-in icon names matching the assets dir: database, model, search, email, salesperson.
- Use one of:
  - builtin: "<name>"
  - url: "builtin:<name>"
- Place relative to target center with small offset to corners (e.g., dx:-24, dy:-24) and size 16–24.
- If no target is provided, you may use absolute at_x/at_y placement.

OUTPUT
Return ONE valid JSON object matching the schema above. No extra keys, no comments.

USER_INPUT
{content}
"##.to_string();
        prompt = prompt.replace("KINDSLOT", kind).replace("DIRSLOT", default_dir).replace("{content}", &chat_input.content);

        let ai_response_str = call_llm_ai_model(&prompt, &tier).await.map_err(|e| anyhow::anyhow!(e))?;

        // Try to parse strict JSON GraphData from the LLM.
        let graph_data: GraphData = match serde_json::from_str(&ai_response_str) {
            Ok(gd) => gd,
            Err(_e) => {
                // Fallback: heuristic edge-list parser (A -> B -> C, commas separate statements)
                let mut nodes: std::collections::BTreeMap<String, crate::state::NodeData> = std::collections::BTreeMap::new();
                let mut edges: Vec<crate::state::EdgeData> = Vec::new();

                let content = chat_input.content.replace("\n", ",");
                let mut edge_counter = 0usize;
                for part in content.split(',') {
                    let s = part.trim();
                    if s.is_empty() { continue; }
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

                GraphData {
                    nodes: nodes.into_values().collect(),
                    edges,
                    layout_hints: Some(crate::state::LayoutHints { direction: "TB".to_string(), algorithm: "dagre".to_string() }),
                    global_style: Some(crate::state::GlobalStyle { font: "Inter".to_string(), background: "#ffffff".to_string(), theme: Some("minimal".to_string()) }),
                    decorations: None,
                    containers: None,
                }
            }
        };

        let ai_response = AiResponse {
            status: AiStatus::Success,
            message: Some("ok".to_string()),
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
                    let mut gd: GraphData = serde_json::from_value(rendered_graph_data_value.clone())
                        .map_err(|e| anyhow::anyhow!("Failed to deserialize GraphData: {}", e))?;
                    // Auto-layout to avoid overlaps, ignoring LLM-provided coordinates
                    let (mut dir, node_gap, rank_gap) = ("LR".to_string(), 180.0, 140.0);
                    if let Some(h) = gd.layout_hints.as_ref() {
                        if !h.direction.is_empty() { dir = h.direction.to_uppercase(); }
                    }
                    apply_auto_layout(&mut gd, node_gap, rank_gap, &dir, 4);
                    // Write Excalidraw scene if requested
                    if let Some(path_val) = context.get("export_excalidraw_path").cloned() {
                        if let Ok(opt_path) = serde_json::from_value::<Option<String>>(path_val) {
                            if let Some(path) = opt_path {
                                // read options
                                let allow_images = context.get("allow_images").and_then(|v| v.as_bool()).unwrap_or(false);
                                let assets_dir = context.get("assets_dir").and_then(|v| v.as_str()).unwrap_or("");
                                let scene = crate::excalidraw::graphdata_to_excalidraw_scene_with_opts(&gd, allow_images, assets_dir);
                                let scene_str = serde_json::to_string_pretty(&scene).unwrap_or_else(|_| scene.to_string());
                                if let Err(e) = fs::write(&path, scene_str) {
                                    eprintln!("Failed to write Excalidraw scene to {}: {}", path, e);
                                } else {
                                    eprintln!("Excalidraw scene exported to {}", path);
                                    // Auto-render PNG and SVG to project_root/docs/screens with suggested filename (independent of CWD)
                                    let project_root = Path::new(env!("CARGO_MANIFEST_DIR"));
                                    let suggested = suggest_filename(&shared_state.chat_input.content);
                                    let out_dir_abs = project_root.join("docs/screens");
                                    if let Err(e) = fs::create_dir_all(&out_dir_abs) { eprintln!("Failed to ensure docs/screens: {}", e); }
                                    let out_png_abs = out_dir_abs.join(format!("{}.png", suggested));
                                    let out_svg_abs = out_dir_abs.join(format!("{}.svg", suggested));
                                    let render_script_abs = project_root.join("tools/render-excalidraw/render.js");
                                    // Canonicalize scene path if possible
                                    let scene_abs = Path::new(&path).canonicalize().unwrap_or_else(|_| Path::new(&path).to_path_buf());
                                    let status = Command::new("node")
                                        .arg(render_script_abs)
                                        .arg(&scene_abs)
                                        .arg(&out_png_abs)
                                        .status();
                                    match status {
                                        Ok(s) if s.success() => eprintln!("Rendered PNG -> {}", out_png_abs.display()),
                                        Ok(s) => eprintln!("Renderer exited with status {}", s),
                                        Err(e) => eprintln!("Failed to run renderer: {}", e),
                                    }
                                    // Render SVG
                                    let render_script_abs = project_root.join("tools/render-excalidraw/render.js");
                                    let status_svg = Command::new("node")
                                        .arg(render_script_abs)
                                        .arg(&scene_abs)
                                        .arg(&out_svg_abs)
                                        .status();
                                    match status_svg {
                                        Ok(s) if s.success() => eprintln!("Rendered SVG -> {}", out_svg_abs.display()),
                                        Ok(s) => eprintln!("Renderer (SVG) exited with status {}", s),
                                        Err(e) => eprintln!("Failed to run renderer for SVG: {}", e),
                                    }
                                }
                            }
                        }
                    }
                    shared_state.ai_response.graph_data = Some(gd);
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
            Ok(_value) => {
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
