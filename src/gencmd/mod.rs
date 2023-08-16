use std::ffi::CStr;

use crate::{error::*, ffi, global::GlobalInstance};

pub mod commands;
pub mod response;

#[cfg(feature = "global_singleton")]
pub mod global;

pub mod unique;

pub trait Command<'a> {
	type Response;

	const COMMAND_STR: &'static str;

	fn parse_response(response: &'a str) -> Result<Self::Response, GencmdCmdError>;
}

/// A wrapper around the gencmd interface.
///
/// This holds an internal buffer for communication and an Arc to the instance.
#[derive(Clone)]
pub struct Gencmd {
	buffer: [u8; ffi::GENCMDSERVICE_MSGFIFO_SIZE as usize],
}
impl Gencmd {
	pub fn new() -> Self {
		Gencmd {
			buffer: [0u8; ffi::GENCMDSERVICE_MSGFIFO_SIZE as usize],
		}
	}

	/// Send a string command and receive a string response.
	///
	/// This function does not parse the response unless it is the error.
	pub fn send_cmd_raw(
		&mut self,
		instance: &mut GlobalInstance,
		command: &str,
	) -> Result<&str, GencmdCmdError> {
		if command.len() >= ffi::GENCMD_MAX_LENGTH as usize {
			return Err(GencmdCmdError::CommandTooLong);
		}
		// this is true now but one never knows with C APIs
		debug_assert!(ffi::GENCMD_MAX_LENGTH <= ffi::GENCMDSERVICE_MSGFIFO_SIZE);

		// use buffer to get that null-terminated goodness of a C string
		self.buffer[..command.len()].copy_from_slice(command.as_bytes());
		self.buffer[command.len()] = 0;

		// SAFETY: We call the retrieve right under and have unique access
		unsafe {
			instance
				.send_command(CStr::from_bytes_with_nul(&self.buffer[..=command.len()]).unwrap())?;
		}

		let len = instance.retrieve_response(&mut self.buffer)?;

		let response = std::str::from_utf8(&self.buffer[..len])?;

		if response.starts_with("error=") {
			let error = Self::parse_error(response)?;
			return Err(error.into());
		}

		Ok(response)
	}

	pub fn send_cmd<'a, C: Command<'a>>(
		&'a mut self,
		instance: &mut GlobalInstance,
	) -> Result<C::Response, GencmdCmdError> {
		let response = self.send_cmd_raw(instance, C::COMMAND_STR)?;

		C::parse_response(response)
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
					"Invalid code".to_string().into(),
				))
			}
		};

		Ok(error)
	}
}
