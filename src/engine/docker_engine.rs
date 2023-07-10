use anyhow::{Context, Result};
use async_trait::async_trait;
use bollard::container::{
    Config, CreateContainerOptions, InspectContainerOptions, LogsOptions, RemoveContainerOptions,
    StartContainerOptions, StatsOptions, StopContainerOptions,
};
use bollard::image::RemoveImageOptions;
use bollard::network::{ConnectNetworkOptions, CreateNetworkOptions, ListNetworksOptions};
use bollard::service::{ContainerStateStatusEnum, EndpointSettings};
use bollard::Docker;
use bytes::Bytes;
use futures::stream::BoxStream;
use futures::StreamExt;
use log::{error, info, trace};
use std::collections::HashMap;

use super::errors::EngineError;
use super::Engine;
use crate::application::{ApplicationStats, ApplicationStatus};
use crate::config::application_config::ApplicationConfig;
use crate::config::traefik_config::TraefikConfig;
use crate::Application;

/// # Docker execution engine
/// Implementation of an execution engine that uses Docker as a backend.
pub struct DockerEngine {
    /// The Docker driver.
    docker: Docker,
    /// The application config, see [Config](Config)
    config: ApplicationConfig,
}

impl DockerEngine {
    /// # New
    /// Create an instance of the Docker execution engine.
    pub async fn new() -> Self {
        // Attempt to connect to the Docker engine
        info!("Creating new Docker engine");

        let docker = match Docker::connect_with_socket_defaults() {
            Ok(v) => v,
            Err(e) => panic!("Unable to connect to the Docker engine: {:?}", e),
        };

        let config = ApplicationConfig::from_env();

        // Create the network if needed
        let list_options: ListNetworksOptions<&str> = ListNetworksOptions {
            filters: HashMap::from([("name", vec![config.traefik_config.network_name.as_ref()])]),
        };

        match docker.list_networks(Some(list_options)).await {
            Ok(v) if v.len() == 0 => {
                // Create the fallback network
                let network_options: CreateNetworkOptions<&str> = CreateNetworkOptions {
                    name: &config.traefik_config.network_name,
                    ..Default::default()
                };

                let result_network_creation = docker.create_network(network_options).await;
                match result_network_creation {
                    Ok(_) => trace!(
                        "Docker network created: {}",
                        &config.traefik_config.network_name
                    ),
                    Err(_) => error!(
                        "Failed to create Docker network: {}",
                        &config.traefik_config.network_name
                    ),
                }
            }
            Ok(_) => (),
            Err(_) => error!("Failed to list Docker networks"),
        }

        DockerEngine { docker, config }
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
    fn build_container_name(container_name: &str) -> String {
        format!("client-app_{}", container_name)
    }
}

#[async_trait]
impl Engine for DockerEngine {
    async fn start_application(&self, app: &Application) -> Result<(), EngineError> {
        // Create the Docker container configuration
        let options = Some(CreateContainerOptions {
            name: Self::build_container_name(&app.container_name),
            ..Default::default()
        });

        let config: Config<String> = Config {
            image: Some(app.image_name.clone()),
            env: Some(
                app.env_variables
                    .iter()
                    .map(|(k, v)| format!("{}={}", k, v))
                    .collect(),
            ),

            labels: Some(build_traefik_labels(app, &self.config.traefik_config)),
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
                    "Failed to create the container for application {}",
                    app.container_name
                )
            })?;

        // Once the container is created, connect it to the traefik network
        self.docker
            .connect_network(
                &self.config.traefik_config.network_name,
                ConnectNetworkOptions {
                    container: &container_id,
                    endpoint_config: EndpointSettings::default(),
                },
            )
            .await
            .with_context(|| {
                format!(
                    "Failed to attach the container for application {} to network {}",
                    app.container_name, self.config.traefik_config.network_name,
                )
            })?;

        self.docker
            .start_container(&container_id, None::<StartContainerOptions<String>>)
            .await
            .with_context(|| {
                format!(
                    "Failed to start the container for application {}",
                    app.container_name
                )
            })?;

        Ok(())
    }

    async fn stop_application(&self, container_name: &str) -> Result<(), EngineError> {
        // Stop the application
        let container_name = Self::build_container_name(container_name);
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
                    "Failed to stop the container for application {}",
                    container_name
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
                    "Failed to remove the container for application {}",
                    container_name
                )
            })?;

        Ok(())
    }

    async fn get_application_status(
        &self,
        container_name: &str,
    ) -> Result<ApplicationStatus, EngineError> {
        // Inspect the container
        let options = Some(InspectContainerOptions { size: false });

        match self
            .docker
            .inspect_container(&Self::build_container_name(container_name), options)
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

                Err(e)
                    .with_context(|| {
                        format!(
                            "Failed to get the status for application {}",
                            container_name
                        )
                    })
                    .map_err(|e| e.into())
            }
        }
    }

    fn get_logs(&self, container_name: &str) -> BoxStream<Result<Bytes, EngineError>> {
        // Get the logs
        let options = Some(LogsOptions::<String> {
            stdout: true,
            stderr: true,
            ..Default::default()
        });

        self.docker
            .logs(&Self::build_container_name(container_name), options)
            .map(|item| {
                // Map the item to have the correct type
                item.map(|v| v.into_bytes()).map_err(|e| e.into())
            })
            .boxed()
    }

    async fn get_stats(
        &self,
        container_name: &str,
    ) -> Result<Option<ApplicationStats>, EngineError> {
        // Get the stats
        let options = Some(StatsOptions {
            stream: false,
            one_shot: false,
        });

        self.docker
            .stats(&Self::build_container_name(container_name), options)
            .next()
            .await
            .transpose()
            .map(|item| {
                item.map(|v| {
                    // Compute the CPU percent
                    let cpu_delta = (v.cpu_stats.cpu_usage.total_usage as f64)
                        - (v.precpu_stats.cpu_usage.total_usage as f64);
                    let system_delta = (v.cpu_stats.system_cpu_usage.unwrap_or_default() as f64)
                        - (v.precpu_stats.system_cpu_usage.unwrap_or_default() as f64);

                    let cpu_percent = if cpu_delta > 0.0 && system_delta > 0.0 {
                        Some(
                            (cpu_delta / system_delta)
                                * (v.cpu_stats.online_cpus.unwrap_or_default() as f64)
                                * 100.0,
                        )
                    } else {
                        Some(0.0)
                    };

                    ApplicationStats {
                        memory_usage: v.memory_stats.usage,
                        memory_limit: v.memory_stats.limit,
                        cpu_usage: cpu_percent,
                    }
                })
            })
            .with_context(|| {
                format!(
                    "Failed to get the statistics for application {}",
                    container_name
                )
            })
            .map_err(|err| err.into())
    }

    async fn remove_application_image(&self, app: &Application) -> Result<(), EngineError> {
        // Remove the image from the cache
        let options = Some(RemoveImageOptions {
            ..Default::default()
        });

        self.docker
            .remove_image(
                format!("{}:{}", app.image_name, app.image_tag).as_str(),
                options,
                None,
            )
            .await
            .map(|_| {})
            .with_context(|| {
                format!(
                    "Failed to remove the image {}:{} for application {}",
                    app.image_name, app.image_tag, app.container_name
                )
            })
            .map_err(|e| e.into())
    }
}

/// # Build Traefik Labels
/// Build the labels necessary for network routing, perhaps a middleware system would be better
///
/// This function will always try to re-route to the port 80
///
/// # Arguments
/// - [Application](Application) struct.
/// - [TraefikConfig](TraefikConfig) struct
///
/// # Returns
/// - A HashMap<String, String>
fn build_traefik_labels(
    app: &Application,
    traefik_config: &TraefikConfig,
) -> HashMap<String, String> {
    HashMap::from([
        ("traefik.enable".into(), "true".into()),
        (
            format!("traefik.http.routers.{}.entrypoints", app.container_name),
            "websecure".into(),
        ),
        (
            format!(
                "traefik.http.services.{}.loadbalancer.server.port",
                app.container_name
            ),
            "80".into(),
        ),
        (
            format!("traefik.http.routers.{}.rule", app.container_name),
            format!(
                "Host(`{}.user-app.{}`)",
                app.container_name, traefik_config.fqdn
            ),
        ),
    ])
}
