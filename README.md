# Discord Capture Limiter

A lightweight daemon that automatically manages multiple Discord audio capture streams in PipeWire, ensuring only the oldest stream remains active to prevent additional capture streams being played through your video game streams. Built for NixOS, may work on other distros. The whole thing was made by Claude AI so I'm sure the code can be improved.

## Problem Solved

When streaming or recording with Discord, multiple `discord_capture` streams can be created (e.g., when playing YouTube videos, Spotify, or other media). This causes:
- Unwanted background audio being broadcast to viewers
- Confusion about which audio stream is actually being captured, as they are using the same name and have no distinguishing features

Discord Capture Limiter automatically detects when new `discord_capture` streams are created and mutes all except the one with the lowest ID (should be the first/primary stream, more testing might reveal something different).

## Features

- 🎵 **Automatic Detection**: Monitors PipeWire for new Discord capture streams
- 🔇 **Smart Muting**: Keeps only the lowest-ID stream unmuted
- ⚙️ **Configurable**: TOML configuration file support
- 🔒 **Secure**: Runs as user service with security hardening
- 📝 **Logging**: Comprehensive logging with configurable levels
- 🚀 **Lightweight**: Minimal resource usage, pure Rust implementation

## Installation

### NixOS (Recommended)

Add to your NixOS configuration:

```nix
{
  services.discord-capture-limiter.enable = true;
  
  # Optional: customize settings
  services.discord-capture-limiter.settings = {
    log_level = "info";
    check_interval_ms = 1000;
  };
}
```

### Manual Installation

```bash
# Clone and build
git clone https://github.com/ediblerope/Discord-Capture-Limiter
cd discord-capture-limiter
cargo build --release

# Install binary
sudo cp target/release/discord-capture-limiter /usr/local/bin/

# Install and enable user service
mkdir -p ~/.config/systemd/user
cp discord-capture-limiter.service ~/.config/systemd/user/
systemctl --user enable --now discord-capture-limiter
```

### Nix Package Manager

```bash
nix-env -iA nixpkgs.discord-capture-limiter
# or with flakes:
nix profile install nixpkgs#discord-capture-limiter
```

## Configuration

Create `~/.config/discord-capture-limiter/config.toml`:

```toml
# Node name to monitor (usually "discord_capture")
target_node_name = "discord_capture"

# Logging level: error, warn, info, debug, trace
log_level = "info" 

# Check interval in milliseconds
check_interval_ms = 1000
```

## Usage

### As a Service (Recommended)

```bash
# Start the user service
systemctl --user start discord-capture-limiter

# Enable auto-start
systemctl --user enable discord-capture-limiter

# Check status
systemctl --user status discord-capture-limiter

# View logs
journalctl --user -u discord-capture-limiter -f
```

### Manual Execution

```bash
# Run with debug logging
RUST_LOG=debug discord-capture-limiter

# Run with custom config
discord-capture-limiter # Reads from ~/.config/discord-capture-limiter/config.toml
```

## How It Works

1. **Monitors PipeWire**: Uses `pw-cli` to periodically check for `Stream/Input/Audio` nodes named `discord_capture`
2. **Tracks Stream IDs**: Maintains a list of all active Discord capture streams
3. **Smart Muting**: When multiple streams exist, mutes all except the lowest ID
4. **Automatic Cleanup**: Removes tracking for streams that no longer exist

## Requirements

- **PipeWire**: Audio server with `pw-cli` tools
- **Discord**: Running with audio capture enabled
- **Linux**: Tested on NixOS, might work on any Linux distribution (I have no clue)

## Development

### Building

```bash
# Development shell (NixOS)
nix-shell

# Build
cargo build

# Run tests
cargo test

# Run with debug logging
RUST_LOG=debug cargo run
```

### Testing

1. Start the limiter: `RUST_LOG=debug cargo run`
2. Join a Discord voice channel
3. Start a stream of a game
4. Play audio (YouTube, Spotify, etc.)
5. Observe automatic muting in logs

## Contributing

Contributions welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality  
4. Submit a pull request

## License

Licensed under:
- MIT License ([LICENSE-MIT](LICENSE-MIT))

## Acknowledgments

- Built for the PipeWire audio ecosystem
- Inspired by an issue that is occurring on my NixOS installation, no idea how widespread this issue is. I had it and this fixes it.
- Thanks to the Claude AI lol
