use log::info;
use tonic::{transport::Server, Request, Response, Status};

use paastech_proto::pomegranate_proto::pomegranate_server::{Pomegranate, PomegranateServer};
use paastech_proto::pomegranate_proto::{
    ApplyConfigDeploymentRequest, DeleteDeploymentRequest, DeploymentStatusRequest,
    ResponseMessage, RestartDeploymentRequest, StartDeploymentRequest, StopDeploymentRequest,
};

#[derive(Debug, Default)]
pub struct PomegranateGrpcService {}

#[tonic::async_trait]
impl Pomegranate for PomegranateGrpcService {
    async fn start_deployment(
        &self,
        request: Request<StartDeploymentRequest>,
    ) -> Result<Response<ResponseMessage>, Status> {
        let project_uuid = request.into_inner().project_uuid;
        // TODO: Implement the logic for starting deployment

        let response = ResponseMessage {
            // Fill in the fields of the response message
            // This one is a template for test purposes
            message: format!("Start deployment ! UUID : {}!", project_uuid).into(),
        };
        Ok(Response::new(response))
    }

    async fn restart_deployment(
        &self,
        request: Request<RestartDeploymentRequest>,
    ) -> Result<Response<ResponseMessage>, Status> {
        let project_uuid = request.into_inner().project_uuid;
        // TODO: Implement the logic for restarting deployment

        let response = ResponseMessage {
            // Fill in the fields of the response message
            // This one is a template for test purposes
            message: format!("Restart Deployment ! UUID : {}!", project_uuid).into(),
        };
        Ok(Response::new(response))
    }

    async fn delete_deployment(
        &self,
        request: Request<DeleteDeploymentRequest>,
    ) -> Result<Response<ResponseMessage>, Status> {
        let project_uuid = request.into_inner().project_uuid;
        // TODO: Implement the logic for deleting deployment

        let response = ResponseMessage {
            // Fill in the fields of the response message
            // This one is a template for test purposes
            message: format!("Delete Deployment ! UUID : {}!", project_uuid).into(),
        };
        Ok(Response::new(response))
    }

    async fn stop_deployment(
        &self,
        request: Request<StopDeploymentRequest>,
    ) -> Result<Response<ResponseMessage>, Status> {
        let project_uuid = request.into_inner().project_uuid;
        // TODO: Implement the logic for stopping deployment

        let response = ResponseMessage {
            // Fill in the fields of the response message
            // This one is a template for test purposes
            message: format!("Stop Deployment ! UUID : {}!", project_uuid).into(),
        };
        Ok(Response::new(response))
    }

    async fn deployment_status(
        &self,
        request: Request<DeploymentStatusRequest>,
    ) -> Result<Response<ResponseMessage>, Status> {
        let project_uuid = request.into_inner().project_uuid;
        // TODO: Implement the logic for stopping deployment

        let response = ResponseMessage {
            // Fill in the fields of the response message
            // This one is a template for test purposes
            message: format!("Deployment status ! UUID : {}!", project_uuid).into(),
        };
        Ok(Response::new(response))
    }

    async fn apply_config_deployment(
        &self,
        request: Request<ApplyConfigDeploymentRequest>,
    ) -> Result<Response<ResponseMessage>, Status> {
        let config = request.into_inner().config;
        // TODO: Implement the logic for applying configuration to the deployment

        let response = ResponseMessage {
            // Fill in the fields of the response message
            // This one is a template for test purposes
            message: format!("Apply Deployment Config ! Config : {}!", config).into(),
        };
        Ok(Response::new(response))
    }
}

pub(crate) async fn start_server() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse().unwrap();

    let pomegranate_grpc_service = PomegranateGrpcService::default();

    info!("gRPC server started on {}", addr);

    Server::builder()
        .add_service(PomegranateServer::new(pomegranate_grpc_service))
        .serve(addr)
        .await?;

    Ok(())
}
