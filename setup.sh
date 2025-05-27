#!/bin/bash

set -e

BINARY_NAME="pipeboom"
SERVICE_NAME="me.shadhaan.${BINARY_NAME}"
INSTALL_DIR="$HOME/bin"
PLIST_DIR="$HOME/Library/LaunchAgents"
PLIST_FILE="${PLIST_DIR}/${SERVICE_NAME}.plist"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log_info() {
  echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
  echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
  echo -e "${RED}[ERROR]${NC} $1"
}

check_rust_project() {
  if [ ! -f "Cargo.toml" ]; then
    log_error "No Cargo.toml found. Please run this script from your Rust project root."
    exit 1
  fi
}

build_binary() {
  log_info "Building Rust binary in release mode..."
  cargo build --release

  if [ ! -f "target/release/${BINARY_NAME}" ]; then
    log_error "Binary 'target/release/${BINARY_NAME}' not found after build."
    log_error "Make sure BINARY_NAME matches your binary name in Cargo.toml"
    exit 1
  fi

  log_info "Build completed successfully"
}

install_binary() {
  log_info "Installing binary to ${INSTALL_DIR}..."

  mkdir -p "${INSTALL_DIR}"

  cp "target/release/${BINARY_NAME}" "${INSTALL_DIR}/"
  chmod +x "${INSTALL_DIR}/${BINARY_NAME}"

  log_info "Binary installed to ${INSTALL_DIR}/${BINARY_NAME}"
}

create_plist() {
  log_info "Creating Launch Agent plist file..."

  mkdir -p "${PLIST_DIR}"

  cat >"${PLIST_FILE}" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>${SERVICE_NAME}</string>
    
    <key>ProgramArguments</key>
    <array>
        <string>${INSTALL_DIR}/${BINARY_NAME}</string>
    </array>
    
    <key>RunAtLoad</key>
    <true/>
    
    <key>KeepAlive</key>
    <true/>
    
    <key>StandardOutPath</key>
    <string>${HOME}/Library/Logs/${BINARY_NAME}.log</string>
    
    <key>StandardErrorPath</key>
    <string>${HOME}/Library/Logs/${BINARY_NAME}.err</string>
    
    <key>WorkingDirectory</key>
    <string>${HOME}</string>
</dict>
</plist>
EOF

  log_info "Plist file created at ${PLIST_FILE}"
}

load_launch_agent() {
  log_info "Loading Launch Agent..."

  launchctl unload "${PLIST_FILE}" 2>/dev/null || true

  if launchctl load "${PLIST_FILE}"; then
    log_info "Launch Agent loaded successfully"
  else
    log_error "Failed to load Launch Agent"
    exit 1
  fi
}

uninstall() {
  log_info "Uninstalling Launch Agent..."

  if launchctl unload "${PLIST_FILE}" 2>/dev/null; then
    log_info "Launch Agent unloaded"
  fi

  if [ -f "${PLIST_FILE}" ]; then
    rm "${PLIST_FILE}"
    log_info "Plist file removed"
  fi

  if [ -f "${INSTALL_DIR}/${BINARY_NAME}" ]; then
    rm "${INSTALL_DIR}/${BINARY_NAME}"
    log_info "Binary removed"
  fi

  log_info "Uninstall completed"
}

show_help() {
  echo "Usage: $0 [COMMAND]"
  echo ""
  echo "Commands:"
  echo "  install     Build and install the Launch Agent (default)"
  echo "  uninstall   Remove the Launch Agent and binary"
  echo "  build       Only build the binary"
  echo "  help        Show this help message"
}

main() {
  case "${1:-install}" in
  "install")
    check_rust_project
    build_binary
    install_binary
    create_plist
    load_launch_agent
    log_info "Installation completed!"
    log_info "Logs will be available at:"
    log_info "  Stdout: ${HOME}/Library/Logs/${BINARY_NAME}.log"
    log_info "  Stderr: ${HOME}/Library/Logs/${BINARY_NAME}.err"
    ;;
  "uninstall")
    uninstall
    ;;
  "build")
    check_rust_project
    build_binary
    ;;
  "help" | "-h" | "--help")
    show_help
    ;;
  *)
    log_error "Unknown command: $1"
    show_help
    exit 1
    ;;
  esac
}

main "$@"
