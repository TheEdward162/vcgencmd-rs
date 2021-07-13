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
	///
	/// ### Memory race
	/// There is a race between the last strong reference being decremented and dropped and the `GlobalInstance` inside of it being deinitialized.
	/// This means that for a little while this function may return [`GencmdInitError::AlreadyInitialized`](GencmdInitError::AlreadyInitialized) even
	/// when the global singleton is no longer being held.
	///
	/// ```text
	///                            GlobalInstnace Drop starts
	///                            |    Error here as an instance still exists
	///                            |    |    GlobalInstnace Drop ends
	///                            v    v    v
	/// ONE_INSTANCE: |------------|_________|
	/// Weak Arc    : |-------|_________|....
	/// Strong Arc  : |-------|         |....
	///                       ^         ^
	///                       |         Attempt to create new singleton instance
	///                       Drop Arc here, weak becomes non-upgradable
	/// ```
	///
	/// To alleviate this, this function may spin around that error until the `GlobalInstnace::drop` finishes. However, if the second instance
	/// is being held by an external actor and not through the global singleton functionality, then the spin will never finish unless it is
	/// limited. For that purpose, this function takes the `SPIN_RETRY_LIMIT` generic argument which can be used to limit the maximum number of
	/// retry spins before an error is returned (that is, `new` will be attempted `SPIN_RETRY_LIMIT + 1` times).
	pub fn instance<const SPIN_RETRY_LIMIT: usize>() -> Result<Arc<Mutex<Self>>, GencmdInitError> {
		let mut lock = GLOBAL_INSTANCE.lock().expect("mutex poisoned");

		if let Some(instance) = lock.upgrade() {
			return Ok(instance)
		}

		let instance = {
			let mut limit = SPIN_RETRY_LIMIT;
			let instance = loop {
				match Self::new() {
					Err(GencmdInitError::AlreadyInitialized) => {
						std::thread::yield_now();
					}
					Err(err) => return Err(err),
					Ok(instance) => break instance
				}

				if limit == 0 {
					return Err(GencmdInitError::AlreadyInitialized)
				}
				limit -= 1;
			};

			log::trace!(
				"Spun {} times out of maximum {}",
				SPIN_RETRY_LIMIT - limit,
				SPIN_RETRY_LIMIT
			);

			instance
		};

		let instance = Arc::new(Mutex::new(instance));
		*lock = Arc::downgrade(&instance);

		Ok(instance)
	}
}
