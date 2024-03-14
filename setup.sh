#!/bin/bash

BIN_NAME="discord_apple_music_rpc"
BIN_PATH=/opt/byytelope/bin

cargo build --release

sudo mkdir -p $BIN_PATH
sudo mv target/release/$BIN_NAME $BIN_PATH/$BIN_NAME

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
       <string>/opt/byytelope/bin/discord_apple_music_rpc</string>
   </array>
   <key>RunAtLoad</key>
   <true/>
   <key>KeepAlive</key>
   <true/>
    <key>EnvironmentVariables</key>
    <dict>
        <key>DISCORD_APP_ID</key>
        <string>${DISCORD_APP_ID}</string>
    </dict>
</dict>
</plist>
EOF

launchctl load -w $HOME/Library/LaunchAgents/com.byytelope.damr.plist
