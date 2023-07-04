use bytes::Bytes;
use futures::StreamExt;
use log::{info, trace};
use std::collections::HashMap;
use std::str::from_utf8;
use tonic::{transport::Server, Request, Response, Status};

use crate::application::Application;
use crate::engine::docker_engine::DockerEngine;
use crate::engine::Engine;
use paastech_proto::pomegranate::pomegranate_server::{Pomegranate, PomegranateServer};
use paastech_proto::pomegranate::{
    ApplyConfigDeploymentRequest, DeleteDeploymentRequest, DeploymentLogRequest,
    DeploymentStatRequest, DeploymentStats, DeploymentStatusRequest, ResponseMessage,
    ResponseMessageStatus, RestartDeploymentRequest, StartDeploymentRequest, StopDeploymentRequest,
};

/// # Pomegranate gRPC server
/// The gRPC server that implements the Pomegranate routes.
pub struct PomegranateGrpcServer {
    docker_engine: DockerEngine,
}

#[tonic::async_trait]
impl Pomegranate for PomegranateGrpcServer {
    /// # Start Deployment
    /// Start a deployment from its `uuid`.
    /// # Arguments
    /// The request containing the `uuid` of the deployment to start.
    /// # Returns
    /// A message indicating the deployment was started, wrapped in a Result.
    /// # Errors
    /// If the deployment does not exist, returns a `not_found` error
    /// If the deployment failed to start, returns an `internal` error
    async fn start_deployment(
        &self,
        request: Request<StartDeploymentRequest>,
    ) -> Result<Response<ResponseMessage>, Status> {
        let request = request.into_inner();
        let deployment_uuid = request.deployment_uuid;
        let project_uuid = request.project_uuid;
        let user_uuid = request.user_uuid;

        trace!("Creating app {}", deployment_uuid);

        let app = get_app(&deployment_uuid, &project_uuid, &user_uuid);

        trace!("Starting app {}", deployment_uuid);

        let message = match self.docker_engine.start_application(&app).await {
            Ok(_) => {
                trace!("Started app {}", deployment_uuid);
                format!("Started application {}", app.project_id)
            }
            Err(e) => {
                trace!("Failed to start app {}: {}", deployment_uuid, e);
                return Err(Status::internal(format!(
                    "Failed to start application {}: {}",
                    app.project_id, e
                )));
            }
        };

        let response = ResponseMessage { message };
        Ok(Response::new(response))
    }

    /// # Restart deployment
    /// Restart a deployment from its `uuid`.
    /// # Arguments
    /// The request containing the `uuid` of the deployment to restart.
    /// # Returns
    /// A message indicating the deployment was restarted, wrapped in a Result.
    /// # Errors
    /// If the deployment does not exist, returns a `not_found` error.
    /// If the deployment failed to restart, returns an `internal` error.
    async fn restart_deployment(
        &self,
        request: Request<RestartDeploymentRequest>,
    ) -> Result<Response<ResponseMessage>, Status> {
        let request = request.into_inner();
        let deployment_uuid = request.deployment_uuid;
        let project_uuid = request.project_uuid;
        let user_uuid = request.user_uuid;

        trace!("Creating app {}", deployment_uuid);

        let app = get_app(&deployment_uuid, &project_uuid, &user_uuid);

        let message = match self.docker_engine.restart_application(&app).await {
            Ok(_) => {
                trace!("Restarted app {}", deployment_uuid);
                format!("Restarted application {}", app.project_id)
            }
            Err(e) => {
                trace!("Failed to restart app {}: {}", deployment_uuid, e);
                return Err(Status::internal(format!(
                    "Failed to restart application {}: {}",
                    app.project_id, e
                )));
            }
        };

        let response = ResponseMessage { message };
        Ok(Response::new(response))
    }

    /// # Delete Deployment
    /// Delete a configuration to a deployment from its `uuid`.
    /// # Arguments
    /// The request containing the `uuid` of the deployment to delete.
    /// # Returns
    /// A message indicating the deployment was deleted, wrapped in a Result.
    /// # Errors
    /// If the deployment does not exist, returns a `not_found` error.
    /// If the deployment could not be deleted, returns an `internal` error.
    async fn delete_deployment(
        &self,
        request: Request<DeleteDeploymentRequest>,
    ) -> Result<Response<ResponseMessage>, Status> {
        let request = request.into_inner();
        let deployment_uuid = request.deployment_uuid;
        let project_uuid = request.project_uuid;

        let message = match self
            .docker_engine
            .stop_application(project_uuid.as_str(), deployment_uuid.as_str())
            .await
        {
            Ok(_) => {
                //TODO: change message when deletion is implemented
                trace!("Stopped app {}. It was not deleted", deployment_uuid);
                format!(
                    "Stopped application {}. It was not deleted",
                    deployment_uuid
                )
            }
            Err(e) => {
                return Err(Status::internal(format!(
                    "Failed to delete application {}: {}",
                    deployment_uuid, e
                )));
            }
        };

        //TODO: Delete the app from the database & prune its image

        let response = ResponseMessage { message };
        Ok(Response::new(response))
    }

    /// # Stop Deployment
    /// Stop a deployment from its `uuid`.
    /// # Arguments
    /// The request containing the `uuid` of the deployment to stop.
    /// # Returns
    /// A message indicating the deployment was stopped, wrapped in a Result.
    /// # Errors
    /// If the deployment does not exist, returns a `not_found` error.
    /// If the deployment failed to stop, returns an `internal` error.
    async fn stop_deployment(
        &self,
        request: Request<StopDeploymentRequest>,
    ) -> Result<Response<ResponseMessage>, Status> {
        let request = request.into_inner();
        let deployment_uuid = request.deployment_uuid;
        let project_uuid = request.project_uuid;

        let message = match self
            .docker_engine
            .stop_application(project_uuid.as_str(), deployment_uuid.as_str())
            .await
        {
            Ok(_) => {
                trace!("Stopped app {}", deployment_uuid);
                format!("Stopped application {}", deployment_uuid)
            }
            Err(e) => {
                trace!("Failed to stop app {}: {}", deployment_uuid, e);
                return Err(Status::internal(format!(
                    "Failed to stop application {}: {}",
                    deployment_uuid, e
                )));
            }
        };

        let response = ResponseMessage { message };
        Ok(Response::new(response))
    }

    /// # Deployment Status
    /// Get the status of a deployment from its `uuid`.
    /// # Arguments
    /// The request containing the `uuid` of the deployment to get the status of.
    /// # Returns
    /// The status of the deployment, wrapped in a Result.
    /// # Errors
    /// If the deployment does not exist, returns a `not_found` error.
    /// If the deployment status could not be checked, returns an `internal` error.
    async fn deployment_status(
        &self,
        request: Request<DeploymentStatusRequest>,
    ) -> Result<Response<ResponseMessageStatus>, Status> {
        let request = request.into_inner();
        let deployment_uuid = request.deployment_uuid;
        let project_uuid = request.project_uuid;

        let status: (String, String) = match self
            .docker_engine
            .get_application_status(&project_uuid, &deployment_uuid)
            .await
        {
            Ok(status) => {
                trace!("Status of app {}: {:?}", deployment_uuid, status);
                (
                    format!("Application {} is {:?}", deployment_uuid, status),
                    format!("{:?}", status),
                )
            }
            Err(e) => {
                trace!("Failed to get status of app {}: {}", deployment_uuid, e);
                return Err(Status::internal(format!(
                    "Failed to get status of application {}: {}",
                    deployment_uuid, e
                )));
            }
        };

        let response = ResponseMessageStatus {
            message: status.0,
            status: status.1,
        };
        Ok(Response::new(response))
    }

    async fn deployment_log(
        &self,
        request: Request<DeploymentLogRequest>,
    ) -> Result<Response<ResponseMessage>, Status> {
        let request = request.into_inner();
        let deployment_uuid = request.deployment_uuid;
        let project_uuid = request.project_uuid;

        trace!("Getting logs of app {}", deployment_uuid);
        let logs: Vec<Result<Bytes, _>> = self
            .docker_engine
            .get_logs(&project_uuid, &deployment_uuid)
            .collect()
            .await;

        let output = logs
            .iter()
            .map(|item| match item {
                Ok(value) => from_utf8(value).unwrap().to_string(),
                Err(err) => format!("Error: {:?}", err),
            })
            .collect::<Vec<String>>()
            .join("\n");

        let response = ResponseMessage { message: output };

        Ok(Response::new(response))
    }

    async fn deployment_stat(
        &self,
        request: Request<DeploymentStatRequest>,
    ) -> Result<Response<DeploymentStats>, Status> {
        let request = request.into_inner();
        let deployment_uuid = request.deployment_uuid;
        let project_uuid = request.project_uuid;

        trace!("Getting stats of app {}", deployment_uuid);

        let stats = self
            .docker_engine
            .get_stats(&project_uuid, &deployment_uuid)
            .await
            .map_err(|e| {
                trace!("Failed to get stats of app {}: {}", deployment_uuid, e);
                Status::internal(format!(
                    "Failed to get stats of application {}: {}",
                    deployment_uuid, e
                ))
            })?
            .ok_or(Status::not_found(format!(
                "Failed to get stats of application {}: {}",
                deployment_uuid, "Not found"
            )))?;

        let response = DeploymentStats {
            message: format!("Stats of app {}", deployment_uuid),
            cpu_usage: stats.cpu_usage.unwrap_or_default(),
            memory_usage: stats.memory_usage.unwrap_or_default(),
            memory_limit: stats.memory_limit.unwrap_or_default(),
        };
        Ok(Response::new(response))
    }

    /// # Apply Config Deployment
    /// Apply a configuration to a deployment from its `uuid`.
    /// # Arguments
    /// The request containing the `uuid` of the deployment to apply the configuration to, as well as its configuration in JSON format.
    /// # Returns
    /// A message indicating the configuration was applied successfully, wrapped in a Result.
    /// # Errors
    /// If the JSON configuration is invalid, returns an `invalid_argument` error.
    /// If the deployment does not exist, returns a `not_found` error.
    /// If the configuration could not be applied, returns an `internal` error.
    async fn apply_config_deployment(
        &self,
        request: Request<ApplyConfigDeploymentRequest>,
    ) -> Result<Response<ResponseMessage>, Status> {
        let request = request.into_inner();
        let config = request.config;
        let deployment_uuid = request.deployment_uuid;
        let project_uuid = request.project_uuid;
        let user_uuid = request.user_uuid;

        trace!("Transforming config string to hashmap");

        let hashmap_config: HashMap<String, String> = match serde_json::from_str(&config) {
            Ok(config) => {
                trace!("JSON config parsed successfully for {}", deployment_uuid);
                config
            }
            Err(e) => {
                trace!("Failed to parse json config: {}", e);
                return Err(Status::invalid_argument(format!(
                    "Failed to parse json config: {}",
                    e
                )));
            }
        };

        trace!(
            "Creating app {} with config {:?}",
            deployment_uuid,
            hashmap_config
        );

        let app = Application {
            application_id: deployment_uuid.clone(),
            project_id: project_uuid,
            image_name: format!("{}/{}", user_uuid, deployment_uuid),
            image_tag: String::from("latest"),
            env_variables: hashmap_config,
        };

        let message = match self.docker_engine.restart_application(&app).await {
            Ok(_) => {
                trace!("Applied config to {}", deployment_uuid);
                format!("Applied config to application {}", deployment_uuid)
            }
            Err(e) => {
                trace!(
                    "Failed to apply config to application {}: {}",
                    deployment_uuid,
                    e
                );
                return Err(Status::internal(format!(
                    "Failed to apply config to application {}: {}",
                    deployment_uuid, e
                )));
            }
        };

        let response = ResponseMessage { message };
        Ok(Response::new(response))
    }
}

/// # Get App from db with uuid
/// Get an `Application` from the database from its uuid.
/// # Arguments
/// - The borrowed `uuid` of the application to get.
/// # Returns
/// The application, wrapped in a Result.
/// # Errors
/// If the application is not found in the database, returns a `Status::not_found`.
fn get_app(deployment_uuid: &str, project_uuid: &str, user_uuid: &str) -> Application {
    Application {
        application_id: deployment_uuid.to_string(),
        project_id: project_uuid.to_string(),
        image_name: format!("{}/{}", user_uuid, deployment_uuid),
        image_tag: String::from("latest"),
        ..Default::default()
    }
}

/// # Start Server
/// Start the gRPC server.
/// # Arguments
/// - The Docker engine to use to manage containers.
/// - The database reference to use to get the deployments.
/// # Returns
/// Nothing.
pub async fn start_server(docker_engine: DockerEngine) -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;

    let pomegranate_grpc_server = PomegranateGrpcServer { docker_engine };

    info!("gRPC server started on {}", addr);

    Server::builder()
        .add_service(PomegranateServer::new(pomegranate_grpc_server))
        .serve(addr)
        .await?;

    Ok(())
}
