pub struct SessionConfig {
    pub default_ttl: std::time::Duration,
    pub max_concurrent_sessions_per_user: usize,
}

impl Default for SessionConfig {
    fn default() -> Self {
        SessionConfig {
            default_ttl: std::time::Duration::from_secs(3600 * 24 * 3),
            max_concurrent_sessions_per_user: 3,
        }
    }
}
