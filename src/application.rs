use std::collections::HashMap;

#[derive(Debug)]
pub struct Application {
    pub project_id: String,
    pub application_id: String,
    pub image_name: String,
    pub image_tag: String,
    pub env_variables: HashMap<String, String>,
}
