use super::ffi::{VCHI_CONNECTION_T, VCHI_INSTANCE_T, VCOS_STATUS_T};

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
	_instance_handle: VCHI_INSTANCE_T
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
	_num_connections: u32
) {
	log::trace!("vc_vchi_gencmd_init");
}

#[no_mangle]
pub extern "C" fn vc_gencmd_stop() {
	log::trace!("vc_gencmd_stop");
}


#[no_mangle]
pub extern "C" fn vc_gencmd_send(_format: *const ::std::os::raw::c_char) -> ::std::os::raw::c_int {
	log::trace!("vc_gencmd_send");

	0
}

#[no_mangle]
pub extern "C" fn vc_gencmd_read_response(
	response: *mut ::std::os::raw::c_char,
	maxlen: ::std::os::raw::c_int
) -> ::std::os::raw::c_int {
	// TODO: Mock some actual responses here - that will need shared state with `vc_gencmd_send`

	// const RESPONSE: &'static [u8] = b"commands=\"vcos, ap_output_control, ap_output_post_processing, vchi_test_init, vchi_test_exit, pm_set_policy, pm_get_status, pm_show_stats, pm_start_logging, pm_stop_logging, version, commands, set_vll_dir, set_backlight, set_logging, get_lcd_info, arbiter, cache_flush, otp_dump, test_result, codec_enabled, get_camera, get_mem, measure_clock, measure_volts, enable_clock, scaling_kernel, scaling_sharpness, get_hvs_asserts, get_throttled, measure_temp, get_config, hdmi_ntsc_freqs, hdmi_adjust_clock, hdmi_status_show, hvs_update_fields, pwm_speedup, force_audio, hdmi_stream_channels, hdmi_channel_map, display_power, read_ring_osc, memtest, dispmanx_list, get_rsts, schmoo, render_bar, disk_notify, inuse_notify, sus_suspend, sus_status, sus_is_enabled, sus_stop_test_thread, egl_platform_switch, mem_validate, mem_oom, mem_reloc_stats, hdmi_cvt, hdmi_timings, readmr, pmicrd, pmicwr, bootloader_version, bootloader_config, file, vctest_memmap, vctest_start, vctest_stop, vctest_set, vctest_get\"\0";
	// const RESPONSE: &'static [u8] = b"throttled=0x0\0";
	const RESPONSE: &'static [u8] = b"error=1 error_msg=\"hello from this error\"\0";

	log::trace!("vc_gencmd_read_response");

	if maxlen < RESPONSE.len() as i32 {
		unsafe {
			*response = 0;
		}
	} else {
		unsafe {
			std::slice::from_raw_parts_mut(response as *mut u8, RESPONSE.len())
				.copy_from_slice(RESPONSE);
		}
	}

	0
}
