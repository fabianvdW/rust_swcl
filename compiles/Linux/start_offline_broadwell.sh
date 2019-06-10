#!/bin/sh
chmod +x rust_offline_broadwell
./rust_offline_broadwell client_log "$@"
cat "client_log.txt"