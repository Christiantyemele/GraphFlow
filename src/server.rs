use axum::{routing::post, Router, Json, extract::State};
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use std::path::Path;
use std::process::Command;
use pocketflow_rs::Context as PfContext;
use utoipa::{OpenApi, ToSchema};
use utoipa_swagger_ui::SwaggerUi;

use crate::flow::create_graph_flow;
use crate::state::{SharedState, UserSession, UserTier, ChatInput, InputType, AiResponse, GraphData};
use crate::excalidraw::graphdata_to_excalidraw_scene_with_opts;

#[derive(Clone)]
pub struct AppConfig {
    pub allow_images: bool,
    pub assets_dir: String,
}

#[derive(Deserialize, ToSchema)]
pub struct GenerateRequest {
    pub content: String,
    #[serde(default)]
    pub tier: Option<String>,
    #[serde(default)]
    pub allow_images: Option<bool>,
    #[serde(default)]
    pub assets_dir: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct GenerateResponse {
    pub graph_data: GraphData,
    pub scene: serde_json::Value,
    pub artifacts: serde_json::Value,
}

#[derive(Deserialize, ToSchema)]
pub struct RenderRequest {
    #[serde(default)]
    pub scene: Option<serde_json::Value>,
    #[serde(default)]
    pub graph_data: Option<GraphData>,
    #[serde(default)]
    pub filename_hint: Option<String>,
    #[serde(default)]
    pub formats: Option<Vec<String>>, // ["png","svg"]
}

#[derive(Serialize, ToSchema)]
pub struct RenderResponse {
    pub suggested: String,
    pub png: Option<String>,
    pub svg: Option<String>,
}

#[derive(OpenApi)]
#[openapi(
    paths(handle_generate, handle_render),
    components(schemas(
        GenerateRequest,
        GenerateResponse,
        RenderRequest,
        RenderResponse,
        
        GraphData,
        crate::state::NodeData,
        crate::state::EdgeData,
        crate::state::LayoutHints,
        crate::state::GlobalStyle,
        crate::state::NodeStyle,
        crate::state::EdgeStyle,
        crate::state::Decoration,
        crate::state::DecorationSize,
        crate::state::DecorationOffset,
        crate::state::Container,
        crate::state::ContainerStyle
    )),
    tags(
        (name = "graph", description = "Graph generation and rendering APIs")
    )
)]
pub struct ApiDoc;

pub async fn run_server(port: u16, default_allow_images: bool, default_assets_dir: String) -> anyhow::Result<()> {
    let cfg = AppConfig { allow_images: default_allow_images, assets_dir: default_assets_dir };
    let app = Router::new()
        .route("/graph/generate", post(handle_generate))
        .route("/graph/render", post(handle_render))
        .merge(SwaggerUi::new("/docs").url("/api-doc/openapi.json", ApiDoc::openapi()))
        .with_state(Arc::new(cfg));

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port)).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

/// Generate GraphData and Excalidraw scene from user content.
#[utoipa::path(
    post,
    path = "/graph/generate",
    request_body = GenerateRequest,
    responses(
        (status = 200, description = "Graph generated", body = GenerateResponse),
        (status = 400, description = "Invalid input"),
        (status = 500, description = "Internal error")
    ),
    tag = "graph"
)]
async fn handle_generate(State(cfg): State<Arc<AppConfig>>, Json(req): Json<GenerateRequest>) -> Result<Json<GenerateResponse>, (StatusCode, String)> {
    let allow_images = req.allow_images.unwrap_or(cfg.allow_images);
    let assets_dir = req.assets_dir.clone().unwrap_or_else(|| cfg.assets_dir.clone());
    let tier = match req.tier.as_deref() { Some("pro") => UserTier::Pro, _ => UserTier::Free };

    // Build shared state and run flow
    let initial_state = SharedState {
        user_session: UserSession { user_id: "api".into(), is_authenticated: true, tier, credits_remaining: 100, last_activity: String::new() },
        chat_input: ChatInput { input_type: InputType::Text, content: req.content.clone(), timestamp: String::new() },
        ai_response: AiResponse { status: crate::state::AiStatus::Success, message: None, graph_data: None, credits_cost: 0 },
        current_graph: None,
        payment_info: None,
    };
    let mut pf_ctx = PfContext::new();
    pf_ctx.set("shared_state", json!(initial_state));
    pf_ctx.set("export_excalidraw_path", json!(Option::<String>::None));
    pf_ctx.set("allow_images", json!(allow_images));
    pf_ctx.set("assets_dir", json!(assets_dir.clone()));

    let flow = create_graph_flow();
    let final_ctx = flow.run(pf_ctx).await.map_err(internal_err)?;

    let shared: SharedState = final_ctx.get("shared_state").cloned().and_then(|v| serde_json::from_value(v).ok()).unwrap_or_default();
    let gd = shared.ai_response.graph_data.clone().ok_or((StatusCode::BAD_REQUEST, "No graph generated".to_string()))?;

    let scene = graphdata_to_excalidraw_scene_with_opts(&gd, allow_images, &assets_dir);
    let suggested = suggest_filename(&req.content);
    Ok(Json(GenerateResponse { graph_data: gd, scene, artifacts: json!({
        "suggested": suggested,
        "png": format!("docs/screens/{}.png", suggested),
        "svg": format!("docs/screens/{}.svg", suggested)
    }) }))
}

/// Render a scene (or GraphData) to PNG/SVG artifacts.
#[utoipa::path(
    post,
    path = "/graph/render",
    request_body = RenderRequest,
    responses(
        (status = 200, description = "Rendered artifacts", body = RenderResponse),
        (status = 400, description = "Invalid input"),
        (status = 500, description = "Internal error")
    ),
    tag = "graph"
)]
async fn handle_render(State(cfg): State<Arc<AppConfig>>, Json(req): Json<RenderRequest>) -> Result<Json<RenderResponse>, (StatusCode, String)> {
    let allow_images = cfg.allow_images;
    let assets_dir = cfg.assets_dir.clone();
    let project_root = Path::new(env!("CARGO_MANIFEST_DIR"));

    let scene = if let Some(scene) = req.scene.clone() {
        scene
    } else if let Some(gd) = req.graph_data.clone() {
        graphdata_to_excalidraw_scene_with_opts(&gd, allow_images, &assets_dir)
    } else {
        return Err((StatusCode::BAD_REQUEST, "Provide scene or graph_data".into()));
    };

    // Write scene to a temp file
    let suggested = suggest_filename(req.filename_hint.as_deref().unwrap_or("graph"));
    let out_dir_abs = project_root.join("docs/screens");
    let _ = std::fs::create_dir_all(&out_dir_abs);
    let scene_path = project_root.join(format!("{}.excalidraw.json", suggested));
    let scene_str = serde_json::to_string_pretty(&scene).unwrap_or_else(|_| scene.to_string());
    std::fs::write(&scene_path, scene_str).map_err(internal_err)?;

    // Render requested formats (default both)
    let formats = req.formats.clone().unwrap_or(vec!["png".into(), "svg".into()]);
    let mut png_path = None;
    let mut svg_path = None;
    let render_script = project_root.join("tools/render-excalidraw/render.js");
    if formats.iter().any(|f| f == "png") {
        let out_png = out_dir_abs.join(format!("{}.png", suggested));
        let s = Command::new("node").arg(&render_script).arg(&scene_path).arg(&out_png).status().map_err(internal_err)?;
        if s.success() { png_path = Some(out_png.display().to_string()); }
    }
    if formats.iter().any(|f| f == "svg") {
        let out_svg = out_dir_abs.join(format!("{}.svg", suggested));
        let s = Command::new("node").arg(&render_script).arg(&scene_path).arg(&out_svg).status().map_err(internal_err)?;
        if s.success() { svg_path = Some(out_svg.display().to_string()); }
    }

    Ok(Json(RenderResponse { suggested, png: png_path, svg: svg_path }))
}

fn internal_err<E: std::fmt::Display>(e: E) -> (StatusCode, String) { (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()) }

// Reuse-like filename helper
fn suggest_filename(text: &str) -> String {
    let mut s = String::new();
    for ch in text.chars() {
        if ch.is_ascii_alphanumeric() { s.push(ch.to_ascii_lowercase()); }
        else if ch.is_whitespace() || ch=='-' || ch=='_' { if !s.ends_with('-') { s.push('-'); } }
        if s.len() >= 48 { break; }
    }
    let s = s.trim_matches('-');
    if s.is_empty() { "graph".to_string() } else { s.to_string() }
}
