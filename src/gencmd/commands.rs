//! This module implements some of the commands and parsing of their responses, as well as the models of their responses (if needed).
//!
//! More commands should be implemented on demand.

use super::{
	response::{self, IntRadix},
	Command, GencmdCmdError,
};

#[cfg(feature = "serde_models")]
use serde::{Deserialize, Serialize};

pub struct CmdCommands;
impl<'a> Command<'a> for CmdCommands {
	type Response = Vec<&'a str>;

	const COMMAND_STR: &'static str = "commands";

	fn parse_response(response: &'a str) -> Result<Self::Response, GencmdCmdError> {
		let (_, commands) = response::parse_field_simple::<&str>(response, "commands")
			.map_err(GencmdCmdError::from_invalid_format)?;

		Ok(commands.split(", ").collect())
	}
}

pub struct CmdMeasureTemp;
impl<'a> Command<'a> for CmdMeasureTemp {
	type Response = f32;

	const COMMAND_STR: &'static str = "measure_temp";

	fn parse_response(response: &'a str) -> Result<Self::Response, GencmdCmdError> {
		let (_, temperature) = response::parse_field::<f32>(response, "temp", None, Some("'C"))
			.map_err(GencmdCmdError::from_invalid_format)?;

		Ok(temperature)
	}
}

pub struct CmdMeasureClockArm;
impl<'a> Command<'a> for CmdMeasureClockArm {
	type Response = u64;

	const COMMAND_STR: &'static str = "measure_clock arm";

	fn parse_response(response: &'a str) -> Result<Self::Response, GencmdCmdError> {
		let (_, frequency) = response::parse_field_simple::<u64>(response, "frequency(48)")
			.map_err(GencmdCmdError::from_invalid_format)?;

		Ok(frequency)
	}
}

#[derive(Default, Debug, Clone, Copy)]
#[cfg_attr(feature = "serde_models", derive(Serialize, Deserialize))]
pub struct ThrottleStatus {
	pub under_voltage: bool,
	pub frequency_capped: bool,
	pub throttled: bool,
	pub soft_temperature_limit: bool,
}
#[rustfmt::skip] // stop reordering muh constants
impl ThrottleStatus {
	pub const BIT_UNDER_VOLTAGE: u32 = 1;
	pub const BIT_FREQUENCT_CAPPED: u32 = 2;
	pub const BIT_THROTTLED: u32 = 4;
	pub const BIT_SOFT_TEMPERATURE_LIMIT: u32 = 8;
	pub const SHIFT_OCCURED: usize = 16;
}
impl ThrottleStatus {
	pub fn from_current(value: u32) -> Self {
		ThrottleStatus {
			under_voltage: value & Self::BIT_UNDER_VOLTAGE != 0,
			frequency_capped: value & Self::BIT_FREQUENCT_CAPPED != 0,
			throttled: value & Self::BIT_THROTTLED != 0,
			soft_temperature_limit: value & Self::BIT_SOFT_TEMPERATURE_LIMIT != 0,
		}
	}

	pub fn from_occured(value: u32) -> Self {
		Self::from_current(value >> Self::SHIFT_OCCURED)
	}

	pub fn to_current(self) -> u32 {
		(self.under_voltage as u32) * Self::BIT_UNDER_VOLTAGE
			| (self.frequency_capped as u32) * Self::BIT_FREQUENCT_CAPPED
			| (self.throttled as u32) * Self::BIT_THROTTLED
			| (self.soft_temperature_limit as u32) * Self::BIT_SOFT_TEMPERATURE_LIMIT
	}

	pub fn to_occured(self) -> u32 {
		self.to_current() << Self::SHIFT_OCCURED
	}
}

#[derive(Debug, Default)]
#[cfg_attr(feature = "serde_models", derive(Serialize, Deserialize))]
pub struct CpuThrottled {
	pub current: ThrottleStatus,
	pub occured: ThrottleStatus,
}
impl From<u32> for CpuThrottled {
	fn from(value: u32) -> Self {
		CpuThrottled {
			current: ThrottleStatus::from_current(value),
			occured: ThrottleStatus::from_occured(value),
		}
	}
}
impl From<CpuThrottled> for u32 {
	fn from(value: CpuThrottled) -> Self {
		value.current.to_current() | value.occured.to_occured()
	}
}

pub struct CmdGetThrottled;
impl<'a> Command<'a> for CmdGetThrottled {
	type Response = CpuThrottled;

	const COMMAND_STR: &'static str = "get_throttled";

	fn parse_response(response: &'a str) -> Result<Self::Response, GencmdCmdError> {
		let (_, throttled) =
			response::parse_field::<IntRadix<u32, 16>>(response, "throttled", Some("0x"), None)
				.map_err(GencmdCmdError::from_invalid_format)?;

		Ok(CpuThrottled::from(throttled.0))
	}
}
