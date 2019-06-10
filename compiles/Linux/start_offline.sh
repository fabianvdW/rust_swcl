#!/bin/sh
chmod +x rust_offline 
./rust_offline client_log "$@"
cat "client_log.txt"