use anyhow::Result;
use pipeline::Pipeline;

pub mod executor;
pub mod pipeline;

#[derive(Debug)]
pub enum PipelineError {
    NotFound,
    ExecutionFailed(String),
    InternalError,
}

pub enum PipelineStatus {
    Success,
    Pending,
    Error,
}

pub struct PipelineStatusResponse {
    status: PipelineStatus,
}

pub struct PipelineStartResponse {
    id: String,
}

pub trait PipelineExecutor {
    fn execute<P: Into<Pipeline>>(
        &self,
        pipeline: P,
    ) -> Result<PipelineStartResponse, PipelineError>;
    fn status(&self, pipeline_id: &str) -> Result<PipelineStatus, PipelineError>;
    fn start(&self, pipeline_id: &str) -> Result<PipelineStartResponse, PipelineError>;
}
