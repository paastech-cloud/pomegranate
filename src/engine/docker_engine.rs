use bollard::Docker;
use log::{info, trace};

use super::engine::Engine;
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
}

impl Engine for DockerEngine {
    fn start_application(&self, app: &Application) {
        // TODO: Implement me
        trace!("Starting app: {:?}", app);
    }
}
