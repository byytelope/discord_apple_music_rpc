#!/bin/bash -x

BIN_NAME="damr"
BIN_PATH=/opt/byytelope/bin

cargo build --release

sudo mkdir -p $BIN_PATH
sudo mv target/release/discord_apple_music_rpc $BIN_PATH/$BIN_NAME

chmod +x $BIN_PATH/$BIN_NAME

cat << EOF > $HOME/Library/LaunchAgents/com.byytelope.damr.plist
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.byytelope.damr</string>
    <key>ProgramArguments</key>
    <array>
       <string>/opt/byytelope/bin/damr</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
</dict>
</plist>
EOF

launchctl load -w $HOME/Library/LaunchAgents/com.byytelope.damr.plist
