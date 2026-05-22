#!/usr/bin/bash
echo "starting program"

sudo busybox devmem 0x2440020 w 0x5
sudo busybox devmem 0x2448030 w 0xA
echo "set GPIO config"

sudo systemctl restart nvargus-daemon
echo "restarted nvargus-daemon"

RELEASE_BIN="./target/release/gpt-alarm"
ps ax | grep "$RELEASE_BIN"
sudo pkill -f "$RELEASE_BIN"
echo "killed release processes"

DEBUG_BIN="./target/debug/gpt-alarm"
ps ax | grep "$DEBUG_BIN"
sudo pkill -f "$DEBUG_BIN"
echo "killed debug processes"

while ! systemctl is-active --quiet nvargus-daemon; do
        echo "Waiting for nvargus-daemon"
        sleep 1
    done
echo "confirmed nvargus-daemon active" 

if [[ "$RELEASE" == "1" ]]; then
    echo "stage: Release"
    STAGE="release"
    BIN="$RELEASE_BIN"
else 
    echo "stage: Debug"
    STAGE="debug"
    BIN="$DEBUG_BIN"
fi

echo "" > "${STAGE}.log"

echo "spawning rust"
sudo -b su -c "$BIN" > "${STAGE}.log" 2>&1 &

echo "launch.sh completed"