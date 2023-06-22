use async_trait::async_trait;
use bollard::container::{Config, CreateContainerOptions, StartContainerOptions};
use bollard::image::CreateImageOptions;
use bollard::Docker;
use futures::stream::TryStreamExt;
use log::{info, trace};

use super::engine::Engine;
use crate::errors::ApplicationStartError;
use crate::Application;

pub struct DockerEngine {
    docker: Docker,
}

impl DockerEngine {
    pub fn new() -> Self {
        // Attempt to connect to the Docker engine
        info!("Creating new Docker engine");

        let docker = match Docker::connect_with_socket_defaults() {
            Ok(v) => v,
            Err(e) => panic!("Unable to connect to the Docker engine: {:?}", e),
        };

        DockerEngine { docker }
    }
}

#[async_trait]
impl Engine for DockerEngine {
    async fn start_application(&self, app: &Application) -> Result<(), ApplicationStartError> {
        // Create the image
        trace!(
            "Pulling container image: {}:{}",
            app.image_name,
            app.image_tag
        );

        match self
            .docker
            .create_image(
                Some(CreateImageOptions {
                    from_image: app.image_name.clone(),
                    tag: app.image_tag.clone(),
                    ..Default::default()
                }),
                None,
                None,
            )
            .try_collect::<Vec<_>>()
            .await
        {
            Ok(_) => (),
            Err(e) => {
                return Err(ApplicationStartError {
                    source: Box::new(e),
                })
            }
        };

        // Create the Docker container configuration
        let options = Some(CreateContainerOptions {
            name: format!("{}-{}", app.project_id, app.application_id),
            ..Default::default()
        });

        let config = Config {
            image: Some(app.image_name.clone()),
            env: Some(
                app.env_variables
                    .clone()
                    .into_iter()
                    .map(|(k, v)| format!("{}={}", k, v))
                    .collect(),
            ),
            ..Default::default()
        };

        // Start the application
        trace!(
            "Starting app: {:?} (options: {:?}, config: {:?})",
            app,
            options,
            config
        );

        let container_id = match self.docker.create_container(options, config).await {
            Ok(v) => v.id,
            Err(e) => {
                return Err(ApplicationStartError {
                    source: Box::new(e),
                })
            }
        };

        match self
            .docker
            .start_container(&container_id, None::<StartContainerOptions<String>>)
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => {
                return Err(ApplicationStartError {
                    source: Box::new(e),
                })
            }
        }
    }
}
