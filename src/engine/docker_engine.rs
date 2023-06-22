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
                return Err(Error::ApplicationCannotStart {
                    source: Box::new(e),
                });
            }
        };

        // Create the Docker container configuration
        let options = Some(CreateContainerOptions {
            name: Self::build_container_name(&app.project_id, &app.application_id),
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
                return Err(Error::ApplicationCannotStart {
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
                return Err(Error::ApplicationCannotStart {
                    source: Box::new(e),
                })
            }
        }
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

        match self
            .docker
            .stop_container(&container_name, stop_options)
            .await
        {
            Ok(_) => (),
            Err(e) => {
                return Err(Error::ApplicationCannotStop {
                    source: Box::new(e),
                });
            }
        }

        // Destroy the application
        let remove_options = Some(RemoveContainerOptions {
            ..Default::default()
        });

        trace!(
            "Removing container: {} (options: {:?})",
            container_name,
            remove_options,
        );

        match self
            .docker
            .remove_container(&container_name, remove_options)
            .await
        {
            Ok(_) => (),
            Err(e) => {
                return Err(Error::ApplicationCannotStop {
                    source: Box::new(e),
                });
            }
        }

        Ok(())
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
                        Ok(false)
                    } else {
                        Err(Error::ApplicationStateUnavailable {
                            source: Box::new(e),
                        })
                    }
                } else {
                    Err(Error::ApplicationStateUnavailable {
                        source: Box::new(e),
                    })
                }
            }
        }
    }
}
