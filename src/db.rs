use crate::application::Application;
use std::collections::HashMap;
use std::fmt::Error;

pub struct Db {}

impl Db {
    pub fn new() -> Db {
        Db {}
    }

    pub fn get_deployment_join_project(
        &self,
        uuid: String,
    ) -> Result<DeploymentJoinProject, Error> {
        DeploymentJoinProject::new(uuid)
    }

    pub async fn set_deployment_config(&self, _uuid: String, _config: String) -> Result<(), Error> {
        Ok(())
    }

    /// # Get App
    /// Get an application from its uuid.
    /// # Arguments
    /// - The uuid of the application to get.
    /// # Returns
    /// Nothing, wrapped in a result.
    pub fn get_app(&self, uuid: String) -> Result<Application, Error> {
        match self.get_deployment_join_project(uuid.clone()) {
            Ok(deployment) => Ok(Application {
                application_id: uuid.clone(),
                project_id: deployment.project_uuid,
                image_name: format!("{}/{}", deployment.user_id, uuid),
                image_tag: String::from("latest"),
                ..Default::default()
            }),
            Err(e) => Err(e),
        }
    }

    /// # Get Custom App
    /// Get an application from its uuid and a given configuration.
    /// # Arguments
    /// - The uuid of the application to get.
    /// - The configuration to apply to the application.
    /// # Returns
    /// Nothing, wrapped in a result.
    pub fn get_custom_app(
        &self,
        uuid: String,
        hashmap_config: HashMap<String, String>,
    ) -> Result<Application, Error> {
        match self.get_deployment_join_project(uuid.clone()) {
            Ok(app) => Ok(Application {
                application_id: uuid.clone(),
                project_id: app.project_uuid.clone(),
                image_name: format!("{}/{}", app.user_id, uuid),
                image_tag: String::from("latest"),
                env_variables: hashmap_config,
            }),
            Err(e) => Err(e),
        }
    }
}

pub struct DeploymentJoinProject {
    pub uuid: String,
    pub name: String,
    pub config: String,
    pub project_uuid: String,
    pub user_id: String,
}

impl DeploymentJoinProject {
    pub fn new(uuid: String) -> Result<DeploymentJoinProject, Error> {
        if uuid == "nginx" {
            Ok(DeploymentJoinProject {
                uuid,
                name: String::from("nginx"),
                config: String::from(""),
                project_uuid: String::from("test"),
                user_id: String::from("test"),
            })
        } else {
            Err(Error)
        }
    }
}
