use log::info;
use tonic::{transport::Server, Request, Response, Status};

use crate::application::Application;
use crate::db::Db;
use crate::engine::docker_engine::DockerEngine;
use crate::engine::Engine;
use paastech_proto::pomegranate::pomegranate_server::{Pomegranate, PomegranateServer};
use paastech_proto::pomegranate::{
    ApplyConfigDeploymentRequest, DeleteDeploymentRequest, DeploymentStatusRequest,
    ResponseMessage, RestartDeploymentRequest, StartDeploymentRequest, StopDeploymentRequest,
};

/// # Pomegranate gRPC server
/// The gRPC server that implements the Pomegranate routes.
pub struct PomegranateGrpcServer {
    docker_engine: DockerEngine,
    db: Db,
}

#[tonic::async_trait]
impl Pomegranate for PomegranateGrpcServer {
    /// # Start Deployment
    /// Start a deployment from its uuid.
    /// # Arguments
    /// The request containing the uuid of the deployment to start.
    /// # Returns
    /// Nothing, wrapped in a Result.
    async fn start_deployment(
        &self,
        request: Request<StartDeploymentRequest>,
    ) -> Result<Response<ResponseMessage>, Status> {
        let deployment_uuid = request.into_inner().deployment_uuid;

        let app = get_app(&self.db, deployment_uuid);

        let message: String = match self.docker_engine.start_application(&app).await {
            Ok(_) => {
                format!("Started application {}", app.project_id)
            }
            Err(e) => {
                format!("Failed to start application {}: {}", app.project_id, e)
            }
        };

        let response = ResponseMessage { message };
        Ok(Response::new(response))
    }

    async fn restart_deployment(
        &self,
        request: Request<RestartDeploymentRequest>,
    ) -> Result<Response<ResponseMessage>, Status> {
        let deployment_uuid = request.into_inner().deployment_uuid;

        let app = get_app(&self.db, deployment_uuid);

        let message: String = match self.docker_engine.restart_application(&app).await {
            Ok(_) => {
                format!("Restarted application {}", app.project_id)
            }
            Err(e) => {
                format!("Failed to restart application {}: {}", app.project_id, e)
            }
        };

        let response = ResponseMessage { message };
        Ok(Response::new(response))
    }

    /// # Delete Deployment
    /// Delete a configuration to a deployment from its uuid.
    /// # Arguments
    /// The request containing the uuid of the deployment to delete.
    /// # Returns
    /// Nothing, wrapped in a Result.
    async fn delete_deployment(
        &self,
        request: Request<DeleteDeploymentRequest>,
    ) -> Result<Response<ResponseMessage>, Status> {
        let deployment_uuid = request.into_inner().deployment_uuid;

        let app = get_app(&self.db, deployment_uuid);

        let message: String = match self
            .docker_engine
            .stop_application(app.project_id.as_str(), app.application_id.as_str())
            .await
        {
            Ok(_) => {
                //TODO: change message when deletion is implemented
                format!("Stopped application {}. It was not deleted", app.project_id)
            }
            Err(e) => {
                format!("Failed to delete application {}: {}", app.project_id, e)
            }
        };

        //TODO: Delete the app from the database & prune its image

        let response = ResponseMessage { message };
        Ok(Response::new(response))
    }

    /// # Stop Deployment
    /// Stop a deployment from its uuid.
    /// # Arguments
    /// The request containing the uuid of the deployment to stop.
    /// # Returns
    /// Nothing, wrapped in a Result.
    async fn stop_deployment(
        &self,
        request: Request<StopDeploymentRequest>,
    ) -> Result<Response<ResponseMessage>, Status> {
        let deployment_uuid = request.into_inner().deployment_uuid;

        let app = get_app(&self.db, deployment_uuid);

        let message: String = match self
            .docker_engine
            .stop_application(app.project_id.as_str(), app.application_id.as_str())
            .await
        {
            Ok(_) => {
                format!("Stopped application {}", app.project_id)
            }
            Err(e) => {
                format!("Failed to stop application {}: {}", app.project_id, e)
            }
        };

        let response = ResponseMessage { message };
        Ok(Response::new(response))
    }

    /// # Deployment Status
    /// Get the status of a deployment from its uuid.
    /// # Arguments
    /// The request containing the uuid of the deployment to get the status of.
    /// # Returns
    /// Nothing, wrapped in a Result.
    async fn deployment_status(
        &self,
        request: Request<DeploymentStatusRequest>,
    ) -> Result<Response<ResponseMessage>, Status> {
        let deployment_uuid = request.into_inner().deployment_uuid;

        let app = get_app(&self.db, deployment_uuid);

        let status = self
            .docker_engine
            .get_application_status(app.project_id.as_str(), app.application_id.as_str())
            .await
            .unwrap();

        let message = format!("Application {} is {:?}", app.project_id, status);

        let response = ResponseMessage { message };
        Ok(Response::new(response))
    }

    /// # Apply Config Deployment
    /// Apply a configuration to a deployment from its uuid.
    /// # Arguments
    /// The request containing the uuid of the deployment to apply the configuration to, as well as its configuration in JSON format.
    /// # Returns
    /// Nothing, wrapped in a Result.
    async fn apply_config_deployment(
        &self,
        request: Request<ApplyConfigDeploymentRequest>,
    ) -> Result<Response<ResponseMessage>, Status> {
        let request = request.into_inner();
        let config = request.config;
        let deployment_uuid = request.deployment_uuid;

        let deployment_join_project = &self.db.get_deployment_join_project(deployment_uuid.clone());

        let app = Application {
            application_id: deployment_uuid.clone(),
            project_id: deployment_join_project.project_uuid.clone(),
            image_name: format!("{}/{}", deployment_join_project.user_id, deployment_uuid),
            image_tag: String::from("latest"),
            env_variables: serde_json::from_str(&config).unwrap(),
        };

        let message: String = match self.docker_engine.restart_application(&app).await {
            Ok(_) => match self.db.set_deployment_config(deployment_uuid, config).await {
                Ok(_) => {
                    format!("Applied config to application {}", app.project_id)
                }
                Err(e) => {
                    format!(
                        "Failed to save config to database {}: {}",
                        app.project_id, e
                    )
                }
            },
            Err(e) => {
                format!(
                    "Failed to apply config to application {}: {}",
                    app.project_id, e
                )
            }
        };

        let response = ResponseMessage { message };
        Ok(Response::new(response))
    }
}

/// # Get App
/// Get an application from its uuid.
/// # Arguments
/// - The database reference to use to get the deployments.
/// - The uuid of the application to get.
/// # Returns
/// The application.
fn get_app(db: &Db, uuid: String) -> Application {
    let deployment_join_project = db.get_deployment_join_project(uuid.clone());

    Application {
        application_id: uuid.clone(),
        project_id: deployment_join_project.project_uuid,
        image_name: format!("{}/{}", deployment_join_project.user_id, uuid),
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
/// Nothing, wrapped in a Result.
pub async fn start_server(
    docker_engine: DockerEngine,
    db: Db,
) -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse().unwrap();

    let pomegranate_grpc_server = PomegranateGrpcServer { docker_engine, db };

    info!("gRPC server started on {}", addr);

    Server::builder()
        .add_service(PomegranateServer::new(pomegranate_grpc_server))
        .serve(addr)
        .await?;

    Ok(())
}
