use std::net::{IpAddr, SocketAddr};
use serde::Deserialize;

pub fn get_config<'de, T: Deserialize<'de>>(config_path: &str) -> Result<T, config::ConfigError> {
    let config = config::Config::builder()
        .add_source(config::File::with_name(config_path))
        .build()
        .unwrap();
    config.try_deserialize::<T>()
}

#[derive(Debug, Deserialize)]
pub struct GoalConfiguration {
    pub server: ServerConfiguration,
    pub dns: DnsConfiguration,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfiguration {
    ip_address: IpAddr,
    port: u16,
}

impl ServerConfiguration {
    pub fn bind_address(&self) -> SocketAddr {
        SocketAddr::new(self.ip_address, self.port)
    }
}

#[derive(Debug, Deserialize)]
pub struct DnsConfiguration {
    pub server_address: SocketAddr,
}
