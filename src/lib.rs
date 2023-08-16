//! Provides actual bindings and wrapper around VideoCore gencmd interface.
//!
//! The raw bindings are available in the [`ffi`](ffi) module.
//!
//! Currently implemented commands can be seen in [`gencmd::commands`](gencmd::commands) module. Note that
//! since the gencmd interface is a simple textual protocol it can be used even without an explicit implementation (see [`Gencmd::send_cmd_raw`](gencmd::Gencmd::send_cmd_raw)).
//! The advantage of implementing a command specifically is parsing of the response into an appropriate type.
//!
//! ## Usage
//!
//! There are two patterns this library supports out of the box:
//!
//! ### Unique instance
//!
//! The [`GencmdUnique`](gencmd::unique::GencmdUnique) convenience wrapper provides an easy way to create
//! a unique instance to be used on one thread at a time.
//!
//! Note that there can ever only be one instance of the global context per-process.
//! Attempting to create multiple instances returns an error. If you need access from multiple threads simultaneously
//! consider using the global singleton or implementing a similar solution yourself.
//!
//! ```
//! use videocore_gencmd::prelude::*;
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//! 	let mut gencmd = GencmdUnique::new()?;
//!
//! 	let temperature = gencmd.send_cmd::<CmdMeasureTemp>()?;
//!
//! 	println!("VC temperature is: {}", temperature);
//!
//! 	Ok(())
//! }
//! ```
//!
//! ### Global multi-threaded instance
//!
//! The unique instance approach should be sufficient for most usecases. However, sometimes it is required
//! to invoke gencmd interface from multiple threads at one time. For this there is the `global_singleton` feature
//! that exposes a global, lazy-initialized, weak-pointed singleton and a convenience [`GencmdGlobal`](gencmd::global::GencmdGlobal).
//!
//! ```
//! use videocore_gencmd::prelude::*;
//! fn main() {
//! 	let temperature_measurer =
//! 		std::thread::spawn(|| -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
//! 			let mut gencmd = GencmdGlobal::new()?;
//!
//! 			let mut limit = 3;
//! 			while limit > 0 {
//! 				let temperature = gencmd.send_cmd::<CmdMeasureTemp>()?;
//! 				println!("VC temperature is: {}", temperature);
//!
//! 				std::thread::sleep(std::time::Duration::from_secs(1));
//! 				limit -= 1;
//! 			}
//!
//! 			Ok(())
//! 		});
//!
//! 	let freq_measurer =
//! 		std::thread::spawn(|| -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
//! 			let mut gencmd = GencmdGlobal::new()?;
//!
//! 			let mut limit = 3;
//! 			while limit > 0 {
//! 				let frequency = gencmd.send_cmd::<CmdMeasureClockArm>()?;
//! 				println!("Arm frequency is: {} Hz", frequency);
//!
//! 				std::thread::sleep(std::time::Duration::from_secs(1));
//! 				limit -= 1;
//! 			}
//!
//! 			Ok(())
//! 		});
//!
//! 	temperature_measurer.join().unwrap();
//! 	freq_measurer.join().unwrap();
//! }
//! ```
//!
//! ## Features
//!
//! ### `run_bindgen`
//!
//! Runs the bindgen in build.rs. Without this feature enabled the bindings previously generated and pasted into the sources
//! are used instead as the bindgen cannot be used on all platforms.
//!
//! ### `mock_vc_ffi`
//!
//! FFI bindings naturally expect a library to link against. For testing purposes this is not always ideal, so this feature instead
//! exposes C ABI functions that match the bindings and mock the interface to some extent.
//!
//! ### `cli_app`
//!
//! This feature gates optional crates needed to build the cli app which functionally mirrors the C binary `vcgencmd`.
//!
//! ### `global_singleton`
//!
//! Enables the global singleton implementation as described above.
//!
//! ### `serde_models`
//!
//! Derive serde `Serialize` and `Deserialize` for custom command response models.

pub mod error;
pub mod ffi;
pub mod gencmd;
pub mod global;

pub mod prelude;

#[cfg(test)]
mod test {
	use std::sync::Once;

	static ONCE: Once = Once::new();
	pub fn setup_global() {
		ONCE.call_once(|| {
			// edwardium_logger::Logger::new(
			// 	edwardium_logger::targets::stderr::StderrTarget::new(log::Level::Trace, Default::default()),
			// 	std::time::Instant::now()
			// ).init_boxed().expect("Could not initialize logger");
		});
	}
}
