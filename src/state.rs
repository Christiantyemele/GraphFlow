use serde::{Serialize, Deserialize};
use utoipa::ToSchema;
use pocketflow_rs::ProcessState;

#[derive(Debug, Clone, Serialize, Deserialize, Default, ToSchema)]
pub struct SharedState {
    pub user_session: UserSession,
    pub chat_input: ChatInput,
    pub ai_response: AiResponse,
    pub current_graph: Option<Graph>,
    pub payment_info: Option<PaymentInfo>,
}

impl SharedState {
    pub fn success_state() -> Self {
        let mut state = Self::default();
        state.ai_response.status = AiStatus::Success;
        state
    }

    pub fn failure_state() -> Self {
        let mut state = Self::default();
        state.ai_response.status = AiStatus::Failure;
        state
    }
}

impl ProcessState for SharedState {
    fn is_default(&self) -> bool {
        false
    }

    fn to_condition(&self) -> String {
        self.ai_response.to_condition()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, ToSchema)]
pub struct UserSession {
    pub user_id: String,
    pub is_authenticated: bool,
    pub tier: UserTier,
    pub credits_remaining: u32,
    pub last_activity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, ToSchema)]
pub enum UserTier {
    #[default]
    Free,
    Pro,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, ToSchema)]
pub struct ChatInput {
    pub input_type: InputType,
    pub content: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, ToSchema)]
pub enum InputType {
    #[default]
    Text,
    Image,
    Link,
    Video,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, ToSchema)]
pub struct AiResponse {
    pub status: AiStatus,
    pub message: Option<String>,
    pub graph_data: Option<GraphData>,
    pub credits_cost: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, ToSchema)]
pub enum AiStatus {
    #[default]
    Success,
    Failure,
}

impl ProcessState for AiResponse {
    fn is_default(&self) -> bool {
        false
    }

    fn to_condition(&self) -> String {
        match self.status {
            AiStatus::Success => "success".to_string(),
            AiStatus::Failure => "failure".to_string(),
        }
    }
}

impl ProcessState for AiStatus {
    fn is_default(&self) -> bool {
        false
    }

    fn to_condition(&self) -> String {
        match self {
            AiStatus::Success => "success".to_string(),
            AiStatus::Failure => "failure".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, ToSchema)]
pub struct GraphData {
    pub nodes: Vec<NodeData>,
    pub edges: Vec<EdgeData>,
    pub layout_hints: Option<LayoutHints>,
    pub global_style: Option<GlobalStyle>,
    pub decorations: Option<Vec<Decoration>>, // optional visuals/icons/notes
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, ToSchema)]
pub struct NodeData {
    pub id: String,
    pub label: String,
    pub x: f32,
    pub y: f32,
    pub style: NodeStyle,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, ToSchema)]
pub struct EdgeData {
    pub id: String,
    pub source: String,
    pub target: String,
    pub label: String,
    pub style: EdgeStyle,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, ToSchema)]
pub struct LayoutHints {
    pub direction: String,
    pub algorithm: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, ToSchema)]
pub struct GlobalStyle {
    pub font: String,
    pub background: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, ToSchema)]
pub struct NodeStyle {
    pub shape: String,
    pub color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, ToSchema)]
pub struct EdgeStyle {
    pub line: String,
    pub arrow: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, ToSchema)]
pub struct DecorationOffset { pub dx: f32, pub dy: f32 }

#[derive(Debug, Clone, Serialize, Deserialize, Default, ToSchema)]
pub struct DecorationSize { pub w: f32, pub h: f32 }

#[derive(Debug, Clone, Serialize, Deserialize, Default, ToSchema)]
pub struct Decoration {
    pub r#type: String,          // "icon" | "image" | "note"
    pub target: Option<String>,  // node-id or edge-id; if None, use absolute
    pub at_x: Option<f32>,       // absolute x (center), optional
    pub at_y: Option<f32>,       // absolute y (center), optional
    pub builtin: Option<String>, // e.g., salesperson, email
    pub url: Option<String>,     // for assets (e.g., builtin:email or relative path)
    pub size: Option<DecorationSize>,
    pub offset: Option<DecorationOffset>,
    pub text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, ToSchema)]
pub struct Graph {
    pub graph_id: String,
    pub user_id: String,
    pub name: String,
    pub data: GraphData,
    pub last_edited: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PaymentInfo {
    pub transaction_id: String,
    pub amount: f32,
    pub currency: String,
    pub status: PaymentStatus,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum PaymentStatus {
    #[default]
    Completed,
    Pending,
    Failed,
}
