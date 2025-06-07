<p align="center">
  <img src="https://raw.githubusercontent.com/byytelope/pipeboom/refs/heads/main/assets/logo.png" alt="PipeBoom Logo" width="150">
</p>
<h1 align="center">PipeBoom 🎵|💥</h1>
<p align="center">Automatically sync your Apple Music listening activity to Discord's Rich Presence, showing your friends what you're currently jamming to.</p>
<br/>

<!--toc:start-->

- [Features](#features)
- [Requirements](#requirements)
- [Quick Start](#quick-start)
  - [Launch Agent Setup (Recommended)](#launch-agent-setup-recommended)
  - [Binary Setup](#binary-setup)
  - [Uninstalling](#uninstalling)
- [Commands and Options](#commands-and-options)
  - [`setup`](#setup)
  - [`uninstall`](#uninstall)
  - [`service`](#service)
- [How It Works](#how-it-works)
- [Troubleshooting](#troubleshooting)
  - [Checking Logs](#checking-logs)
  - [Common Issues](#common-issues)
  - [Manual Service Management](#manual-service-management)
- [Development](#development)
  - [Building](#building)
  - [Dependencies](#dependencies)
- [Contributing](#contributing)
- [Support](#support)
- [Privacy](#privacy)
- [License](#license)

<!--toc:end-->

## Features

- 🎵 Real-time Apple Music integration
- 🤖 Discord Rich Presence updates
- 🚀 Automatic startup with macOS Launch Agent
- 📝 Built-in logging for troubleshooting
- 🔄 Auto-restart on crashes

## Requirements

- **macOS >= 10.15**
- **Discord desktop app** (latest version recommended)
- **Apple Music**
- **Rust >= 1.85** (for building from source)

## Quick Start

### Launch Agent Setup (Recommended)

1. Clone the repository:
   ```bash
   git clone https://github.com/byytelope/pipeboom.git
   cd pipeboom
   ```

2. Build and install as a Launch Agent:
   ```bash
   cargo build --release
   cp target/release/pipeboom ~/.local/bin/
   pipeboom setup
   ```

That's it! The app will now start automatically when you log in and keep your
Discord status synced with Apple Music.

### Binary Setup

If you prefer to setup manually:

1. Build the project:
   ```bash
   cargo build --release
   ```

2. Copy the binary to your preferred location:
   ```bash
   cp target/release/pipeboom ~/.local/bin
   ```

3. Run the binary:
   ```bash
   pipeboom
   ```

### Uninstalling

All traces of PipeBoom can be removed in 2 easy steps:

1. Run the uninstall command:
   ```bash
   pipeboom uninstall
   ```

2. Delete the binary:
   ```bash
   rm $(which pipeboom)
   ```

   > This command requires the `pipeboom` binary to be located in a directory in
   > path. Please remove it manually if it is not.

## Commands and Options

### `setup`

Set up the Launch Agent (install and start)

```bash
pipeboom setup
```

### `uninstall`

Uninstall the Launch Agent

```bash
pipeboom uninstall
```

### `service`

Control the PipeBoom service using IPC

```bash
pipeboom service [OPTION]
```

| Option         | Description                 |
| -------------- | --------------------------- |
| `start`        | Start PipeBoom service      |
| `stop`         | Stop PipeBoom service       |
| `current-song` | Get current song details    |
| `status`       | Get current PipeBoom Status |
| `shutdown`     | Kill PipeBoom daemon        |

You can also override the default options:

```bash
pipeboom --poll-interval 2 --log-level debug --max-log-size 5 --socket-path ~/.local/sockets
```

For more information:

```bash
pipeboom help
```

## How It Works

1. The app polls Apple Music for the currently playing track using Osascript
2. When a song changes, it updates Discord's rich presence through IPC
3. Your Discord status shows the current song, artist, and album

## Troubleshooting

### Checking Logs

If the app isn't working as expected, check the logs:

1. View stdout logs

   ```bash
   tail -f ~/Library/Logs/pipeboom.log
   ```

2. View error logs

   ```bash
   tail -f ~/Library/Logs/pipeboom.err
   ```

### Common Issues

**Discord not showing status:**

- Make sure Discord desktop app is running
- Check that "Share your detected activities with others" is enabled in Discord
  `Activity Settings > Activity Privacy`
- Restart Discord after installing

**Apple Music not detected:**

- Ensure Apple Music app is running and playing music
- Make sure you have SharePlay disabled as this may affect music detection
- Check that PipeBoom has necessary permissions

**Launch Agent not starting:**

- Check the service status:

  ```bash
  pipeboom status
  ```

- Reload the service:

  ```bash
  launchctl unload ~/Library/LaunchAgents/me.shadhaan.pipeboom.plist && launchctl load ~/Library/LaunchAgents/me.shadhaan.pipeboom.plist
  ```

### Manual Service Management

- Load the Launch Agent

  ```bash
  launchctl load ~/Library/LaunchAgents/me.shadhaan.pipeboom.plist
  ```

- Unload the Launch Agent

  ```bash
  launchctl unload ~/Library/LaunchAgents/me.shadhaan.pipeboom.plist
  ```

- Check if PipeBoom daemon is running

  ```bash
  launchctl list | grep pipeboom
  ```

## Development

### Building

- Debug build

  ```bash
  cargo build
  ```

- Release build

  ```bash
  cargo build --release
  ```

- Run locally

  ```bash
  cargo run
  ```

### Dependencies

This project uses:

- Apple Osascript CLI for Apple Music integration
- [discord-rich-presence](https://github.com/vionya/discord-rich-presence)
  library for Discord IPC integration

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/ts`)
3. Commit your changes (`git commit -m 'Add ts feature'`)
4. Push to the branch (`git push origin feature/ts`)
5. Open a Pull Request

## Support

If you encounter issues, please
[open an issue](https://github.com/byytelope/pipeboom/issues/new/choose).

## Privacy

PipeBoom does **not** collect or transmit any personal data, and it never will.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file
for details.

---

**Note:** This app requires macOS due to its dependence on Apple Music's
scripting interface. Windows support is planned.
