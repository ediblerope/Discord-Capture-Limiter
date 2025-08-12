use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tokio::process::Command;
use tokio::time::{sleep, Duration};
use tracing::{debug, error, info, warn};

#[derive(Debug, Deserialize, Serialize)]
struct Config {
    #[serde(default = "default_target_name")]
    target_node_name: String,
    #[serde(default = "default_log_level")]
    log_level: String,
    #[serde(default = "default_check_interval")]
    check_interval_ms: u64,
}

fn default_target_name() -> String {
    "discord_capture".to_string()
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_check_interval() -> u64 {
    1000
}

impl Default for Config {
    fn default() -> Self {
        Self {
            target_node_name: default_target_name(),
            log_level: default_log_level(),
            check_interval_ms: default_check_interval(),
        }
    }
}

// We don't need a separate struct since we're using the ID as the HashMap key
// Just store a simple marker that this stream exists
type StreamInfo = ();

struct DiscordCaptureLimiter {
    config: Config,
}

impl DiscordCaptureLimiter {
    fn new(config: Config) -> Self {
        Self { config }
    }

    async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Starting Discord Capture Limiter");
        info!("Target node name: {}", self.config.target_node_name);

        // Check if pw-cli is available
        if let Err(e) = Command::new("pw-cli").arg("--version").output().await {
            error!("pw-cli not found: {}. Make sure PipeWire tools are installed.", e);
            return Err("pw-cli not available".into());
        }

        let mut previous_streams: HashMap<u32, StreamInfo> = HashMap::new();

        // Main monitoring loop
        loop {
            match self.get_current_streams().await {
                Ok(current_streams) => {
                    // Check for new streams
                    let new_streams: Vec<_> = current_streams.keys()
                    .filter(|id| !previous_streams.contains_key(id))
                    .collect();

                    if !new_streams.is_empty() {
                        info!("New discord_capture streams detected: {:?}", new_streams);
                    }

                    // Update mute states if we have multiple streams
                    if current_streams.len() > 1 {
                        if let Err(e) = self.update_mute_states(&current_streams).await {
                            warn!("Failed to update mute states: {}", e);
                        }
                    } else if current_streams.len() == 1 {
                        // Ensure the single stream is unmuted
                        let stream_id = *current_streams.keys().next().unwrap();
                        if let Err(e) = self.set_stream_mute(stream_id, false).await {
                            warn!("Failed to unmute single stream {}: {}", stream_id, e);
                        }
                    }

                    previous_streams = current_streams;
                }
                Err(e) => {
                    warn!("Failed to get current streams: {}", e);
                }
            }

            sleep(Duration::from_millis(self.config.check_interval_ms)).await;
        }
    }

    async fn get_current_streams(&self) -> Result<HashMap<u32, StreamInfo>, Box<dyn std::error::Error>> {
        let output = Command::new("pw-cli")
        .args(["list-objects", "Node"])
        .output()
        .await?;

        if !output.status.success() {
            return Err("pw-cli list-objects failed".into());
        }

        let output_str = String::from_utf8(output.stdout)?;
        let mut streams = HashMap::new();

        // Parse the pw-cli output
        let mut current_id: Option<u32> = None;
        let mut current_name: Option<String> = None;
        let mut current_media_class: Option<String> = None;

        for line in output_str.lines() {
            let line = line.trim();

            // Look for node ID lines: "id 75, type PipeWire:Interface:Node/3"
            if line.starts_with("id ") {
                if let Some(id_str) = line.split(',').next().and_then(|s| s.strip_prefix("id ")) {
                    current_id = id_str.trim().parse().ok();
                    current_name = None;
                    current_media_class = None;
                }
            }
            // Look for node.name
            else if line.contains("node.name = ") {
                if let Some(name) = line.split('=').nth(1) {
                    current_name = Some(name.trim().trim_matches('"').to_string());
                }
            }
            // Look for media.class
            else if line.contains("media.class = ") {
                if let Some(class) = line.split('=').nth(1) {
                    current_media_class = Some(class.trim().trim_matches('"').to_string());
                }
            }

            // Check if we have all the info we need for a discord_capture stream
            if let (Some(id), Some(name), Some(media_class)) = (&current_id, &current_name, &current_media_class) {
                if name == &self.config.target_node_name && media_class == "Stream/Input/Audio" {
                    streams.insert(*id, ());
                    debug!("Found discord_capture stream: id {}", id);
                }
            }
        }

        Ok(streams)
    }

    async fn update_mute_states(&self, streams: &HashMap<u32, StreamInfo>) -> Result<(), Box<dyn std::error::Error>> {
        let stream_ids: Vec<u32> = streams.keys().cloned().collect();

        if stream_ids.len() <= 1 {
            return Ok(());
        }

        let min_id = stream_ids.iter().min().cloned().unwrap_or(0);

        info!("Found {} discord_capture streams, keeping id {} unmuted",
              stream_ids.len(), min_id);

        for &id in &stream_ids {
            let should_mute = id != min_id;

            if let Err(e) = self.set_stream_mute(id, should_mute).await {
                warn!("Failed to set mute state for stream {}: {}", id, e);
            } else {
                debug!("Stream {} mute state: {}", id, should_mute);
            }
        }

        Ok(())
    }

    async fn set_stream_mute(&self, node_id: u32, mute: bool) -> Result<(), Box<dyn std::error::Error>> {
        // Use pw-cli to mute/unmute the stream
        let mute_str = if mute { "true" } else { "false" };
        let output = Command::new("pw-cli")
        .args(["set-param", &node_id.to_string(), "Props", &format!("{{ mute: {} }}", mute_str)])
        .output()
        .await?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(format!("pw-cli command failed: {}", error_msg).into());
        }

        Ok(())
    }
}

fn load_config() -> Config {
    let config_paths = [
        PathBuf::from(std::env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| {
            format!("{}/.config", std::env::var("HOME").unwrap_or_default())
        })).join("discord-capture-limiter/config.toml"),
        PathBuf::from("/etc/discord-capture-limiter/config.toml"),
    ];

    for path in &config_paths {
        if path.exists() {
            info!("Loading config from: {}", path.display());
            match fs::read_to_string(path) {
                Ok(contents) => match toml::from_str(&contents) {
                    Ok(config) => return config,
                    Err(e) => error!("Failed to parse config file {}: {}", path.display(), e),
                },
                Err(e) => error!("Failed to read config file {}: {}", path.display(), e),
            }
        }
    }

    info!("No config file found, using defaults");
    Config::default()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = load_config();

    // Initialize logging
    let log_level = match config.log_level.as_str() {
        "error" => tracing::Level::ERROR,
        "warn" => tracing::Level::WARN,
        "info" => tracing::Level::INFO,
        "debug" => tracing::Level::DEBUG,
        "trace" => tracing::Level::TRACE,
        _ => tracing::Level::INFO,
    };

    tracing_subscriber::fmt()
    .with_max_level(log_level)
    .init();

    let mut limiter = DiscordCaptureLimiter::new(config);
    limiter.run().await
}
