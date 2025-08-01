use rocket::serde::json::Json;
use rocket::{get, post, State};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

struct PipelineConfig {
    name: String,
}

struct PipelineStatus {
    id: String,
    status: String,
}

fn run_pipeline(config: PipelineConfig) -> String {
    "pipeline_id_placeholder".to_string()
}

#[post("/pipeline/<id>")]
fn run_local_pipeline(id: String) -> String {
    format!("Running local pipeline with id {}", id)
}

#[post("/pipeline", data = "<config>")]
fn run_new_pipeline(
    config: Json<PipelineConfig>,
    state: &State<Arc<Mutex<HashMap<String, PipelineStatus>>>>,
) -> Json<PipelineStatus> {
    let pipeline_id = run_pipeline(config.into_inner());
    let mut status_map = state.lock().unwrap();
    status_map.insert(
        pipeline_id.clone(),
        PipelineStatus {
            id: pipeline_id.clone(),
            status: "running".to_string(),
        },
    );
    Json(PipelineStatus {
        id: pipeline_id,
        status: "running".to_string(),
    })
}

#[get("/status/<p_id>")]
fn get_pipeline_status(
    p_id: String,
    state: &State<Arc<Mutex<HashMap<String, PipelineStatus>>>>,
) -> Option<Json<PipelineStatus>> {
    state
        .lock()
        .unwrap()
        .get(&p_id)
        .map(|status| Json(status.clone()))
}

#[get("/status")]
fn get_all_pipeline_status(
    state: &State<Arc<Mutex<HashMap<String, PipelineStatus>>>>,
) -> Json<Vec<PipelineStatus>> {
    let status_map = state.lock().unwrap();
    Json(status_map.values().cloned().collect())
}
