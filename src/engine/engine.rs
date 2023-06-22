use async_trait::async_trait;

use crate::application::Application;
use crate::errors::ApplicationStartError;

#[async_trait]
pub trait Engine {
    async fn start_application(&self, app: &Application) -> Result<(), ApplicationStartError>;
}
