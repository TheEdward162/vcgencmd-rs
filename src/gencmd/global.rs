use std::{
	ops::DerefMut,
	sync::{Arc, Mutex}
};

use crate::{
	error::{GencmdCmdError, GencmdInitError},
	gencmd::{Command, Gencmd},
	global::GlobalInstance
};

pub struct GencmdGlobal(pub Gencmd, pub Arc<Mutex<GlobalInstance>>);
impl GencmdGlobal {
	pub fn new() -> Result<Self, GencmdInitError> {
		let instance = GlobalInstance::instance()?;

		let gencmd = Gencmd::new();

		Ok(GencmdGlobal(gencmd, instance))
	}

	pub fn send_cmd_raw(&mut self, command: &str) -> Result<&str, GencmdCmdError> {
		let mut lock = self.1.lock().expect("mutex poisoned");

		self.0.send_cmd_raw(lock.deref_mut(), command)
	}

	pub fn send_cmd<'a, C: Command<'a>>(&'a mut self) -> Result<C::Response, GencmdCmdError> {
		let mut lock = self.1.lock().expect("mutex poisoned");

		self.0.send_cmd::<C>(lock.deref_mut())
	}
}
