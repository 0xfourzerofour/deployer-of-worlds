#[derive(Debug)]
pub enum ActionType {
    Deployment,
    Write,
    ConditionalWrite,
}

#[derive(Debug)]
pub struct Action {
    action_type: ActionType,
    depends_on: Vec<String>,
    id: String,
    name: String,
    data: String,
    inputs: Vec<String>,
    outputs_schema: String,
}
