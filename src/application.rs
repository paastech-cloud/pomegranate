use std::collections::HashMap;

#[derive(Debug)]
pub struct Application {
    pub project_id: String,
    pub image_name: String,
    pub env_variables: HashMap<String, String>,
}
