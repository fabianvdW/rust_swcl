#!/bin/sh
chmod +x rust_online_skylake
./rust_online_skylake "$@"
cat "client_log.txt"