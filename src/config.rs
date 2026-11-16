use std::net::SocketAddr;

#[derive(Clone, Debug)]
pub struct Config {
    pub bind: SocketAddr,
    pub accepted_origin: String,
}
impl Config {
    pub fn from_values(bind: Option<&str>, origin: Option<&str>) -> Result<Self, String> {
        let origin = origin.unwrap_or(crate::security::PUBLIC_ORIGIN);
        if !crate::security::valid_origin(origin) {
            return Err("ACCEPTED_ORIGIN must be a canonical http(s) origin without credentials, path, query, or fragment".into());
        }
        Ok(Self {
            bind: bind
                .unwrap_or("0.0.0.0:8080")
                .parse()
                .map_err(|e| format!("invalid BIND_ADDR: {e}"))?,
            accepted_origin: origin.into(),
        })
    }
}
