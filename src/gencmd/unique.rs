use std::borrow::BorrowMut;

use crate::{
	error::{GencmdCmdError, GencmdInitError},
	gencmd::{Command, Gencmd},
	global::GlobalInstance,
};

pub struct GencmdUnique<
	G: BorrowMut<Gencmd> = Gencmd,
	I: BorrowMut<GlobalInstance> = GlobalInstance,
>(pub G, pub I);
impl GencmdUnique {
	pub fn new() -> Result<Self, GencmdInitError> {
		let instance = GlobalInstance::new()?;

		let gencmd = Gencmd::new();

		Ok(GencmdUnique(gencmd, instance))
	}
}
impl<G: BorrowMut<Gencmd>, I: BorrowMut<GlobalInstance>> GencmdUnique<G, I> {
	pub fn send_cmd_raw(&mut self, command: &str) -> Result<&str, GencmdCmdError> {
		self.0
			.borrow_mut()
			.send_cmd_raw(self.1.borrow_mut(), command)
	}

	pub fn send_cmd<'a, C: Command<'a>>(&'a mut self) -> Result<C::Response, GencmdCmdError> {
		self.0.borrow_mut().send_cmd::<C>(self.1.borrow_mut())
	}
}
