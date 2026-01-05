pub struct Config {
    pub bind_addr: String,
    pub database_url: String,
    pub db_max_connections: u32,
}

impl Config {
    pub fn from_env() -> Self {
        let bind_addr = std::env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string());
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL is required");
        let db_max_connections = std::env::var("DB_MAX_CONNECTIONS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(5);

        Self {
            bind_addr,
            database_url,
            db_max_connections,
        }
    }
}
