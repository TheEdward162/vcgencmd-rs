use std::{
	ffi::CStr,
	sync::atomic::{AtomicBool, Ordering as AtomicOrdering}
};

use crate::{error::*, ffi};

#[cfg(feature = "global_singleton")]
pub mod singleton;

static ONE_INSTANCE: AtomicBool = AtomicBool::new(false);

pub struct GlobalInstance {
	instance: ffi::VCHI_INSTANCE_T,
	#[allow(dead_code)] // :shrug:
	connection: *mut ffi::VCHI_CONNECTION_T
}
impl GlobalInstance {
	/// Initializes a new instance of videocore connection.
	///
	/// Returns an error if another connection already exists.
	///
	/// It is up to the caller to share that one connection to all
	/// places where they might need it. However, if "global_singleton"
	/// feature is enabled, a global, lazy singleton implementation is
	/// provided in `singleton` module.
	pub fn new() -> Result<Self, GencmdInitError> {
		if ONE_INSTANCE.swap(true, AtomicOrdering::Acquire) {
			return Err(GencmdInitError::AlreadyInitialized)
		}

		log::info!("Initializing videocore gencmd instance");

		unsafe {
			ffi::vcos_init()
				.to_result()
				.map_err(GencmdInitError::VcosInit)?
		};

		let mut instance: ffi::VCHI_INSTANCE_T = std::ptr::null_mut();
		let result = unsafe { ffi::vchi_initialise(&mut instance) };
		if result != 0 || instance == std::ptr::null_mut() {
			return Err(GencmdInitError::VchiInit)
		}

		let result = unsafe { ffi::vchi_connect(std::ptr::null_mut(), 0, instance) };
		if result != 0 {
			return Err(GencmdInitError::VchiConnect)
		}

		let mut connection: *mut ffi::VCHI_CONNECTION_T = std::ptr::null_mut();
		unsafe { ffi::vc_vchi_gencmd_init(instance, &mut connection, 1) };

		log::debug!("instance: {:p}", instance);
		log::debug!("connection: {:p}", connection);

		Ok(GlobalInstance {
			instance,
			connection
		})
	}

	/// Sends a command to the instance.
	///
	/// ### Panic
	/// Will panic if this instance has been deinitialized.
	///
	/// ### Safety
	/// Looks like the response must be picked up before another thread issues a send, otherwise
	/// the entire _system_ gets broken and all communication with vc gencmd starts going haywire.
	#[allow(unused_unsafe)]
	pub unsafe fn send_command(&mut self, command: &CStr) -> Result<(), GencmdCmdError> {
		const FORMAT: &'static [u8] = b"%s\0";

		if self.is_deinitialized() {
			panic!("This instance has been deinitialized");
		}

		if command.to_bytes().len() + 1 > ffi::GENCMD_MAX_LENGTH as usize {
			return Err(GencmdCmdError::CommandTooLong)
		}

		// SAFETY: Things are initialized, the strings are null terminated,
		// the format takes one string argument (internally calls vsnprintf)
		// There are also no races because internally this locks a mutex.
		log::debug!("sending vc command: {:?}", command);
		let result = unsafe {
			ffi::vc_gencmd_send(
				FORMAT.as_ptr() as *const std::os::raw::c_char,
				command.as_ptr()
			)
		};
		if result != 0 {
			return Err(GencmdCmdError::Send)
		}

		Ok(())
	}

	/// Retrieves the response from the instance.
	///
	/// Returns number of bytes read into `buffer` (excluding the null terminator).
	///
	/// ### Panic
	/// Will panic if this instance has been deinitialized.
	pub fn retrieve_response(&mut self, buffer: &mut [u8]) -> Result<usize, GencmdCmdError> {
		if self.is_deinitialized() {
			panic!("This instance has been deinitialized");
		}

		// SAFETY: we have mutable access to buffer and pass in the correct buffer len
		// There are also no races because internally this locks a mutex.
		let result = unsafe {
			ffi::vc_gencmd_read_response(
				buffer.as_mut_ptr() as *mut std::os::raw::c_char,
				buffer.len() as std::os::raw::c_int
			)
		};
		if result != 0 {
			return Err(GencmdCmdError::Read)
		}

		// strlen, but sane
		let len = buffer.iter().position(|&b| b == 0).unwrap_or(buffer.len());

		log::debug!(
			"retrieved vc response: {:?}",
			CStr::from_bytes_with_nul(&buffer[..= len]).unwrap()
		);

		Ok(len)
	}

	/// Returns true if `self.deinit` has been called at least once on this instance.
	pub fn is_deinitialized(&self) -> bool {
		self.instance == std::ptr::null_mut()
	}

	/// Deinitializes `self`, returning a potential error.
	///
	/// If `deinit` is not called,[`deinit_ref_mut`](Self::deinit_ref_mut) will be called in `drop` and will panic on error.
	pub fn deinit(mut self) -> Result<(), GencmdDeinitError> {
		self.deinit_ref_mut()
	}

	/// Deinitializes `self`, returning a potential error.
	///
	/// It's okay to call this multiple times, but after a deinitialization other
	/// methods that work with this instance will panic.
	///
	/// If `deinit_ref_mut` is not called it will be called in `drop` and will panic on error.
	pub fn deinit_ref_mut(&mut self) -> Result<(), GencmdDeinitError> {
		if self.is_deinitialized() {
			return Ok(())
		}

		log::info!("Deinitializing videocore gencmd instance");

		unsafe { ffi::vc_gencmd_stop() };

		let result = unsafe { ffi::vchi_disconnect(self.instance) };
		if result != 0 {
			return Err(GencmdDeinitError::VchiDisconnect)
		}

		unsafe { ffi::vcos_deinit() };

		self.instance = std::ptr::null_mut();

		ONE_INSTANCE.store(false, AtomicOrdering::Release);

		Ok(())
	}
}
// SAFETY: The vc state is process-wide
unsafe impl Send for GlobalInstance {}
impl Drop for GlobalInstance {
	fn drop(&mut self) {
		match self.deinit_ref_mut() {
			Ok(()) => (),
			Err(err) => {
				log::error!("gencmd deinitialization failed inside drop: {}", err);
				panic!("gencmd deinitialization failed inside drop: {}", err);
			}
		}
	}
}

// these tests will only pass if there are no other tests that use the global singleton
#[cfg(test)]
mod test {
	use super::{GencmdInitError, GlobalInstance};

	#[test]
	#[ignore]
	fn global_instance_is_unique() {
		let instance = GlobalInstance::new().unwrap();

		match GlobalInstance::new() {
			Err(GencmdInitError::AlreadyInitialized) => (),
			_ => panic!("global instance must be unique")
		}

		instance.deinit().unwrap();
	}

	#[test]
	#[ignore]
	fn global_instance_is_unique_respects_drop() {
		let instance = GlobalInstance::new().unwrap();
		std::mem::drop(instance);

		GlobalInstance::new().unwrap();
	}
}
