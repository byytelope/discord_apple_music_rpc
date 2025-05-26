<p align="center">
  <img src="https://raw.githubusercontent.com/byytelope/pipeboom/refs/heads/main/assets/logo.png" alt="PipeBoom Logo" width="300">
</p>

<h1 align="center">PipeBoom 🍎|🔥</h1>

Automatically sync your Apple Music listening activity to Discord's Rich Presence, showing your friends what you're currently jamming to.

## Features

- 🎵 Real-time Apple Music integration
- 🤖 Discord Rich Presence updates
- 🚀 Automatic startup with macOS Launch Agent
- 📝 Built-in logging for troubleshooting
- 🔄 Auto-restart on crashes

## Requirements

- macOS (required for Apple Music integration)
- Discord desktop app
- Apple Music app
- Rust (for building from source)

## Quick Start

### Automated Installation

1. Clone the repository:
   ```bash
   git clone https://github.com/byytelope/pipeboom.git
   cd pipeboom
   ```

2. Run the setup script:
   ```bash
   chmod +x setup.sh
   ./setup.sh
   ```

That's it! The app will now start automatically when you log in and keep your Discord status synced with Apple Music.

### Manual Installation

If you prefer to install manually:

1. Build the project:
   ```bash
   cargo build --release
   ```

2. Copy the binary to your preferred location:
   ```bash
   cp target/release/pipeboom /usr/local/bin/
   ```

3. Run the binary:
   ```bash
   pipeboom
   ```

## Setup Script Usage

The included `setup.sh` script provides several commands:

```bash
./setup.sh install    # Build and install as Launch Agent (default)
./setup.sh uninstall  # Remove the Launch Agent and binary
./setup.sh status     # Check if the service is running
./setup.sh build      # Only build the binary
./setup.sh help       # Show help information
```

## How It Works

1. The app monitors Apple Music for currently playing tracks
2. When a song changes, it updates Discord's Rich Presence
3. Your Discord status shows the current song, artist, and album
4. Friends can see what you're listening to in real-time

## Troubleshooting

### Checking Logs

If the app isn't working as expected, check the logs:

```bash
# View stdout logs
tail -f ~/Library/Logs/pipeboom.log

# View error logs
tail -f ~/Library/Logs/pipeboom.err
```

### Common Issues

**Discord not showing status:**
- Make sure Discord desktop app is running
- Check that "Display currently running game as status message" is enabled in Discord settings
- Restart Discord after installing

**Apple Music not detected:**
- Ensure Apple Music app is running and playing music
- Check that the app has necessary permissions

**Launch Agent not starting:**
- Check the service status: `./setup.sh status`
- Reload the service: `launchctl unload ~/Library/LaunchAgents/me.shadhaan.pipeboom.plist && launchctl load ~/Library/LaunchAgents/me.shadhaan.pipeboom.plist`

### Manual Service Management

```bash
# Load the Launch Agent
launchctl load ~/Library/LaunchAgents/me.shadhaan.pipeboom.plist

# Unload the Launch Agent
launchctl unload ~/Library/LaunchAgents/me.shadhaan.pipeboom.plist

# Check if running
launchctl list | grep pipeboom
```

## Development

### Building

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Run locally
cargo run
```

### Dependencies

This project uses:
- Rust standard library for core functionality
- macOS system APIs for Apple Music integration
- Discord RPC libraries for status updates

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/ts`)
3. Commit your changes (`git commit -m 'Add ts'`)
4. Push to the branch (`git push origin feature/ts`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

**Note:** This app requires macOS due to its dependence on Apple Music's scripting interface. Windows support is planned.
