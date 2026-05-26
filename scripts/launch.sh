#!/usr/bin/bash

cd /etc/night-vision
source ./constants.sh
echo "starting program as $STAGE"

sudo busybox devmem 0x2440020 w 0x5
sudo busybox devmem 0x2448030 w 0xA
echo "set GPIO config"

sudo systemctl restart nvargus-daemon
echo "restarted nvargus-daemon"

while ! systemctl is-active --quiet nvargus-daemon; do
        echo "Waiting for nvargus-daemon"
        sleep 1
    done

echo "confirmed nvargus-daemon active" 

LOGFILE="${NV_VAR_DIR}/${STAGE}.log"

echo "" > "$LOGFILE"

echo "spawning rust at $NV_BIN"
sudo "$NV_BIN" > "$LOGFILE" 2>&1