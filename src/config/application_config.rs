use super::traefik_config::TraefikConfig;

/// Struct encapsulating every configuration needed by the app
#[derive(Debug, Default)]
pub struct ApplicationConfig {
    /// Network configuration, see [TraefikConfig](TraefikConfig) struct
    pub traefik_config: TraefikConfig,
}

impl ApplicationConfig {
    pub fn from_env() -> Self {
        ApplicationConfig {
            traefik_config: TraefikConfig::from_env(),
        }
    }
}
