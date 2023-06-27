pub mod docker_engine;

use async_trait::async_trait;

use crate::application::{Application, ApplicationStatus};
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
    ///
    /// # Example
    /// ```
    /// let app = Application {
    ///     application_id: String::from("webapp"),
    ///     project_id: String::from("test"),
    ///     image_name: String::from("nginx"),
    ///     image_tag: String::from("latest"),
    ///     ..Default::default()
    /// };
    ///
    /// let engine = MyEngine::new();
    /// engine.start_application(&app).await.unwrap();
    /// ```
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
    ///
    /// # Example
    /// ```
    /// let engine = MyEngine::new();
    /// engine.stop_application(String::from("test"), String::from("webapp")).await.unwrap();
    /// ```
    async fn stop_application(&self, project_id: &str, application_id: &str) -> Result<(), Error>;

    /// # Get application status
    /// Get the running status of a PaaS application.
    ///
    /// # Arguments
    /// - ID of the project that the application is a part of.
    /// - ID of the application.
    ///
    /// # Returns
    /// - The running status of the application, wrapped in a Result.
    ///
    /// # Example
    /// ```
    /// let engine = MyEngine::new();
    /// let running = engine.get_application_status(String::from("test"), String::from("webapp"))
    ///     .await
    ///     .unwrap();
    /// ```
    async fn get_application_status(
        &self,
        project_id: &str,
        application_id: &str,
    ) -> Result<ApplicationStatus, Error>;

    /// # Restart application
    /// Restart a PaaS application.
    ///
    /// This function simply tries to stop the PaaS application, ignoring any failure to do so,
    /// and then start it again.
    ///
    /// # Arguments
    /// - [Application](Application) struct.
    ///
    /// # Returns
    /// - Nothing, wrapped in a Result.
    ///
    /// # Example
    /// ```
    /// let app = Application {
    ///     application_id: String::from("webapp"),
    ///     project_id: String::from("test"),
    ///     image_name: String::from("nginx"),
    ///     image_tag: String::from("latest"),
    ///     ..Default::default()
    /// };
    ///
    /// let engine = MyEngine::new();
    /// engine.restart_application(&app).await.unwrap();
    /// ```
    async fn restart_application(&self, app: &Application) -> Result<(), Error> {
        // Try to stop the application
        self.stop_application(&app.project_id, &app.application_id)
            .await
            .ok();

        // Start the application
        self.start_application(app).await
    }
}
