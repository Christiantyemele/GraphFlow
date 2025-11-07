use pocketflow_rs::{build_flow, Flow};
use crate::nodes::{
    AuthenticationNode, ChatInputNode, AIProcessingNode, GraphRenderingNode,
    GraphPersistenceNode, CreditUpdateNode, PaymentProcessingNode, UserFeedbackNode,
};
use crate::state::SharedState;

/// Create and return the GraphFlow application flow
pub fn create_graph_flow() -> Flow<SharedState> {
    // Instantiate all nodes
    let authentication_node = AuthenticationNode;
    let chat_input_node = ChatInputNode;
    let ai_processing_node = AIProcessingNode;
    let graph_rendering_node = GraphRenderingNode;
    let graph_persistence_node = GraphPersistenceNode;
    let credit_update_node = CreditUpdateNode;
    let payment_processing_node = PaymentProcessingNode;
    let user_feedback_node = UserFeedbackNode;

    // Build flow with nodes connected according to the design diagram
    let flow = build_flow!(
        start: ("authentication", authentication_node),
        nodes: [
            ("chat_input", chat_input_node),
            ("ai_processing", ai_processing_node),
            ("graph_rendering", graph_rendering_node),
            ("graph_persistence", graph_persistence_node),
            ("credit_update", credit_update_node),
            ("payment_processing", payment_processing_node),
            ("user_feedback", user_feedback_node)
        ],
        edges: [
            // Authentication flow
            ("authentication", "chat_input", SharedState::success_state()),
            ("authentication", "user_feedback", SharedState::failure_state()),

            // User Input -> AI Processing
            ("chat_input", "ai_processing", SharedState::success_state()),

            // AI Processing outcomes
            ("ai_processing", "graph_rendering", SharedState::success_state()),
            ("ai_processing", "user_feedback", SharedState::failure_state()), // insufficient_credits, unsupported_feature, ai_processing_error

            // Graph Rendering -> Persistence & Credit Update
            ("graph_rendering", "graph_persistence", SharedState::success_state()),
            ("graph_rendering", "credit_update", SharedState::success_state()),

            // After persistence/credits, finish via feedback
            ("graph_persistence", "user_feedback", SharedState::success_state()),
            ("credit_update", "user_feedback", SharedState::success_state()),

            // Payment flow only on failure
            ("user_feedback", "payment_processing", SharedState::failure_state()),
            ("payment_processing", "ai_processing", SharedState::success_state()),
            ("payment_processing", "user_feedback", SharedState::failure_state())
        ]
    );

    flow
}
