#!/usr/bin/bash
DEBUG_BIN="./target/debug/night-vision"
RELEASE_BIN="./target/release/night-vision"
LLAMA_SERVER_BIN="/opt/llama.cpp/build/bin/llama-server"

ps ax | grep "$RELEASE_BIN"
sudo pkill -f "$RELEASE_BIN"

ps ax | grep "$DEBUG_BIN"
sudo pkill -f "$DEBUG_BIN"

ps ax | grep "$LLAMA_SERVER_BIN"
sudo pkill -f "$LLAMA_SERVER_BIN"