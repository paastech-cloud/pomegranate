pub mod docker_engine;

use anyhow::Result;
use async_trait::async_trait;
use bytes::Bytes;
use futures::stream::BoxStream;

use crate::application::{Application, ApplicationStats, ApplicationStatus};

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
    ///     container_name: String::from("bob_webapp"),
    ///     image_name: String::from("nginx"),
    ///     image_tag: String::from("latest"),
    ///     ..Default::default()
    /// };
    ///
    /// let engine = MyEngine::new();
    /// engine.start_application(&app).await.unwrap();
    /// ```
    async fn start_application(&self, app: &Application) -> Result<()>;

    /// # Stop application
    /// Stop a PaaS application.
    ///
    /// # Arguments
    /// - Container name of the application
    ///
    /// # Returns
    /// - Nothing, wrapped in a Result.
    ///
    /// # Example
    /// ```
    /// let engine = MyEngine::new();
    /// engine.stop_application(&String::from("test"), &String::from("webapp")).await.unwrap();
    /// ```
    async fn stop_application(&self, container_name: &str) -> Result<()>;

    /// # Get application status
    /// Get the running status of a PaaS application.
    ///
    /// # Arguments
    /// - Container name of the application
    ///
    /// # Returns
    /// - The running status of the application, wrapped in a Result.
    ///
    /// # Example
    /// ```
    /// let engine = MyEngine::new();
    /// let running = engine.get_application_status(&String::from("test"), &String::from("webapp"))
    ///     .await
    ///     .unwrap();
    /// ```
    async fn get_application_status(&self, container_name: &str) -> Result<ApplicationStatus>;

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
    async fn restart_application(&self, app: &Application) -> Result<()> {
        // Try to stop the application
        self.stop_application(&app.container_name).await.ok();

        // Start the application
        self.start_application(app).await
    }

    /// # Get logs
    /// Retrieve the logs from the stdout and stderr streams of a PaaS application.
    ///
    /// This function returns the logs that were generated by the application since it started
    /// and until this function was called.
    ///
    /// # Arguments
    /// - Container name of the application
    ///
    /// # Returns
    /// A stream of bytes corresponding to the logs, wrapped in a Result.
    ///
    /// # Example
    /// ```
    /// let engine = MyEngine::new();
    ///
    /// engine.get_logs(&String::from("test"), &String::from("webapp"))
    ///     .for_each(|item| {
    ///         println!("{:?}", item.unwrap());
    ///         future::ready(())
    ///     })
    ///     .await;
    /// ```
    fn get_logs(&self, container_name: &str) -> BoxStream<Result<Bytes>>;

    /// # Get stats
    /// Get the resource usage statistics of a PaaS application.
    ///
    /// # Arguments
    /// - Container name of the application
    ///
    /// # Returns
    /// - The statistics, wrapped in a Result.
    ///
    /// # Example
    /// ```
    /// let engine = MyEngine::new();
    /// let stats = engine.get_stats(&String::from("test"), &String::from("webapp"))
    ///     .await
    ///     .unwrap();
    ///
    /// println!("{:#?}", stats);
    /// ```
    async fn get_stats(&self, container_name: &str) -> Result<Option<ApplicationStats>>;

    /// # Remove application image
    /// Remove the container image of a PaaS application from the local cache.
    ///
    /// The image is *NOT* removed from the cache if still used by at least one container.
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
    /// engine.remove_application_image(&app)
    ///     .await
    ///     .unwrap();
    /// ```
    async fn remove_application_image(&self, app: &Application) -> Result<()>;
}
