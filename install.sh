#!/bin/bash

# Exit on error
set -e

# Run the build script
./build.sh

# Determine the OS
OS=$(uname)

# Install the service
if [ "$OS" == "Linux" ]; then
    echo "Installing for Linux..."
    sudo cp -r dist /opt/audio_extractor
    sudo cp audio_extractor.service /etc/systemd/system/
    sudo systemctl daemon-reload
    sudo systemctl enable audio_extractor
    sudo systemctl start audio_extractor
    echo "Installation complete."
elif [ "$OS" == "Darwin" ]; then
    echo "Installing for macOS..."
    sudo cp -r dist /opt/audio_extractor
    sudo cp com.audioextractor.plist /Library/LaunchDaemons/
    sudo launchctl load /Library/LaunchDaemons/com.audioextractor.plist
    echo "Installation complete."
else
    echo "Unsupported OS: $OS"
    exit 1
fi
