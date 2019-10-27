#!/usr/bin/env bash

pid=$(lsof -t -i udp:4485)
kill -9 ${pid}

echo "Starting server"
cargo run --bin server &
sleep 1
cargo run --bin client
