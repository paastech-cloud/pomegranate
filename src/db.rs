use std::fmt::Error;

pub struct Db {}

impl Db {
    pub fn new() -> Db {
        Db {}
    }

    pub fn get_deployment_join_project(&self, uuid: String) -> DeploymentJoinProject {
        DeploymentJoinProject::new(uuid)
    }

    pub async fn set_deployment_config(&self, _uuid: String, _config: String) -> Result<(), Error> {
        Ok(())
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
    pub fn new(uuid: String) -> DeploymentJoinProject {
        DeploymentJoinProject {
            uuid,
            name: String::from("a name"),
            config: String::from("a config"),
            project_uuid: String::from("a project"),
            user_id: String::from("a user"),
        }
    }
}
