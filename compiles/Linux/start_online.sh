#!/bin/sh
chmod +x rust_online 
./rust_online "$@"
cat "client_log.txt"