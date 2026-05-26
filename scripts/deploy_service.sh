#!/usr/bin/bash
# only need to run script this once

source ./constants.sh 

mkdir -p -v "$NV_ETC_DIR"
cp -v kill.sh "$NV_ETC_DIR/kill.sh"
cp -v constants.sh "$NV_ETC_DIR/constants.sh"
cp -v launch.sh "$NV_ETC_DIR/launch.sh"
cp -v "$NV_BIN" "$NV_ETC_DIR"

mkdir -p "$NV_VAR_DIR"

cp -v nightvision.service "$SERVICE_DIR"

echo "done moving"

echo "reloading systemd"
sudo systemctl daemon-reload

sudo systemctl start nightvision
echo "sleeping until check"
sleep 3
sudo systemctl status nightvision --no-pager -l


# also need to manually add this to `chrontab -e`: `0 12 * * * /sbin/shutdown -r now`