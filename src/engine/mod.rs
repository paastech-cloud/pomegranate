pub mod docker_engine;

use async_trait::async_trait;

use crate::application::Application;
use crate::errors::Error;

#[async_trait]
pub trait Engine {
    async fn start_application(&self, app: &Application) -> Result<(), Error>;
    async fn stop_application(&self, project_id: &str, application_id: &str) -> Result<(), Error>;
    async fn is_application_running(
        &self,
        project_id: &str,
        application_id: &str,
    ) -> Result<bool, Error>;
}
