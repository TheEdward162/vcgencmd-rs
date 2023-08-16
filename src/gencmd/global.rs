use std::{
	borrow::BorrowMut,
	ops::DerefMut,
	sync::{Arc, Mutex},
};

use crate::{
	error::{GencmdCmdError, GencmdInitError},
	gencmd::{Command, Gencmd},
	global::GlobalInstance,
};

pub struct GencmdGlobal<
	G: BorrowMut<Gencmd> = Gencmd,
	I: BorrowMut<GlobalInstance> = GlobalInstance,
>(pub G, pub Arc<Mutex<I>>);
impl GencmdGlobal {
	pub fn new() -> Result<Self, GencmdInitError> {
		let instance = GlobalInstance::instance::<128>()?;

		let gencmd = Gencmd::new();

		Ok(GencmdGlobal(gencmd, instance))
	}
}
impl<G: BorrowMut<Gencmd>, I: BorrowMut<GlobalInstance>> GencmdGlobal<G, I> {
	pub fn send_cmd_raw(&mut self, command: &str) -> Result<&str, GencmdCmdError> {
		let mut lock = self.1.lock().expect("mutex poisoned");

		self.0
			.borrow_mut()
			.send_cmd_raw(lock.deref_mut().borrow_mut(), command)
	}

	pub fn send_cmd<'a, C: Command<'a>>(&'a mut self) -> Result<C::Response, GencmdCmdError> {
		let mut lock = self.1.lock().expect("mutex poisoned");

		self.0
			.borrow_mut()
			.send_cmd::<C>(lock.deref_mut().borrow_mut())
	}
}

#[cfg(test)]
mod test {
	use crate::gencmd::global::GencmdGlobal;

	use crate::gencmd::commands::{
		CmdCommands, CmdGetThrottled, CmdMeasureClockArm, CmdMeasureTemp,
	};

	#[test]
	fn test_cmd_commands() {
		crate::test::setup_global();

		let mut gencmd = GencmdGlobal::new().unwrap();

		let commands = dbg!(gencmd.send_cmd::<CmdCommands>()).unwrap();

		assert!(commands.contains(&"commands"));
		assert!(commands.contains(&"measure_clock"));
		assert!(commands.contains(&"measure_temp"));
		assert!(commands.contains(&"get_throttled"));
	}

	#[test]
	fn test_cmd_measure_temp() {
		crate::test::setup_global();

		let mut gencmd = GencmdGlobal::new().unwrap();

		let temp = dbg!(gencmd.send_cmd::<CmdMeasureTemp>()).unwrap();

		assert!(temp > 0.0);
	}

	#[test]
	fn test_cmd_measure_clock_arm() {
		crate::test::setup_global();

		let mut gencmd = GencmdGlobal::new().unwrap();

		let freq = dbg!(gencmd.send_cmd::<CmdMeasureClockArm>()).unwrap();

		assert!(freq > 0);
	}

	#[test]
	fn test_cmd_get_throttled() {
		crate::test::setup_global();

		let mut gencmd = GencmdGlobal::new().unwrap();

		dbg!(gencmd.send_cmd::<CmdGetThrottled>()).unwrap();
	}

	#[test]
	fn test_cmds_threads_racing() {
		crate::test::setup_global();

		let threads: Vec<_> = (0..30)
			.map(|i| match i % 3 {
				0 => std::thread::spawn(|| {
					let mut gencmd = GencmdGlobal::new().unwrap();

					gencmd.send_cmd::<CmdGetThrottled>().unwrap();
				}),
				1 => std::thread::spawn(|| {
					let mut gencmd = GencmdGlobal::new().unwrap();

					gencmd.send_cmd::<CmdMeasureClockArm>().unwrap();
				}),
				2 => std::thread::spawn(|| {
					let mut gencmd = GencmdGlobal::new().unwrap();

					gencmd.send_cmd::<CmdMeasureTemp>().unwrap();
				}),
				_ => unreachable!(),
			})
			.collect();

		for thread in threads {
			thread.join().unwrap();
		}
	}
}
