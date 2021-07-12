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
