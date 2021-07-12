use crate::{
	error::{GencmdCmdError, GencmdInitError},
	gencmd::{Command, Gencmd},
	global::GlobalInstance
};

pub struct GencmdUnique(pub Gencmd, pub GlobalInstance);
impl GencmdUnique {
	pub fn new() -> Result<Self, GencmdInitError> {
		let instance = GlobalInstance::new()?;

		let gencmd = Gencmd::new();

		Ok(GencmdUnique(gencmd, instance))
	}

	pub fn send_cmd_raw(&mut self, command: &str) -> Result<&str, GencmdCmdError> {
		self.0.send_cmd_raw(&mut self.1, command)
	}

	pub fn send_cmd<'a, C: Command<'a>>(&'a mut self) -> Result<C::Response, GencmdCmdError> {
		self.0.send_cmd::<C>(&mut self.1)
	}
}
