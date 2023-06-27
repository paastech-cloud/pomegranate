pub mod docker_engine;

use async_trait::async_trait;

use crate::application::Application;
use crate::errors::Error;

/// # Execution engine
/// A trait for an execution engine that can start PaaS applications.
#[async_trait]
pub trait Engine {
    /// # Start application
    /// Start a PaaS application.
    ///
    /// # Arguments
    /// - [Application](Application) struct.
    ///
    /// # Returns
    /// - Nothing, wrapped in a Result.
    async fn start_application(&self, app: &Application) -> Result<(), Error>;

    /// # Stop application
    /// Stop a PaaS application.
    ///
    /// # Arguments
    /// - ID of the project that the application is a part of.
    /// - ID of the application.
    ///
    /// # Returns
    /// - Nothing, wrapped in a Result.
    async fn stop_application(&self, project_id: &str, application_id: &str) -> Result<(), Error>;

    /// # Is application running
    /// Get the running status of a PaaS application.
    ///
    /// # Arguments
    /// - ID of the project that the application is a part of.
    /// - ID of the application.
    ///
    /// # Returns
    /// - Whether the application is running, wrapped in a Result.
    async fn is_application_running(
        &self,
        project_id: &str,
        application_id: &str,
    ) -> Result<bool, Error>;

    /// # Restart application
    /// Restart a PaaS application.
    ///
    /// # Arguments
    /// - [Application](Application) struct.
    ///
    /// # Returns
    /// - Nothing, wrapped in a Result.
    async fn restart_application(&self, app: &Application) -> Result<(), Error> {
        // Try to stop the application
        self.stop_application(&app.project_id, &app.application_id)
            .await
            .ok();

        // Start the application
        self.start_application(app).await
    }
}
