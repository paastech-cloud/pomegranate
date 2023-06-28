use std::collections::HashMap;

/// # PaaS application
/// Information about an application of the PaaS.
#[derive(Debug, Default)]
pub struct Application {
    /// ID of the project that the application is a part of.
    pub project_id: String,

    /// ID of the application.
    pub application_id: String,

    /// Name of the Docker image to deploy.
    pub image_name: String,

    /// Tag of the Docker image to deploy.
    pub image_tag: String,

    /// Key-value map of the environment variables to set in the container.
    pub env_variables: HashMap<String, String>,
}

/// # PaaS application status
/// Running status of a PaaS application.
#[derive(Debug, Default)]
pub enum ApplicationStatus {
    /// The application status couldn't be retrieved.
    #[default]
    Unknown,

    /// The application is starting or restarting.
    Starting,

    /// The application is running.
    Running,

    /// The application is stopping.
    Stopping,

    /// The application has exited, is dead or is paused.
    Stopped,
}
