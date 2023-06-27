use async_trait::async_trait;
use bollard::container::{
    Config, CreateContainerOptions, InspectContainerOptions, RemoveContainerOptions,
    StartContainerOptions, StopContainerOptions,
};
use bollard::image::CreateImageOptions;
use bollard::Docker;
use futures::stream::TryStreamExt;
use log::{info, trace};

use super::Engine;
use crate::errors::Error;
use crate::Application;

/// # Docker execution engine
/// Implementation of an execution engine that uses Docker as a backend.
pub struct DockerEngine {
    /// The Docker driver.
    docker: Docker,
}

impl DockerEngine {
    /// # New
    /// Create an instance of the Docker execution engine.
    pub fn new() -> Self {
        // Attempt to connect to the Docker engine
        info!("Creating new Docker engine");

        let docker = match Docker::connect_with_socket_defaults() {
            Ok(v) => v,
            Err(e) => panic!("Unable to connect to the Docker engine: {:?}", e),
        };

        DockerEngine { docker }
    }

    /// # Build container name
    /// Construct the name of the container associated with a PaaS application.
    ///
    /// # Arguments
    /// - ID of the project that the application is a part of.
    /// - ID of the application.
    ///
    /// # Returns
    /// - The name of the associated container.
    fn build_container_name(project_id: &str, application_id: &str) -> String {
        format!("client-app_{}_{}", project_id, application_id)
    }
}

#[async_trait]
impl Engine for DockerEngine {
    async fn start_application(&self, app: &Application) -> Result<(), Error> {
        // Create the image
        trace!(
            "Pulling container image: {}:{}",
            app.image_name,
            app.image_tag
        );

        self.docker
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
            .map_err(|e| Error::ApplicationCannotStart {
                source: Box::new(e),
            })?;

        // Create the Docker container configuration
        let options = Some(CreateContainerOptions {
            name: Self::build_container_name(&app.project_id, &app.application_id),
            ..Default::default()
        });

        let config = Config {
            image: Some(app.image_name.clone()),
            env: Some(
                app.env_variables
                    .iter()
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

        let container_id = self
            .docker
            .create_container(options, config)
            .await
            .map(|v| v.id)
            .map_err(|e| Error::ApplicationCannotStart {
                source: Box::new(e),
            })?;

        self.docker
            .start_container(&container_id, None::<StartContainerOptions<String>>)
            .await
            .map_err(|e| Error::ApplicationCannotStart {
                source: Box::new(e),
            })
    }

    async fn stop_application(&self, project_id: &str, application_id: &str) -> Result<(), Error> {
        // Stop the application
        let container_name = Self::build_container_name(project_id, application_id);
        let stop_options = Some(StopContainerOptions { t: 10 });

        trace!(
            "Stopping container: {} (options: {:?})",
            container_name,
            stop_options,
        );

        self.docker
            .stop_container(&container_name, stop_options)
            .await
            .map_err(|e| Error::ApplicationCannotStop {
                source: Box::new(e),
            })?;

        // Destroy the application
        let remove_options = Some(RemoveContainerOptions {
            ..Default::default()
        });

        trace!(
            "Removing container: {} (options: {:?})",
            container_name,
            remove_options,
        );

        self.docker
            .remove_container(&container_name, remove_options)
            .await
            .map_err(|e| Error::ApplicationCannotStop {
                source: Box::new(e),
            })
    }

    async fn is_application_running(
        &self,
        project_id: &str,
        application_id: &str,
    ) -> Result<bool, Error> {
        // Inspect the container
        let options = Some(InspectContainerOptions { size: false });

        match self
            .docker
            .inspect_container(
                &Self::build_container_name(project_id, application_id),
                options,
            )
            .await
        {
            Ok(_) => Ok(true),
            Err(e) => {
                // Only return an OK result if Docker returned a 404
                if let bollard::errors::Error::DockerResponseServerError { status_code, .. } = e {
                    if status_code == 404 {
                        return Ok(false);
                    }
                }

                Err(Error::ApplicationStateUnavailable {
                    source: Box::new(e),
                })
            }
        }
    }
}
