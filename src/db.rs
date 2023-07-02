use crate::application::Application;
use chrono::{Utc};
use std::collections::HashMap;
use std::fmt::Error;

pub struct Db {}

impl Db {
    pub fn new() -> Db {
        Db {}
    }

    pub fn get_deployment_and_project(&self, uuid: String) -> Result<DeploymentAndProject, Error> {
        DeploymentAndProject::new(uuid)
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
        match self.get_deployment_and_project(uuid) {
            Ok(app) => Ok(Application {
                application_id: app.deployment.uuid.clone(),
                project_id: app.project.uuid,
                image_name: format!("{}/{}", app.project.user_id, app.deployment.uuid),
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
        match self.get_deployment_and_project(uuid) {
            Ok(app) => Ok(Application {
                application_id: app.deployment.uuid.clone(),
                project_id: app.project.uuid,
                image_name: format!("{}/{}", app.project.user_id, app.deployment.uuid),
                image_tag: String::from("latest"),
                env_variables: hashmap_config,
            }),
            Err(e) => Err(e),
        }
    }
}

pub struct Deployment {
    pub uuid: String,
    pub name: String,
    pub config: String,
    pub created_at: String,
    pub updated_at: String,
    pub project_uuid: String,
}

pub struct Project {
    pub uuid: String,
    pub name: String,
    pub created_at: String,
    pub updated_at: String,
    pub user_id: String,
}

pub struct DeploymentAndProject {
    pub project: Project,
    pub deployment: Deployment,
}

impl DeploymentAndProject {
    pub fn new(uuid: String) -> Result<DeploymentAndProject, Error> {
        if uuid == "nginx" {
            Ok(DeploymentAndProject {
                project: Project {
                    uuid: String::from("uuid_project"),
                    name: String::from("name_project"),
                    created_at: Utc::now().to_string(),
                    updated_at: Utc::now().to_string(),
                    user_id: String::from("user_project"),
                },
                deployment: Deployment {
                    uuid: String::from("nginx"),
                    name: String::from("nginx"),
                    config: String::from(""),
                    created_at: Utc::now().to_string(),
                    updated_at: Utc::now().to_string(),
                    project_uuid: String::from("uuid_project"),
                },
            })
        } else {
            Err(Error)
        }
    }
}
