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
