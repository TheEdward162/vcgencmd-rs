pub use crate::{
	error::*,
	gencmd::{commands::*, unique::GencmdUnique, Gencmd},
	global::GlobalInstance,
};

#[cfg(feature = "global_singleton")]
pub use crate::gencmd::global::GencmdGlobal;
