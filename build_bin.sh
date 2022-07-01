#!/usr/bin/env bash
BIN="gui"

LINUX="./target/release/${BIN}"
WIN="./target/release/${BIN}.exe"

cargo build --release --features=gui

mkdir ./bin 2>/dev/null

if [ -f "$LINUX" ]; then
    cp "$LINUX" ./bin/pixelbuster
    cargo build --release --features=gui --target x86_64-pc-windows-gnu
    cp "./target/x86_64-pc-windows-gnu/release/${BIN}.exe" "./bin/pixelbuster_${BIN}.exe"
fi

if [ -f "$WIN" ]; then
    cp "$WIN" ./bin/pixelbuster.exe
    cargo build --release --features=gui --target x86_64-unknown-linux-gnu
    cp "./target/x86_64-unknown-linux-gnu/release/${BIN}" "./bin/pixelbuster_${BIN}"
fi
