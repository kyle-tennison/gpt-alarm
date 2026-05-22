#!/usr/bin/bash
DEBUG_BIN="./target/debug/night-vision"
RELEASE_BIN="./target/release/night-vision"

ps ax | grep "$RELEASE_BIN"
sudo pkill -f "$RELEASE_BIN"

ps ax | grep "$DEBUG_BIN"
sudo pkill -f "$DEBUG_BIN"