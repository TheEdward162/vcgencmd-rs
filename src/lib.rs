pub mod commands;
pub mod error;
pub mod ffi;
pub mod response;

#[cfg(feature = "simulated_ffi")]
pub mod simulated;

use error::*;

/// Best-effort wrapper around gencmd interface.
///
/// It's probably not a good idea to have multiple instances of this within one process.
/// There is no documentation so it's hard to know what exactly is allowed.
// TODO: Maybe a global lazy singleton?
pub struct Gencmd {
	instance: ffi::VCHI_INSTANCE_T,
	#[allow(dead_code)] // :shrug:
	connection: *mut ffi::VCHI_CONNECTION_T,
	buffer: [u8; ffi::GENCMDSERVICE_MSGFIFO_SIZE as usize]
}
impl Gencmd {
	pub fn new() -> Result<Self, GencmdInitError> {
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

		Ok(Gencmd {
			instance,
			connection,
			buffer: [0u8; ffi::GENCMDSERVICE_MSGFIFO_SIZE as usize]
		})
	}

	/// Send a string command and receive a string response.
	///
	/// This function does not parse the command
	pub fn cmd_send(&mut self, command: &str) -> Result<&str, GencmdCmdError> {
		if self.is_deinitialized() {
			panic!("This instance has been deinitialized");
		}

		const FORMAT: &'static [u8] = b"%s\0";

		if command.len() > ffi::GENCMD_MAX_LENGTH as usize {
			return Err(GencmdCmdError::CommandTooLong)
		}

		// use buffer to get that null-terminated goodness of a string
		self.buffer[.. command.len()].copy_from_slice(command.as_bytes());
		self.buffer[command.len()] = 0;

		// SAFETY: Things are initialized, the strings are null terminated,
		// the format takes one string argument (internally valls vsnprintf)
		let result = unsafe {
			ffi::vc_gencmd_send(
				FORMAT.as_ptr() as *const std::os::raw::c_char,
				self.buffer.as_ptr()
			)
		};
		if result != 0 {
			return Err(GencmdCmdError::Send)
		}

		// SAFETY: we have mutable access to self and pass in the correct buffer len
		let result = unsafe {
			ffi::vc_gencmd_read_response(
				self.buffer.as_mut_ptr() as *mut std::os::raw::c_char,
				self.buffer.len() as std::os::raw::c_int
			)
		};
		if result != 0 {
			return Err(GencmdCmdError::Read)
		}

		let response = {
			// strlen, but sane
			let len = self
				.buffer
				.iter()
				.position(|&b| b == 0)
				.unwrap_or(self.buffer.len());

			std::str::from_utf8(&self.buffer[.. len])?
		};
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
// SAFETY: Interally the thing uses a mutex
unsafe impl Send for Gencmd {}
impl Drop for Gencmd {
	fn drop(&mut self) {
		match self.deinit() {
			Ok(()) => (),
			Err(err) => {
				log::error!("Gencmd deinitialization failed inside drop: {}", err);
				panic!("Gencmd deinitialization failed inside drop: {}", err);
			}
		}
	}
}
