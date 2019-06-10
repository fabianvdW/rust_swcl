#!/bin/sh
chmod +x rust_offline_skylake
./rust_offline_skylake client_log "$@"
cat "client_log.txt"