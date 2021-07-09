use std::{ffi::CStr, sync::{Arc, Mutex}};

use crate::{ffi, error::*, GlobalInstance};

pub mod response;
pub mod commands;

/// A wrapper around the gencmd interface.
///
/// This holds an internal buffer for communication and an Arc to the instance.
#[derive(Clone)]
pub struct Gencmd {
	instance: Arc<Mutex<GlobalInstance>>,
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
		
		// use buffer to get that null-terminated goodness of a C string
		self.buffer[.. command.len()].copy_from_slice(command.as_bytes());
		self.buffer[command.len()] = 0;

		let len = {
			let mut lock = self.instance.lock().expect("mutex poisoned");

			// SAFETY: We call the retrieve right under
			unsafe {
				lock.send_command(
					CStr::from_bytes_with_nul(&self.buffer[..= command.len()]).unwrap()
				)?;
			}

			let len = lock.retrieve_response(
				&mut self.buffer
			)?;

			len
		};

		let response = std::str::from_utf8(&self.buffer[.. len])?;

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