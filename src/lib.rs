use std::{ffi::CStr, sync::{Arc, Weak, Mutex, Once}};

pub mod commands;
pub mod error;
pub mod ffi;
pub mod response;

#[cfg(feature = "simulated_ffi")]
pub mod simulated;

use error::*;

static ONCE_INIT: Once = Once::new();
static mut GLOBAL_INSTANCE: Option<Mutex<Weak<GlobalInstance>>> = None;

struct GlobalInstance {
	instance: ffi::VCHI_INSTANCE_T,
	#[allow(dead_code)] // :shrug:
	connection: *mut ffi::VCHI_CONNECTION_T
}
impl GlobalInstance {
	/// Returns a singleton instance.
	///
	/// If the instance already exists, the same instance is returned.
	///
	/// Otherwise a new instance is created and returned.
	pub fn instance() -> Result<Arc<Self>, GencmdInitError> {
		ONCE_INIT.call_once(
			// SAFETY: Is only ever run once and other threads are blocked while it is running
			|| unsafe {
				GLOBAL_INSTANCE.replace(
					Mutex::new(Weak::new())
				);
			}
		);

		// SAFETY: The above ONCE_INIT makes sure the mutex is initialized
		let mut lock = unsafe { GLOBAL_INSTANCE.as_ref().unwrap().lock().expect("mutex poisoned") };
		
		let instance = match lock.upgrade() {
			Some(instance) => instance,
			None => {
				let new_instance = Arc::new(
					Self::new()?
				);

				*lock = Arc::downgrade(&new_instance);

				new_instance
			}
		};

		Ok(instance)
	}

	fn new() -> Result<Self, GencmdInitError> {
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

		Ok(
			GlobalInstance {
				instance,
				connection
			}
		)
	}

	/// Sends a command to the instance.
	///
	/// Will panic if this instance has been deinitialized.
	pub fn send_command(&self, command: &CStr) -> Result<(), GencmdCmdError> {
		const FORMAT: &'static [u8] = b"%s\0";

		if self.is_deinitialized() {
			panic!("This instance has been deinitialized");
		}

		if command.to_bytes().len() + 1 > ffi::GENCMD_MAX_LENGTH as usize {
			return Err(GencmdCmdError::CommandTooLong)
		}

		// SAFETY: Things are initialized, the strings are null terminated,
		// the format takes one string argument (internally calls vsnprintf)
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
	/// Will panic if this instance has been deinitialized.
	pub fn retrieve_response(&self, buffer: &mut [u8]) -> Result<usize, GencmdCmdError> {
		if self.is_deinitialized() {
			panic!("This instance has been deinitialized");
		}

		// SAFETY: we have mutable access to buffer and pass in the correct buffer len
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
		let len = buffer
			.iter()
			.position(|&b| b == 0)
			.unwrap_or(buffer.len());

		Ok(len)
	}

	/// Returns true if `self.deinit` has been called at least once on this instance.
	pub fn is_deinitialized(&self) -> bool {
		self.instance == std::ptr::null_mut()
	}

	/// Deinitializes `self`, returning a potential error.
	///
	/// It's okay to call this multiple times, but after a deinitialization other
	/// methods that work with this instance will panic.
	///
	/// If `deinit` is not called it will be called in `drop` and will panic on error.
	pub fn deinit(&mut self) -> Result<(), GencmdDeinitError> {
		if self.is_deinitialized() {
			return Ok(())
		}

		unsafe { ffi::vc_gencmd_stop() };

		let result = unsafe { ffi::vchi_disconnect(self.instance) };
		if result != 0 {
			return Err(GencmdDeinitError::VchiDisconnect)
		}

		unsafe { ffi::vcos_deinit() };

		self.instance = std::ptr::null_mut();

		Ok(())
	}
}
impl Drop for GlobalInstance {
	fn drop(&mut self) {
		match self.deinit() {
			Ok(()) => (),
			Err(err) => {
				log::error!("gencmd deinitialization failed inside drop: {}", err);
				panic!("gencmd deinitialization failed inside drop: {}", err);
			}
		}
	}
}

/// A wrapper around the gencmd interface.
///
/// This holds an internal buffer for communication and an Arc to the instance.
#[derive(Clone)]
pub struct Gencmd {
	instance: Arc<GlobalInstance>,
	buffer: [u8; ffi::GENCMDSERVICE_MSGFIFO_SIZE as usize]
}
impl Gencmd {
	pub fn new() -> Result<Self, GencmdInitError> {
		let instance = GlobalInstance::instance()?;

		Ok(Gencmd {
			instance,
			buffer: [0u8; ffi::GENCMDSERVICE_MSGFIFO_SIZE as usize]
		})
	}

	/// Send a string command and receive a string response.
	///
	/// This function does not parse the response unless it is the error.
	pub fn cmd_send(&mut self, command: &str) -> Result<&str, GencmdCmdError> {
		if command.len() >= ffi::GENCMD_MAX_LENGTH as usize {
			return Err(GencmdCmdError::CommandTooLong)
		}
		
		// use buffer to get that null-terminated goodness of a string
		self.buffer[.. command.len()].copy_from_slice(command.as_bytes());
		self.buffer[command.len()] = 0;

		self.instance.send_command(
			CStr::from_bytes_with_nul(&self.buffer[..= command.len()]).unwrap()
		)?;

		let len = self.instance.retrieve_response(
			&mut self.buffer
		)?;

		let response = std::str::from_utf8(&self.buffer[.. len])?;
		log::debug!("vc response: {}", response);

		if response.starts_with("error=") {
			let error = Self::parse_error(response)?;
			return Err(error.into())
		}

		Ok(response)
	}

	fn parse_error(response: &str) -> Result<GencmdErrorResponse, GencmdCmdError> {
		let (response, code) = response::parse_field_simple::<i32>(response, "error")
			.map_err(GencmdCmdError::from_invalid_format)?;
		let (_, message) = response::parse_field_simple::<&str>(response, "error_msg")
			.map_err(GencmdCmdError::from_invalid_format)?;

		log::error!("gencmd returned: code: {} message: {}", code, message);

		let error = match code {
			1 => GencmdErrorResponse::CommandNotRegistered,
			2 => GencmdErrorResponse::InvalidArguments,
			_ => {
				return Err(GencmdCmdError::InvalidResponseFormat(
					"Invalid code".to_string().into()
				))
			}
		};

		Ok(error)
	}
}
