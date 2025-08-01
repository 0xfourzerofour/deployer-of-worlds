use std::sync::mpsc::{Receiver, Sender};

use crate::{pipeline::Pipeline, PipelineExecutor};

pub enum ExecutorCommand {
    Start(String),
    Status(String),
    Execute(Pipeline),
}

pub enum ExecutorResponse {
    StartResponse(String),
    StatusResponse(String),
    ExecuteResponse(String),
}

pub struct Executor {
    sender: Sender<ExecutorCommand>,
    receiver: Receiver<ExecutorResponse>,
}

impl Executor {
    pub fn new(sender: Sender<ExecutorCommand>, receiver: Receiver<ExecutorResponse>) -> Self {
        Self { sender, receiver }
    }

    pub fn start() {}
}

impl PipelineExecutor for Executor {
    fn execute<P: Into<crate::pipeline::Pipeline>>(
        &self,
        pipeline: P,
    ) -> anyhow::Result<crate::PipelineStartResponse, crate::PipelineError> {
    }

    fn status(
        &self,
        pipeline_id: &str,
    ) -> anyhow::Result<crate::PipelineStatus, crate::PipelineError> {
    }

    fn start(
        &self,
        pipeline_id: &str,
    ) -> anyhow::Result<crate::PipelineStartResponse, crate::PipelineError> {
    }
}
