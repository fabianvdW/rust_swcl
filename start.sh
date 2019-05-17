#!/bin/sh
chmod +x rust_swcl
./rust_swcl "$@"
cat "client_log.txt"