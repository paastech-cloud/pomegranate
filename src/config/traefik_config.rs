use once_cell::sync::Lazy;

#[derive(Debug, Default)]
pub struct TraefikConfig {
    pub fqdn: String,
    pub network_name: String,
}

impl TraefikConfig {
    pub fn new() -> Self {
        TraefikConfig {
            fqdn: std::env::var("POMEGRANATE_FQDN").unwrap_or(String::from("localhost")),
            network_name: std::env::var("POMEGRANATE_DOCKER_NETWORK_NAME")
                .expect("Missing proxy network !"),
        }
    }
}

pub static TRAEFIK_CONFIG: Lazy<TraefikConfig> = Lazy::new(TraefikConfig::new);
