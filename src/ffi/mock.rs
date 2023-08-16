use std::{ffi::CStr, sync::Mutex};

use super::{VCHI_CONNECTION_T, VCHI_INSTANCE_T, VCOS_STATUS_T};

#[no_mangle]
pub extern "C" fn vcos_init() -> VCOS_STATUS_T {
	log::trace!("vcos_init");

	VCOS_STATUS_T::VCOS_SUCCESS
}

#[no_mangle]
pub extern "C" fn vcos_deinit() {
	log::trace!("vcos_deinit");
}

#[no_mangle]
pub extern "C" fn vchi_initialise(instance_handle: *mut VCHI_INSTANCE_T) -> i32 {
	log::trace!("vchi_initialise");

	unsafe {
		*instance_handle = std::ptr::NonNull::dangling().as_ptr();
	}

	0
}

#[no_mangle]
pub extern "C" fn vchi_connect(
	_connections: *mut *mut VCHI_CONNECTION_T,
	_num_connections: u32,
	_instance_handle: VCHI_INSTANCE_T,
) -> i32 {
	log::trace!("vchi_connect");

	0
}

#[no_mangle]
pub extern "C" fn vchi_disconnect(_instance_handle: VCHI_INSTANCE_T) -> i32 {
	log::trace!("vchi_disconnect");

	0
}

#[no_mangle]
pub extern "C" fn vc_vchi_gencmd_init(
	_initialise_instance: VCHI_INSTANCE_T,
	_connections: *mut *mut VCHI_CONNECTION_T,
	_num_connections: u32,
) {
	log::trace!("vc_vchi_gencmd_init");
}

#[no_mangle]
pub extern "C" fn vc_gencmd_stop() {
	log::trace!("vc_gencmd_stop");
}

const RESPONSE_ERROR_1: &'static [u8] = b"error=1 error_msg=\"command not registered\"\0";
const RESPONSE_ERROR_2: &'static [u8] = b"error=2 error_msg=\"invalid arguments\"\0";
const RESPONSE_COMMANDS: &'static [u8] = b"commands=\"vcos, ap_output_control, ap_output_post_processing, vchi_test_init, vchi_test_exit, pm_set_policy, pm_get_status, pm_show_stats, pm_start_logging, pm_stop_logging, version, commands, set_vll_dir, set_backlight, set_logging, get_lcd_info, arbiter, cache_flush, otp_dump, test_result, codec_enabled, get_camera, get_mem, measure_clock, measure_volts, enable_clock, scaling_kernel, scaling_sharpness, get_hvs_asserts, get_throttled, measure_temp, get_config, hdmi_ntsc_freqs, hdmi_adjust_clock, hdmi_status_show, hvs_update_fields, pwm_speedup, force_audio, hdmi_stream_channels, hdmi_channel_map, display_power, read_ring_osc, memtest, dispmanx_list, get_rsts, schmoo, render_bar, disk_notify, inuse_notify, sus_suspend, sus_status, sus_is_enabled, sus_stop_test_thread, egl_platform_switch, mem_validate, mem_oom, mem_reloc_stats, hdmi_cvt, hdmi_timings, readmr, pmicrd, pmicwr, bootloader_version, bootloader_config, file, vctest_memmap, vctest_start, vctest_stop, vctest_set, vctest_get\"\0";
const RESPONSE_GET_THROTTLED: &'static [u8] = b"throttled=0x0\0";
const RESPONSE_MEASURE_CLOCK_ARM: &'static [u8] = b"frequency(48)=6000000\0";
const RESPONSE_MEASURE_TEMP: &'static [u8] = b"temp=45.6'C\0";
static RESPONSE_LAST_SEND: Mutex<&'static [u8]> = Mutex::new(RESPONSE_ERROR_1);

#[no_mangle]
pub extern "C" fn vc_gencmd_send(
	format: *const ::std::os::raw::c_char,
	// this is probably UB, but implementing variadic functions is unstable :(
	arg1: *const ::std::os::raw::c_char,
) -> ::std::os::raw::c_int {
	log::trace!("vc_gencmd_send({:p}, {:p})", format, arg1);

	let mut lock = RESPONSE_LAST_SEND.lock().expect("mutex poisoned");

	let format = unsafe { CStr::from_ptr(format) };

	// not going to reimplement printf
	if format.to_bytes() != b"%s" {
		log::warn!("Unsupported printf format for mock ffi");
		*lock = RESPONSE_ERROR_1;
		return 0;
	}

	let command = match unsafe { CStr::from_ptr(arg1) }.to_str() {
		Ok(command) => command,
		Err(err) => {
			log::warn!("Unsupported command bytes for mock ffi: {}", err);
			*lock = RESPONSE_ERROR_1;
			return 0;
		}
	};

	log::trace!("vc_gencmd_send command: {}", command);
	*lock = match command {
		"commands" => RESPONSE_COMMANDS,
		"get_throttled" => RESPONSE_GET_THROTTLED,
		"measure_clock" => RESPONSE_ERROR_2,
		"measure_clock arm" => RESPONSE_MEASURE_CLOCK_ARM,
		"measure_temp" => RESPONSE_MEASURE_TEMP,
		_ => RESPONSE_ERROR_1,
	};

	0
}

#[no_mangle]
pub extern "C" fn vc_gencmd_read_response(
	response: *mut ::std::os::raw::c_char,
	maxlen: ::std::os::raw::c_int,
) -> ::std::os::raw::c_int {
	log::trace!("vc_gencmd_read_response");

	let mock_response = *RESPONSE_LAST_SEND.lock().expect("mutex poisoned");

	if maxlen < mock_response.len() as i32 {
		unsafe {
			*response = 0;
		}
		return -1;
	}

	unsafe {
		std::slice::from_raw_parts_mut(response as *mut u8, mock_response.len())
			.copy_from_slice(mock_response);
	}

	0
}
