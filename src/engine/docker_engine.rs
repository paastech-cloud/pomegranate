use anyhow::{Context, Result};
use async_trait::async_trait;
use bollard::container::{
    Config, CreateContainerOptions, InspectContainerOptions, RemoveContainerOptions,
    StartContainerOptions, StopContainerOptions,
};
use bollard::image::CreateImageOptions;
use bollard::service::ContainerStateStatusEnum;
use bollard::Docker;
use futures::stream::TryStreamExt;
use log::{info, trace};

use super::Engine;
use crate::application::ApplicationStatus;
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
    async fn start_application(&self, app: &Application) -> Result<()> {
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
            .with_context(|| {
                format!(
                    "Failed to create the image for application {}/{}",
                    app.project_id, app.application_id
                )
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
            .with_context(|| {
                format!(
                    "Failed to create the container for application {}/{}",
                    app.project_id, app.application_id
                )
            })?;

        self.docker
            .start_container(&container_id, None::<StartContainerOptions<String>>)
            .await
            .with_context(|| {
                format!(
                    "Failed to start the container for application {}/{}",
                    app.project_id, app.application_id
                )
            })?;

        Ok(())
    }

    async fn stop_application(&self, project_id: &str, application_id: &str) -> Result<()> {
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
            .with_context(|| {
                format!(
                    "Failed to stop the container for application {}/{}",
                    project_id, application_id
                )
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
            .with_context(|| {
                format!(
                    "Failed to remove the container for application {}/{}",
                    project_id, application_id
                )
            })?;

        Ok(())
    }

    async fn get_application_status(
        &self,
        project_id: &str,
        application_id: &str,
    ) -> Result<ApplicationStatus> {
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
            Ok(v) => {
                // Try to get the state of the container
                if let Some(state) = v.state {
                    if let Some(status) = state.status {
                        return match status {
                            ContainerStateStatusEnum::CREATED => Ok(ApplicationStatus::Starting),
                            ContainerStateStatusEnum::RUNNING => Ok(ApplicationStatus::Running),
                            ContainerStateStatusEnum::PAUSED => Ok(ApplicationStatus::Stopped),
                            ContainerStateStatusEnum::RESTARTING => Ok(ApplicationStatus::Starting),
                            ContainerStateStatusEnum::REMOVING => Ok(ApplicationStatus::Stopping),
                            ContainerStateStatusEnum::EXITED => Ok(ApplicationStatus::Stopped),
                            ContainerStateStatusEnum::DEAD => Ok(ApplicationStatus::Stopped),
                            _ => Ok(ApplicationStatus::Unknown),
                        };
                    }
                }

                // Couldn't get the status of the application
                Ok(ApplicationStatus::Unknown)
            }
            Err(e) => {
                // Only return an OK result if Docker returned a 404
                if let bollard::errors::Error::DockerResponseServerError { status_code, .. } = e {
                    if status_code == 404 {
                        return Ok(ApplicationStatus::Stopped);
                    }
                }

                Err(e).with_context(|| {
                    format!(
                        "Failed to get the status for application {}/{}",
                        project_id, application_id
                    )
                })
            }
        }
    }
}
