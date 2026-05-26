#!/usr/bin/bash

cd /etc/night-vision
source ./constants.sh

ps ax | grep "$NV_BIN"
sudo pkill -f "$NV_BIN"

ps ax | grep "$LLAMA_SERVER_BIN"
sudo pkill -f "$LLAMA_SERVER_BIN"

echo "Shut down nightvision process"