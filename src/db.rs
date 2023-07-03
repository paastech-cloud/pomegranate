use crate::application::Application;
use chrono::Utc;
use std::collections::HashMap;
use std::fmt::Error;

pub struct Db {}

impl Db {
    pub fn new() -> Db {
        Db {}
    }

    pub fn get_deployment_and_project(&self, uuid: &str) -> Result<DeploymentAndProject, Error> {
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
    pub fn get_app(&self, uuid: &str) -> Result<Application, Error> {
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
        uuid: &str,
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

/// # Deployment
/// A Deployment is a struct that contains the information about a deployment as per the database schema.
/// # Fields
/// - `uuid`: The uuid of the deployment.
/// - `name`: The name of the deployment.
/// - `config`: The configuration of the deployment.
/// - `created_at`: The date at which the deployment was created.
/// - `updated_at`: The date at which the deployment was last updated.
/// - `project_uuid`: The uuid of the project that owns the deployment.
pub struct Deployment {
    pub uuid: String,
    pub name: String,
    pub config: String,
    pub created_at: String,
    pub updated_at: String,
    pub project_uuid: String,
}

/// # Project
/// A Project is a struct that contains the information about a project as per the database schema.
/// # Fields
/// - `uuid`: The uuid of the project.
/// - `name`: The name of the project.
/// - `created_at`: The date at which the project was created.
/// - `updated_at`: The date at which the project was last updated.
/// - `user_id`: The uuid of the user that owns the project.
pub struct Project {
    pub uuid: String,
    pub name: String,
    pub created_at: String,
    pub updated_at: String,
    pub user_id: String,
}

/// # DeploymentAndProject
/// A DeploymentAndProject is a struct that contains a Deployment and a Project.
/// # Fields
/// - `project`: The project.
/// - `deployment`: The deployment.
pub struct DeploymentAndProject {
    pub project: Project,
    pub deployment: Deployment,
}

impl DeploymentAndProject {

    /// # /!\ WARNING
    /// As of now, the `uuid` is hardcoded to `nginx`. This function is only present for test purposes and will be deleted as soon as the database is implemented.
    /// # New
    /// Create a new DeploymentAndProject from a `uuid`.
    /// # Arguments
    /// - The `uuid` of the DeploymentAndProject to create.
    /// # Returns
    /// A DeploymentAndProject, wrapped in a result.
    /// # Errors
    /// If the `uuid` is not found, return an error.
    pub fn new(uuid: &str) -> Result<DeploymentAndProject, Error> {
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
