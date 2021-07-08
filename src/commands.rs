use super::{response, Gencmd, GencmdCmdError};

#[derive(Default, Debug)]
pub struct ThrottleStatus {
	pub under_voltage: bool,
	pub frequency_capped: bool,
	pub throttled: bool,
	pub soft_temperature_limit: bool
}
impl ThrottleStatus {
	pub const BIT_FREQUENCT_CAPPED: u32 = 2;
	pub const BIT_SOFT_TEMPERATURE_LIMIT: u32 = 8;
	pub const BIT_THROTTLED: u32 = 4;
	pub const BIT_UNDER_VOLTAGE: u32 = 1;
	pub const SHIFT_OCCURED: usize = 16;

	pub fn from_current(value: u32) -> Self {
		ThrottleStatus {
			under_voltage: value & Self::BIT_UNDER_VOLTAGE != 0,
			frequency_capped: value & Self::BIT_FREQUENCT_CAPPED != 0,
			throttled: value & Self::BIT_THROTTLED != 0,
			soft_temperature_limit: value & Self::BIT_SOFT_TEMPERATURE_LIMIT != 0
		}
	}

	pub fn from_occured(value: u32) -> Self {
		Self::from_current(value >> Self::SHIFT_OCCURED)
	}
}

#[derive(Debug, Default)]
pub struct CpuThrottled {
	pub current: ThrottleStatus,
	pub occured: ThrottleStatus
}
impl From<u32> for CpuThrottled {
	fn from(value: u32) -> Self {
		CpuThrottled {
			current: ThrottleStatus::from_current(value),
			occured: ThrottleStatus::from_occured(value)
		}
	}
}

impl Gencmd {
	pub fn cmd_commands(&mut self) -> Result<Vec<&str>, GencmdCmdError> {
		let response = self.cmd_send("commands")?;

		let (_, commands) = response::parse_field_simple::<&str>(response, "commands")
			.map_err(GencmdCmdError::from_invalid_format)?;

		Ok(commands.split(", ").collect())
	}

	pub fn cmd_measure_temp(&mut self) -> Result<f32, GencmdCmdError> {
		let response = self.cmd_send("measure_temp")?;

		let (_, temperature) = response::parse_field::<f32>(response, "temp", None, Some("'C"))
			.map_err(GencmdCmdError::from_invalid_format)?;

		Ok(temperature)
	}

	pub fn cmd_measure_clock_arm(&mut self) -> Result<u64, GencmdCmdError> {
		let response = self.cmd_send("measure_clock arm")?;

		let (_, frequency) = response::parse_field_simple::<u64>(response, "frequency(48)")
			.map_err(GencmdCmdError::from_invalid_format)?;

		Ok(frequency)
	}

	pub fn cmd_get_throttled(&mut self) -> Result<CpuThrottled, GencmdCmdError> {
		let response = self.cmd_send("get_throttled")?;

		let (_, throttled) = response::parse_field::<u32>(response, "throttled", Some("0x"), None)
			.map_err(GencmdCmdError::from_invalid_format)?;

		Ok(CpuThrottled::from(throttled))
	}
}

#[cfg(test)]
mod test {
	use super::Gencmd;

	#[test]
	fn test_cmd_commands() {
		let mut gencmd = Gencmd::new().unwrap();

		let commands = dbg!(gencmd.cmd_commands()).unwrap();

		assert!(commands.contains(&"commands"));
		assert!(commands.contains(&"measure_clock"));
		assert!(commands.contains(&"measure_temp"));
		assert!(commands.contains(&"get_throttled"));
	}

	#[test]
	fn test_cmd_measure_temp() {
		let mut gencmd = Gencmd::new().unwrap();

		let temp = dbg!(gencmd.cmd_measure_temp()).unwrap();

		assert!(temp > 0.0);
	}

	#[test]
	fn test_cmd_measure_clock_arm() {
		let mut gencmd = Gencmd::new().unwrap();

		let freq = dbg!(gencmd.cmd_measure_clock_arm()).unwrap();

		assert!(freq > 0);
	}

	#[test]
	fn test_cmd_get_throttled() {
		let mut gencmd = Gencmd::new().unwrap();

		dbg!(gencmd.cmd_get_throttled()).unwrap();
	}
}
