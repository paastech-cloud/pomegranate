pub mod docker_engine;

use async_trait::async_trait;

use crate::application::Application;
use crate::errors::{ApplicationIsRunningError, ApplicationStartError, ApplicationStopError};

#[async_trait]
pub trait Engine {
    async fn start_application(&self, app: &Application) -> Result<(), ApplicationStartError>;
    async fn stop_application(
        &self,
        project_id: &str,
        application_id: &str,
    ) -> Result<(), ApplicationStopError>;
    async fn is_application_running(
        &self,
        project_id: &str,
        application_id: &str,
    ) -> Result<bool, ApplicationIsRunningError>;
}
