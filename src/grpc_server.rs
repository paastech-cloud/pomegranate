use bytes::Bytes;
use futures::StreamExt;
use log::{info, trace};
use std::collections::HashMap;
use std::str::from_utf8;
use tonic::codegen::http::StatusCode;
use tonic::{transport::Server, Request, Response, Status};

use crate::application::{Application, ApplicationStatus};
use crate::engine::docker_engine::DockerEngine;
use crate::engine::errors::EngineError;
use crate::engine::Engine;
use paastech_proto::pomegranate::get_status_response::SingleContainerStatus;
use paastech_proto::pomegranate::pomegranate_server::{Pomegranate, PomegranateServer};
use paastech_proto::pomegranate::{
    DeleteImageRequest, DeployRequest, EmptyResponse, GetLogsRequest, GetLogsResponse,
    GetStatisticsRequest, GetStatisticsResponse, GetStatusRequest, GetStatusResponse,
    StopDeployRequest,
};

/// # Pomegranate gRPC server
/// The gRPC server that implements the Pomegranate routes.
pub struct PomegranateGrpcServer {
    docker_engine: DockerEngine,
}

#[tonic::async_trait]
impl Pomegranate for PomegranateGrpcServer {
    /// # Start Deployment
    /// Start a deployment from its `uuid`, `project_uuid` and `user_uuid`.
    /// # Arguments
    /// The request containing the `uuid` of the deployment to start.
    /// # Returns
    /// A message indicating the deployment was started, wrapped in a Result.
    /// # Errors
    /// If the deployment does not exist, returns a `not_found` error
    /// If the deployment failed to start, returns an `internal` error
    async fn deploy(
        &self,
        request: Request<DeployRequest>,
    ) -> Result<Response<EmptyResponse>, Status> {
        let request = request.into_inner();
        let container_name = request.container_name;
        let image_name = request.image_name;
        let image_tag = request.image_tag;
        let env_vars = request.env_vars;

        trace!("Getting app...");

        let app = get_app(&container_name, &image_name, &image_tag, env_vars);

        trace!("Getting app status of {}", container_name);

        match self
            .docker_engine
            .get_application_status(&container_name)
            .await
        {
            Ok(status) => {
                trace!(
                    "Got app status of {} : {:?}, restarting it...",
                    container_name, status
                );
                match self.docker_engine.restart_application(&app).await {
                    Ok(_) => trace!("Restarted app {}", container_name),
                    Err(e) => {
                        trace!("Failed to restart app {}: {}", container_name, e);
                        return Err(translate_err(
                            &e,
                            format!("Failed to restart application {}: {}", container_name, e),
                        ));
                    }
                };
            }
            Err(_) => {
                trace!(
                    "Failed to get app status of {}, starting it",
                    container_name
                );
                match self.docker_engine.start_application(&app).await {
                    Ok(_) => trace!("Started app {}", container_name),
                    Err(e) => {
                        trace!("Failed to start app {}: {}", container_name, e);
                        return Err(translate_err(
                            &e,
                            format!("Failed to start application {}: {}", container_name, e),
                        ));
                    }
                };
            }
        };

        Ok(Response::new(EmptyResponse {}))
    }

    /// # Stop Deployment
    /// Stop a deployment from its `uuid` and `project_uuid`.
    /// # Arguments
    /// The request containing the `uuid` of the deployment to stop.
    /// # Returns
    /// A message indicating the deployment was stopped, wrapped in a Result.
    /// # Errors
    /// If the deployment does not exist, returns a `not_found` error.
    /// If the deployment failed to stop, returns an `internal` error.
    async fn stop_deploy(
        &self,
        request: Request<StopDeployRequest>,
    ) -> Result<Response<EmptyResponse>, Status> {
        let request = request.into_inner();
        let container_name = request.container_name;

        match self.docker_engine.stop_application(&container_name).await {
            Ok(_) => {
                trace!("Stopped app {}", container_name);
            }
            Err(e) => {
                trace!("Failed to stop app {}: {}", container_name, e);
                return Err(translate_err(
                    &e,
                    format!("Failed to stop application {}: {}", container_name, e),
                ));
            }
        };

        Ok(Response::new(EmptyResponse {}))
    }

    /// # Delete Deployment
    /// Delete a configuration to a deployment from its `uuid`, `project_uuid` and `user_uuid`.
    /// # Arguments
    /// The request containing the `uuid` of the deployment to delete.
    /// # Returns
    /// A message indicating the deployment was deleted, wrapped in a Result.
    /// # Errors
    /// If the deployment does not exist, returns a `not_found` error.
    /// If the deployment could not be deleted, returns an `internal` error.
    async fn delete_image(
        &self,
        request: Request<DeleteImageRequest>,
    ) -> Result<Response<EmptyResponse>, Status> {
        let request = request.into_inner();
        let container_name = request.container_name;
        let image_name = request.image_name;
        let image_tag = request.image_tag;

        trace!("Getting app...");
        let app = get_app(&container_name, &image_name, &image_tag, HashMap::new());

        match self.docker_engine.stop_application(&container_name).await {
            Ok(_) => {
                trace!("Stopped app {}", container_name);
            }
            Err(e) => {
                trace!("Could not stop app {}: {}", container_name, e);
            }
        };

        match self.docker_engine.remove_application_image(&app).await {
            Ok(_) => {
                trace!("Deleted app {}", container_name);
            }
            Err(e) => {
                trace!("Failed to delete app {}: {}", container_name, e);
                return Err(translate_err(
                    &e,
                    format!("Failed to delete application {}: {}", container_name, e),
                ));
            }
        };

        Ok(Response::new(EmptyResponse {}))
    }

    /// # Deployment Logs
    /// Get the logs of a deployment from its `uuid` and `project_uuid`.
    /// # Arguments
    /// The request containing the `uuid` and `project_uuid` of the deployment to get the logs of.
    /// # Returns
    /// The logs of the deployment, wrapped in a Result.
    async fn get_logs(
        &self,
        request: Request<GetLogsRequest>,
    ) -> Result<Response<GetLogsResponse>, Status> {
        let request = request.into_inner();
        let container_name = request.container_name;

        trace!("Getting logs of app {}", container_name);

        let logs: Vec<Result<Bytes, _>> =
            self.docker_engine.get_logs(&container_name).collect().await;

        if let Some(Err(err)) = logs.last() {
            return Err(translate_err(
                &err,
                format!(
                    "Failed to get logs of application {}: {}",
                    container_name, err
                ),
            ));
        }

        let output = logs
            .iter()
            .map(|item| match item {
                Ok(value) => from_utf8(value).unwrap_or_default().to_string(),
                Err(err) => format!("Error: {:?}", err),
            })
            .collect::<Vec<String>>()
            .join("\n");

        let response = GetLogsResponse { logs: output };
        Ok(Response::new(response))
    }

    /// # Deployment Stats
    /// Get the stats of a deployment from its `uuid` and `project_uuid`.
    /// # Arguments
    /// The request containing the `uuid` and `project_uuid` of the deployment to get the stats of.
    /// # Returns
    /// The stats of the deployment, wrapped in a Result.
    async fn get_statistics(
        &self,
        request: Request<GetStatisticsRequest>,
    ) -> Result<Response<GetStatisticsResponse>, Status> {
        let request = request.into_inner();
        let container_name = request.container_name;

        trace!("Getting stats of app {}", container_name);

        let stats = self
            .docker_engine
            .get_stats(&container_name)
            .await
            .map_err(|e| {
                trace!("Failed to get stats of app {}: {}", container_name, e);
                translate_err(
                    &e,
                    format!(
                        "Failed to get stats of application {}: {}",
                        container_name, e
                    ),
                )
            })?
            .ok_or(Status::not_found(format!(
                "Failed to get stats of application {}: {}",
                container_name, "Not found"
            )))?;

        let response = GetStatisticsResponse {
            cpu_usage: stats.cpu_usage.unwrap_or_default(),
            memory_usage: stats.memory_usage.unwrap_or_default(),
            memory_limit: stats.memory_limit.unwrap_or_default(),
        };
        Ok(Response::new(response))
    }

    /// # Deployment Status
    /// Get the status of a deployment from its `uuid` and `project_uuid`.
    /// # Arguments
    /// The request containing the `uuid` of the deployment to get the status of.
    /// # Returns
    /// The status of the deployment, wrapped in a Result.
    /// # Errors
    /// If the deployment does not exist, returns a `not_found` error.
    /// If the deployment status could not be checked, returns an `internal` error.
    async fn get_status(
        &self,
        request: Request<GetStatusRequest>,
    ) -> Result<Response<GetStatusResponse>, Status> {
        let request = request.into_inner();
        let container_name = request.container_name;

        let mut output: Vec<SingleContainerStatus> = vec![];

        for c in container_name.iter() {
            trace!("Getting status of app {}", c);

            match self.docker_engine.get_application_status(c).await {
                Ok(status) => {
                    trace!("Status of app {}: {:?}", c, status);
                    let mapped_status = match status {
                        ApplicationStatus::Unknown => 0,
                        ApplicationStatus::Starting => 1,
                        ApplicationStatus::Running => 2,
                        ApplicationStatus::Stopping => 3,
                        ApplicationStatus::Stopped => 4,
                    };

                    output.push(SingleContainerStatus {
                        container_name: c.to_string(),
                        container_status: mapped_status,
                    });
                }
                Err(e) => {
                    trace!("Failed to get status of app {}: {}", c, e);
                }
            };
        }

        let response = GetStatusResponse {
            container_statuses: output,
        };
        Ok(Response::new(response))
    }
}

/// # Get App from db with uuid
/// Get an `Application` from the database from its `uuid`, `project_uuid` and `user_uuid`.
/// # Arguments
/// - The borrowed `container_name` of the app.
/// - The borrowed `image_name` of the app.
/// - The `env_vars` of the app.
/// # Returns
/// The application, wrapped in a Result.
/// # Errors
/// If the application is not found in the database, returns a `Status::not_found`.
fn get_app(
    container_name: &str,
    image_name: &str,
    image_tag: &str,
    env_vars: HashMap<String, String>,
) -> Application {
    Application {
        container_name: String::from(container_name),
        image_name: String::from(image_name),
        image_tag: String::from(image_tag),
        env_variables: env_vars,
    }
}

/// # Translate Error
/// Map the EngineError to a gRPC Status
/// # Returns
/// A gRPC `Status`
fn translate_err(err: &EngineError, message: String) -> Status {
    return match err.code {
        StatusCode::NOT_FOUND => Status::not_found(message),
        StatusCode::BAD_REQUEST => Status::invalid_argument(message),
        StatusCode::CONFLICT => Status::already_exists(message),
        StatusCode::FORBIDDEN => Status::permission_denied(message),
        StatusCode::UNAUTHORIZED => Status::unauthenticated(message),
        StatusCode::SERVICE_UNAVAILABLE => Status::unavailable(message),
        _ => Status::internal(message),
    };
}

/// # Start Server
/// Start the gRPC server.
/// # Arguments
/// - The Docker engine to use to manage containers.
/// - The database reference to use to get the deployments.
/// # Returns
/// Nothing.
pub async fn start_server(docker_engine: DockerEngine) -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::]:50051".parse()?;

    let pomegranate_grpc_server = PomegranateGrpcServer { docker_engine };

    info!("gRPC server started on {}", addr);

    Server::builder()
        .add_service(PomegranateServer::new(pomegranate_grpc_server))
        .serve(addr)
        .await?;

    Ok(())
}
