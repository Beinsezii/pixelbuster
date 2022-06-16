#!/usr/bin/env bash

LINUX="./target/release/libpixelbuster.so"
WIN="./target/release/pixelbuster.dll"

cargo build --release

mkdir ./lib 2>/dev/null

if [ -f "$LINUX" ]; then
    cp "$LINUX" ./lib/libpixelbuster.so
    cargo build --release --target x86_64-pc-windows-gnu
    cp "./target/x86_64-pc-windows-gnu/release/pixelbuster.dll" ./lib/pixelbuster.dll
fi

if [ -f "$WIN" ]; then
    cp "$WIN" ./lib/pixelbuster.dll
    cargo build --release --target x86_64-unknown-linux-gnu
    cp "./target/x86_64-unknown-linux-gnu/release/libpixelbuster.so" ./lib/libpixelbuster.so
fi
