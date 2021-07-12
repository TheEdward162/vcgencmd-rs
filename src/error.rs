use thiserror::Error;

use super::ffi;

#[derive(Error, Debug)]
#[repr(u8)]
pub enum VcosError {
	#[error("Resource temporarily unavailable")]
	Again = ffi::VCOS_STATUS_T::VCOS_EAGAIN.0 as u8,
	#[error("No such file or directory")]
	NoEntry = ffi::VCOS_STATUS_T::VCOS_ENOENT.0 as u8,
	#[error("No space left on device")]
	NoSpace = ffi::VCOS_STATUS_T::VCOS_ENOSPC.0 as u8,
	#[error("Invalid argument")]
	Invalid = ffi::VCOS_STATUS_T::VCOS_EINVAL.0 as u8,
	#[error("Permission denied")]
	Access = ffi::VCOS_STATUS_T::VCOS_EACCESS.0 as u8,
	#[error("Cannot allocate memory")]
	NoMemory = ffi::VCOS_STATUS_T::VCOS_ENOMEM.0 as u8,
	#[error("Function not implemented")]
	NoSys = ffi::VCOS_STATUS_T::VCOS_ENOSYS.0 as u8,
	#[error("File exists")]
	Exist = ffi::VCOS_STATUS_T::VCOS_EEXIST.0 as u8,
	#[error("No such device or address")]
	NxIo = ffi::VCOS_STATUS_T::VCOS_ENXIO.0 as u8,
	#[error("Interrupted system call")]
	Interrupt = ffi::VCOS_STATUS_T::VCOS_EINTR.0 as u8
}

impl ffi::VCOS_STATUS_T {
	pub fn to_result(self) -> Result<(), VcosError> {
		let error = match self {
			ffi::VCOS_STATUS_T::VCOS_SUCCESS => return Ok(()),
			ffi::VCOS_STATUS_T::VCOS_EAGAIN => VcosError::Again,
			ffi::VCOS_STATUS_T::VCOS_ENOENT => VcosError::NoEntry,
			ffi::VCOS_STATUS_T::VCOS_ENOSPC => VcosError::NoSpace,
			ffi::VCOS_STATUS_T::VCOS_EINVAL => VcosError::Invalid,
			ffi::VCOS_STATUS_T::VCOS_EACCESS => VcosError::Access,
			ffi::VCOS_STATUS_T::VCOS_ENOMEM => VcosError::NoMemory,
			ffi::VCOS_STATUS_T::VCOS_ENOSYS => VcosError::NoSys,
			ffi::VCOS_STATUS_T::VCOS_EEXIST => VcosError::Exist,
			ffi::VCOS_STATUS_T::VCOS_ENXIO => VcosError::NxIo,
			ffi::VCOS_STATUS_T::VCOS_EINTR => VcosError::Interrupt,
			_ => unreachable!()
		};

		Err(error)
	}
}

#[derive(Error, Debug)]
pub enum GencmdInitError {
	#[error("Failed to initialize vcos: {0}")]
	VcosInit(VcosError),
	#[error("Failed to initialize vchi")]
	VchiInit,
	#[error("Failed to create vchi connection")]
	VchiConnect,
	#[error("Another instance is already initialized")]
	AlreadyInitialized
}

#[derive(Error, Debug)]
pub enum GencmdDeinitError {
	#[error("Failed to destroy vchi connection")]
	VchiDisconnect
}

#[derive(Error, Debug)]
pub enum GencmdCmdError {
	#[error(
		"Command is too long for this interface (max len {})",
		ffi::GENCMD_MAX_LENGTH
	)]
	CommandTooLong,
	#[error("Failed to send command")]
	Send,
	#[error("Failed to read response")]
	Read,
	#[error("The received response is not valid utf8: {0}")]
	Utf8(#[from] std::str::Utf8Error),
	#[error(transparent)]
	ErrorResponse(#[from] GencmdErrorResponse),
	#[error("Response has invalid format: {0}")]
	InvalidResponseFormat(Box<dyn std::error::Error + Send + Sync>)
}
impl GencmdCmdError {
	pub fn from_invalid_format(error: impl std::error::Error + Send + Sync + 'static) -> Self {
		GencmdCmdError::InvalidResponseFormat(Box::new(error))
	}
}

#[derive(Error, Debug)]
pub enum GencmdErrorResponse {
	#[error("Command not registered")]
	CommandNotRegistered,
	#[error("Invalid arguments")]
	InvalidArguments
}
impl GencmdErrorResponse {
	pub const fn code(&self) -> i32 {
		match self {
			GencmdErrorResponse::CommandNotRegistered => 1,
			GencmdErrorResponse::InvalidArguments => 2
		}
	}
}
