use std::time::Duration;

#[derive(Debug, Clone)]
pub struct Config {
    pub discord_app_id: &'static str,
    pub poll_interval: Duration,
    pub log_level: log::LevelFilter,
    pub max_log_size: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            discord_app_id: "996864734957670452",
            poll_interval: Duration::from_secs(1),
            log_level: log::LevelFilter::Info,
            max_log_size: 20 * 1024 * 1024, // 20MB
        }
    }
}
