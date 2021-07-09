# videocore-gencmd

Actual FFI bindings and reimplementation of the Videocore gencmd interface in Rust.

## Bindings

Bindings are generated using [`bindgen`](https://github.com/rust-lang/rust-bindgen) from code available at https://github.com/raspberrypi/userland.

The actual generation is gated behind the `run_bindgen` feature because not all platforms are capable of running bindgen (because bindgen uses dlopen which doesn't work on musl). If this feature is not chosen pre-generated bindings are copy-pased into the source.

## Architecture

Since the broadcom libraries are very much unfinished, bugs also affect the architecture of the project. The videocore state and instances are provided as a global, lazy-initialized, weak-reference-counted singleton. This means that on the first use the global state is initialized and when all current uses are dropped the global singleton is deinitialized as well. When another instance is needed it is initialized on demand again.

## Commands

Since the gencmd interface is a simple textual protocol, commands can be used even without specific implementation provided. The response is then returned as a string. Parsing utilities are provided in the crate and response parsing is implemented.

## [WIP] CLI

A cli should be implemented in `src/bin/vcgencmd` that mirrors the original C binary (as available in raspberrypi-userland/host_applications/linux/apps/gencmd/gencmd.c).