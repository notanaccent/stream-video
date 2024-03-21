#!/bin/bash

# Kill subprocesses on exit
cleanup() {
    echo "Terminating subprocesses..."
    kill $ws_server_pid $http_server_pid
    exit
}
trap cleanup SIGINT SIGTERM

# Reset ouput dir
rm -r ./output/*
mkdir -p ./output/dash

# Start websocket server on localhost:8080
# python3 ./src/main.py &
cargo build --release
cargo run --release &
ws_server_pid=$!

# Serve project directory on localhost:8000
python3 -m http.server &
http_server_pid=$!

# Client-side video capture
sleep 1
echo "Opening video capture..."
sleep 1
open http://localhost:8000/static/capture.html &

# Client-side video viewer
echo "Opening viewer in 5 seconds..."
sleep 5
open http://localhost:8000/static/viewer.html &

read -p "Press any key to exit..."
cleanup
