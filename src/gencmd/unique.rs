use std::borrow::BorrowMut;

use crate::{
	error::{GencmdCmdError, GencmdInitError},
	gencmd::{Command, Gencmd},
	global::GlobalInstance
};

pub struct GencmdUnique<I: BorrowMut<GlobalInstance> = GlobalInstance>(pub Gencmd, pub I);
impl GencmdUnique<GlobalInstance> {
	pub fn new() -> Result<Self, GencmdInitError> {
		let instance = GlobalInstance::new()?;

		let gencmd = Gencmd::new();

		Ok(GencmdUnique(gencmd, instance))
	}
}
impl<I: BorrowMut<GlobalInstance>> GencmdUnique<I> {
	pub fn send_cmd_raw(&mut self, command: &str) -> Result<&str, GencmdCmdError> {
		self.0.send_cmd_raw(self.1.borrow_mut(), command)
	}

	pub fn send_cmd<'a, C: Command<'a>>(&'a mut self) -> Result<C::Response, GencmdCmdError> {
		self.0.send_cmd::<C>(self.1.borrow_mut())
	}
}
