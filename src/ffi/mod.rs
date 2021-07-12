#![allow(non_camel_case_types)]

#[cfg(feature = "mock_vc_ffi")]
pub mod mock;

#[cfg(feature = "run_bindgen")]
mod inner {
	include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

#[cfg(not(feature = "run_bindgen"))]
mod inner {
	// automatically generated by rust-bindgen 0.58.1

	pub const GENCMDSERVICE_MSGFIFO_SIZE: u32 = 4092;
	pub const GENCMD_MAX_LENGTH: u32 = 512;
	pub type size_t = ::std::os::raw::c_ulong;
	impl VCOS_STATUS_T {
		pub const VCOS_SUCCESS: VCOS_STATUS_T = VCOS_STATUS_T(0);
	}
	impl VCOS_STATUS_T {
		pub const VCOS_EAGAIN: VCOS_STATUS_T = VCOS_STATUS_T(1);
	}
	impl VCOS_STATUS_T {
		pub const VCOS_ENOENT: VCOS_STATUS_T = VCOS_STATUS_T(2);
	}
	impl VCOS_STATUS_T {
		pub const VCOS_ENOSPC: VCOS_STATUS_T = VCOS_STATUS_T(3);
	}
	impl VCOS_STATUS_T {
		pub const VCOS_EINVAL: VCOS_STATUS_T = VCOS_STATUS_T(4);
	}
	impl VCOS_STATUS_T {
		pub const VCOS_EACCESS: VCOS_STATUS_T = VCOS_STATUS_T(5);
	}
	impl VCOS_STATUS_T {
		pub const VCOS_ENOMEM: VCOS_STATUS_T = VCOS_STATUS_T(6);
	}
	impl VCOS_STATUS_T {
		pub const VCOS_ENOSYS: VCOS_STATUS_T = VCOS_STATUS_T(7);
	}
	impl VCOS_STATUS_T {
		pub const VCOS_EEXIST: VCOS_STATUS_T = VCOS_STATUS_T(8);
	}
	impl VCOS_STATUS_T {
		pub const VCOS_ENXIO: VCOS_STATUS_T = VCOS_STATUS_T(9);
	}
	impl VCOS_STATUS_T {
		pub const VCOS_EINTR: VCOS_STATUS_T = VCOS_STATUS_T(10);
	}
	#[repr(transparent)]
	#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
	pub struct VCOS_STATUS_T(pub ::std::os::raw::c_uint);
	extern "C" {
		/// vcos initialization. Call this function before using other vcos functions.
		/// Calls can be nested within the same process; they are reference counted so
		/// that only a call from uninitialized state has any effect.
		/// @note On platforms/toolchains that support it, gcc's constructor attribute or
		///       similar is used to invoke this function before main() or equivalent.
		/// @return Status of initialisation.
		pub fn vcos_init() -> VCOS_STATUS_T;
	}
	extern "C" {
		/// vcos deinitialization. Call this function when vcos is no longer required,
		/// in order to free resources.
		/// Calls can be nested within the same process; they are reference counted so
		/// that only a call that decrements the reference count to 0 has any effect.
		/// @note On platforms/toolchains that support it, gcc's destructor attribute or
		///       similar is used to invoke this function after exit() or equivalent.
		/// @return Status of initialisation.
		pub fn vcos_deinit();
	}
	pub type VCHI_CONNECTION_T = vchi_connection_t;
	#[repr(C)]
	#[repr(align(8))]
	#[derive(Debug, Copy, Clone)]
	pub struct vchi_connection_t {
		pub _bindgen_opaque_blob: [u64; 3usize]
	}
	#[test]
	fn bindgen_test_layout_vchi_connection_t() {
		assert_eq!(
			::std::mem::size_of::<vchi_connection_t>(),
			24usize,
			concat!("Size of: ", stringify!(vchi_connection_t))
		);
		assert_eq!(
			::std::mem::align_of::<vchi_connection_t>(),
			8usize,
			concat!("Alignment of ", stringify!(vchi_connection_t))
		);
	}
	#[repr(C)]
	#[derive(Debug, Copy, Clone)]
	pub struct opaque_vchi_instance_handle_t {
		_unused: [u8; 0]
	}
	pub type VCHI_INSTANCE_T = *mut opaque_vchi_instance_handle_t;
	extern "C" {
		pub fn vchi_initialise(instance_handle: *mut VCHI_INSTANCE_T) -> i32;
	}
	extern "C" {
		pub fn vchi_connect(
			connections: *mut *mut VCHI_CONNECTION_T,
			num_connections: u32,
			instance_handle: VCHI_INSTANCE_T
		) -> i32;
	}
	extern "C" {
		pub fn vchi_disconnect(instance_handle: VCHI_INSTANCE_T) -> i32;
	}
	extern "C" {
		/// NAME
		/// vc_vchi_gencmd_init
		///
		/// SYNOPSIS
		/// void vc_vchi_gencmd_init(VCHI_INSTANCE_T initialise_instance, VCHI_CONNECTION_T **connections, uint32_t num_connections )
		///
		/// FUNCTION
		/// Initialise the general command service for use. A negative return value
		/// indicates failure (which may mean it has not been started on VideoCore).
		///
		/// RETURNS
		/// int
		pub fn vc_vchi_gencmd_init(
			initialise_instance: VCHI_INSTANCE_T,
			connections: *mut *mut VCHI_CONNECTION_T,
			num_connections: u32
		);
	}
	extern "C" {
		/// NAME
		/// vc_gencmd_stop
		///
		/// SYNOPSIS
		/// int vc_gencmd_stop()
		///
		/// FUNCTION
		/// This tells us that the generak command service has stopped, thereby preventing
		/// any of the functions from doing anything.
		///
		/// RETURNS
		/// int
		pub fn vc_gencmd_stop();
	}
	extern "C" {
		/// Send commands to VideoCore.
		/// These all return 0 for success. They return VC_MSGFIFO_FIFO_FULL if there is
		/// insufficient space for the whole message in the fifo, and none of the message is
		/// sent.
		pub fn vc_gencmd_send(format: *const ::std::os::raw::c_char, ...) -> ::std::os::raw::c_int;
	}
	extern "C" {
		/// NAME
		/// vc_gencmd_read_response
		///
		/// SYNOPSIS
		/// int vc_gencmd_read_response
		///
		/// FUNCTION
		/// Block until something comes back
		///
		/// RETURNS
		/// Error code from dequeue message
		pub fn vc_gencmd_read_response(
			response: *mut ::std::os::raw::c_char,
			maxlen: ::std::os::raw::c_int
		) -> ::std::os::raw::c_int;
	}
}

pub use inner::*;