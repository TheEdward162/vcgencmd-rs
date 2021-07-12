use std::sync::{Arc, Mutex, Weak};

use once_cell::sync::Lazy;

use super::GlobalInstance;
use crate::error::GencmdInitError;

type StaticGlobalInstance = Mutex<Weak<Mutex<GlobalInstance>>>;
static GLOBAL_INSTANCE: Lazy<StaticGlobalInstance> = Lazy::new(|| Mutex::new(Weak::new()));

impl GlobalInstance {
	/// Returns a singleton instance.
	///
	/// If the instance already exists, the same instance is returned.
	///
	/// Otherwise a new instance is created using [`Self::new`](Self::new).
	pub fn instance() -> Result<Arc<Mutex<Self>>, GencmdInitError> {
		let mut lock = GLOBAL_INSTANCE.lock().expect("mutex poisoned");

		let instance = match lock.upgrade() {
			Some(instance) => instance,
			None => {
				let new_instance = Arc::new(Mutex::new(Self::new()?));

				*lock = Arc::downgrade(&new_instance);

				new_instance
			}
		};

		Ok(instance)
	}
}
