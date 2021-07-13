//! ## Usage
//!
//! There are two patterns this library supports out of the box:
//!
//! ### Unique instance
//!
//! The [`GencmdUnique`](gencmd::unique::GencmdUnique) convenience wrapper provides an easy way to create
//! a unique instance to be used on one thread at a time.
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
