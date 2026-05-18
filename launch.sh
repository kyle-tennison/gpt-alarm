#!/usr/bin/bash
sudo busybox devmem 0x2440020 w 0x5
sudo busybox devmem 0x2448030 w 0xA
mkfifo /tmp/gpt-alarm || true
sudo chmod 777 /tmp/gpt-alarm