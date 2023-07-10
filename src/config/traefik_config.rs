/// # PaaS Traefik Config
/// Network configuration info
#[derive(Debug, Default)]
pub struct TraefikConfig {
    /// The root url being used by all apps, eg `paastech.cloud`
    pub fqdn: String,
    /// Name of the network in which traefik communicates with spawned containers
    pub network_name: String,
}

impl TraefikConfig {
    /// # From env
    /// Build a new instance of TraefikConfig from env variables
    pub fn from_env() -> Self {
        TraefikConfig {
            fqdn: std::env::var("POMEGRANATE_FQDN").unwrap_or(String::from("localhost")),
            network_name: std::env::var("POMEGRANATE_DOCKER_NETWORK_NAME")
                .unwrap_or(String::from("traefik-fallback-network")),
        }
    }
}
