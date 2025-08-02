use alloy::providers::ProviderBuilder;
use anyhow::Result;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use deployer::{config::config::Config, executor::Executor};
use deployer_core::{Action, Variable};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;
use tracing::{error, info};

#[derive(Clone)]
pub struct AppState {
    pub data_dir: String,
    pub rpc_url: String,
    pub configs: Arc<RwLock<HashMap<String, Config>>>,
}

#[derive(Serialize, Deserialize)]
pub struct ExecutePipelineRequest {
    pub config_name: String,
    pub dry_run: Option<bool>,
}

#[derive(Serialize, Deserialize)]
pub struct ExecuteOneOffRequest {
    pub variables: HashMap<String, Variable>,
    pub actions: Vec<Action>,
    pub dry_run: Option<bool>,
}

#[derive(Serialize)]
pub struct ExecutionResponse {
    pub success: bool,
    pub message: String,
    pub execution_id: Option<String>,
}

#[derive(Serialize)]
pub struct ListConfigsResponse {
    pub configs: Vec<String>,
}

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}

pub async fn create_server(data_dir: String, rpc_url: String) -> Result<Router> {
    let state = AppState {
        data_dir: data_dir.clone(),
        rpc_url,
        configs: Arc::new(RwLock::new(HashMap::new())),
    };

    // Load all configs on startup
    load_configs(&state).await?;

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/configs", get(list_configs))
        // .route("/configs/:name", get(get_config))
        // .route("/execute/:name", post(execute_pipeline))
        // .route("/execute", post(execute_one_off))
        .with_state(state);

    Ok(app)
}

async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

async fn list_configs(State(state): State<AppState>) -> Json<ListConfigsResponse> {
    let configs = state.configs.read().await;
    let config_names: Vec<String> = configs.keys().cloned().collect();

    Json(ListConfigsResponse {
        configs: config_names,
    })
}

async fn get_config(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Json<Option<Config>> {
    let configs = state.configs.read().await;
    Json(configs.get(&name).cloned())
}

async fn execute_pipeline(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(request): Json<ExecutePipelineRequest>,
) -> Result<Json<ExecutionResponse>, StatusCode> {
    info!("Executing pipeline: {}", name);

    // Get config
    let config = {
        let configs = state.configs.read().await;
        match configs.get(&name) {
            Some(config) => config.clone(),
            None => {
                return Ok(Json(ExecutionResponse {
                    success: false,
                    message: format!("Configuration '{}' not found", name),
                    execution_id: None,
                }));
            }
        }
    };

    match execute_config(
        config.clone(),
        &state.rpc_url,
        &state.data_dir,
        request.dry_run.unwrap_or(false),
    )
    .await
    {
        Ok(execution_id) => Ok(Json(ExecutionResponse {
            success: true,
            message: "Pipeline executed successfully".to_string(),
            execution_id: Some(execution_id),
        })),
        Err(e) => {
            error!("Pipeline execution failed: {}", e);
            Ok(Json(ExecutionResponse {
                success: false,
                message: format!("Execution failed: {}", e),
                execution_id: None,
            }))
        }
    }
}

async fn execute_one_off(
    State(state): State<AppState>,
    Json(request): Json<ExecuteOneOffRequest>,
) -> Result<Json<ExecutionResponse>, StatusCode> {
    info!(
        "Executing one-off pipeline with {} actions",
        request.actions.len()
    );

    // Create temporary config from request
    let config = Config {
        variables: request.variables,
        data: HashMap::new(), // No data references for one-off pipelines
        actions: request.actions,
    };

    match execute_config(
        config.clone(),
        &state.rpc_url,
        &state.data_dir,
        request.dry_run.unwrap_or(false),
    )
    .await
    {
        Ok(execution_id) => Ok(Json(ExecutionResponse {
            success: true,
            message: "One-off pipeline executed successfully".to_string(),
            execution_id: Some(execution_id),
        })),
        Err(e) => {
            error!("One-off pipeline execution failed: {}", e);
            Ok(Json(ExecutionResponse {
                success: false,
                message: format!("Execution failed: {}", e),
                execution_id: None,
            }))
        }
    }
}

async fn execute_config(config: Config, rpc_url: &str, data_dir: &str, dry_run: bool) -> Result<String> {
    let execution_id = uuid::Uuid::new_v4().to_string();

    if dry_run {
        info!(
            "Dry run mode - would execute {} actions",
            config.actions.len()
        );
        for action in &config.actions {
            info!(
                "Would execute action: {} (type: {:?})",
                action.id, action.action_data
            );
        }
        return Ok(execution_id);
    }

    // Create provider
    let provider = ProviderBuilder::new().connect_http(rpc_url.parse()?);

    // Create executor with data directory
    let mut executor = Executor::new(provider);
    executor.register_config(config)?;
    executor.execute_actions().await?;

    Ok(execution_id)
}

async fn load_configs(state: &AppState) -> Result<()> {
    let pipelines_dir = std::path::Path::new(&state.data_dir).join("pipelines");

    if !pipelines_dir.exists() {
        info!("Creating pipelines directory: {}", pipelines_dir.display());
        std::fs::create_dir_all(&pipelines_dir)?;
        return Ok(());
    }

    let mut configs = state.configs.write().await;

    for entry in std::fs::read_dir(&pipelines_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("yml")
            || path.extension().and_then(|s| s.to_str()) == Some("yaml")
        {
            let file_name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string();

            match Config::load_from_file(path.to_str().unwrap()) {
                Ok(config) => {
                    info!("Loaded config: {}", file_name);
                    configs.insert(file_name, config);
                }
                Err(e) => {
                    error!("Failed to load config {}: {}", file_name, e);
                }
            }
        }
    }

    info!("Loaded {} configurations", configs.len());
    Ok(())
}
