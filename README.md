Discord Capture Limiter
A lightweight daemon that automatically manages multiple Discord audio capture streams in PipeWire, ensuring only the oldest stream remains active to prevent audio duplication issues.

Problem Solved
When streaming or recording with Discord, multiple discord_capture streams can be created (e.g., when playing YouTube videos, Spotify, or other media). This can cause:

Audio duplication in recordings
Unwanted background audio being broadcast to viewers
Confusion about which audio stream is actually being captured
Discord Capture Limiter automatically detects when new discord_capture streams are created and mutes all except the one with the lowest ID (typically the first/primary stream).

Features
🎵 Automatic Detection: Monitors PipeWire for new Discord capture streams
🔇 Smart Muting: Keeps only the lowest-ID stream unmuted
⚙️ Configurable: TOML configuration file support
🔒 Secure: Runs as user service with security hardening
📝 Logging: Comprehensive logging with configurable levels
🚀 Lightweight: Minimal resource usage, pure Rust implementation
Installation
NixOS (Recommended)
Add to your NixOS configuration:

nix
{
  services.discord-capture-limiter.enable = true;
  
  # Optional: customize settings
  services.discord-capture-limiter.settings = {
    log_level = "info";
    check_interval_ms = 1000;
  };
}
Manual Installation
bash
# Clone and build
git clone https://github.com/yourusername/discord-capture-limiter
cd discord-capture-limiter
cargo build --release

# Install binary
sudo cp target/release/discord-capture-limiter /usr/local/bin/

# Install and enable user service
mkdir -p ~/.config/systemd/user
cp discord-capture-limiter.service ~/.config/systemd/user/
systemctl --user enable --now discord-capture-limiter
Nix Package Manager
bash
nix-env -iA nixpkgs.discord-capture-limiter
# or with flakes:
nix profile install nixpkgs#discord-capture-limiter
Configuration
Create ~/.config/discord-capture-limiter/config.toml:

toml
# Node name to monitor (usually "discord_capture")
target_node_name = "discord_capture"

# Logging level: error, warn, info, debug, trace
log_level = "info" 

# Check interval in milliseconds
check_interval_ms = 1000
Usage
As a Service (Recommended)
bash
# Start the user service
systemctl --user start discord-capture-limiter

# Enable auto-start
systemctl --user enable discord-capture-limiter

# Check status
systemctl --user status discord-capture-limiter

# View logs
journalctl --user -u discord-capture-limiter -f
Manual Execution
bash
# Run with debug logging
RUST_LOG=debug discord-capture-limiter

# Run with custom config
discord-capture-limiter # Reads from ~/.config/discord-capture-limiter/config.toml
How It Works
Monitors PipeWire: Uses pw-cli to periodically check for Stream/Input/Audio nodes named discord_capture
Tracks Stream IDs: Maintains a list of all active Discord capture streams
Smart Muting: When multiple streams exist, mutes all except the lowest ID
Automatic Cleanup: Removes tracking for streams that no longer exist
Requirements
PipeWire: Audio server with pw-cli tools
Discord: Running with audio capture enabled
Linux: Tested on NixOS, should work on any Linux distribution
Development
Building
bash
# Development shell (NixOS)
nix-shell

# Build
cargo build

# Run tests
cargo test

# Run with debug logging
RUST_LOG=debug cargo run
Testing
Start the limiter: RUST_LOG=debug cargo run
Join a Discord voice channel
Play audio (YouTube, Spotify, etc.)
Observe automatic muting in logs
Contributing
Contributions welcome! Please:

Fork the repository
Create a feature branch
Add tests for new functionality
Submit a pull request
License
Licensed under either of:

Apache License, Version 2.0 (LICENSE-APACHE)
MIT License (LICENSE-MIT)
at your option.

Acknowledgments
Built for the PipeWire audio ecosystem
Inspired by real streaming workflow needs
Thanks to the Rust and Nix communities
