#!/bin/sh

cargo build --target aarch64-unknown-linux-musl --release --features cli_app --bin vcgencmd

if [ -n "$1" ]; then
	scp target/aarch64-unknown-linux-musl/release/vcgencmd $1
fi
