#!/bin/sh
chmod +x rust_online_broadwell
./rust_online_broadwell "$@"
cat "client_log.txt"