# videocore-gencmd

Actual FFI bindings and higher-level reimplementation of the Videocore gencmd interface in Rust.

## Bindings

Bindings are generated using [`bindgen`](https://github.com/rust-lang/rust-bindgen) from code available at https://github.com/raspberrypi/userland.

The actual generation is gated behind the `run_bindgen` feature because not all platforms are capable of running bindgen (because bindgen uses dlopen which doesn't work on musl). If this feature is not chosen pre-generated bindings are copy-pased into the source.

## Architecture

Since the broadcom libraries are very much unfinished, bugs also affect the architecture of the project. The videocore state and instances may only be initialized once pre-process.

This is checked at runtime by keeping that information in an `AtomicBool` flag.

One of the ways to share the instance between threads in implemented under the `global_singleton` feature. This provider a global, lazy-initialized, weak-reference-counted singleton. This means that on the first use the global state is initialized. When all current uses are dropped the global singleton is deinitialized as well. When another instance is needed it is initialized on demand again.

Apart from that an instance of `GlobalInstance` can be initialized through its `new` constructor.

## Commands

Since the gencmd interface is a simple textual protocol, commands can be used even without specific implementation provided. The response is then returned as a string. Parsing utilities are provided in the crate and response parsing is implemented. Errors returned from commands sent through the wrapping interface are always parsed.

## CLI

A cli is implemented in `src/bin/vcgencmd` that implements the same functionality as the original C binary (as available in raspberrypi-userland/host_applications/linux/apps/gencmd/gencmd.c). The cli app allows both raw processing (which sends the command as provided and only parses error responses) and a response-parsing command recognition for commands which are implemented.

Here is the output of `videocore-gencmd --help`:
```
videocore-gencmd 0.1.0

USAGE:
    vcgencmd [FLAGS] [OPTIONS] <command>...

FLAGS:
    -h, --help       Prints help information
    -r, --raw        Do not attempt to recognize the command nor parse the response (errors are always parsed)
    -V, --version    Prints version information

OPTIONS:
    -v, --verbosity <verbosity>    Level of verbosity [default: Off]  [possible values: Off, Error, Warn, Info, Debug,
                                   Trace]

ARGS:
    <command>...
```

## Building

Real bindings link to the broadcom VideoCore libraries `vchiq_arm`, `vcos` and `bcm_host` usually found in `/opt/vc/lib` (this is configured in build.rs).

### Musl

To build this crate with real bindings on musl dynamic linking has to be enabled for that target. A good way to do this for a native target is to update cargo config (either globally or in `.cargo/config`) with:

```toml
[build]
rustflags = ["-C", "target-feature=-crt-static"]
```

For a cross compiled target use `[target.target-triple-here]` instead of `[build]`:

```toml
[target.aarch64-unknown-linux-musl] # or any other target
rustflags = ["-C", "target-feature=-crt-static"]
```

This is only needed with real dynamic bindings. It might be possible to build the vc libraries from source as static libraries instead.
