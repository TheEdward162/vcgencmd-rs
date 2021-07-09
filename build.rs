fn main() {
	#[cfg(not(feature = "mock_vc_ffi"))]
	{
		println!("cargo:rustc-link-lib=vchiq_arm");
		println!("cargo:rustc-link-lib=vcos");
		println!("cargo:rustc-link-lib=bcm_host");
		println!("cargo:rustc-link-search=/opt/vc/lib");
	}

	#[cfg(feature = "run_bindgen")]
	run_bindgen()
}

#[cfg(feature = "run_bindgen")]
fn run_bindgen() {
	use std::{env, path::PathBuf};

	const INCLUDE_HEADERS: &'static [&'static str] = &[
		"raspberrypi-userland/interface/vmcs_host/vc_vchi_gencmd.h",
		"raspberrypi-userland/interface/vmcs_host/vc_gencmd_defs.h"
	];

	// an absolute hack of including c files as headers
	const INCLUDE_FILES: &'static [&'static str] =
		&["raspberrypi-userland/interface/vmcs_host/vc_vchi_gencmd.c"];

	let include_files_contents = [std::fs::read_to_string(INCLUDE_FILES[0]).unwrap()];

	println!("cargo:rerun-if-changed={}", INCLUDE_HEADERS[0]);
	println!("cargo:rerun-if-changed={}", INCLUDE_HEADERS[1]);
	println!("cargo:rerun-if-changed={}", INCLUDE_FILES[0]);

	let bindings = bindgen::Builder::default()
		// because otherwise there are some definition errors on non-linux
		.clang_arg("-Wno-everything")
		// the root directory of our git clone
		.clang_arg("-Iraspberrypi-userland/")
		// because it doesn't build even for macos, even though we only want the bindings
		.clang_arg("-D__unix__")
		.allowlist_recursively(false)
		// need these headers included from gencmd.c
		.header(INCLUDE_HEADERS[0])
		.header(INCLUDE_HEADERS[1])
		.header_contents(INCLUDE_FILES[0], &include_files_contents[0])
		// need these functions to reimplement gencmd.c
		.allowlist_function("vcos_init")
		.allowlist_function("vcos_deinit")
		.allowlist_function("vchi_initialise")
		.allowlist_function("vchi_connect")
		.allowlist_function("vc_vchi_gencmd_init")
		.allowlist_function("vc_gencmd_send")
		.allowlist_function("vc_gencmd_read_response")
		.allowlist_function("vc_gencmd_stop")
		.allowlist_function("vchi_disconnect")
		// types used by these functions
		.allowlist_type("VCOS_STATUS_T")
		.newtype_enum("VCOS_STATUS_T")
		.allowlist_type("VCHI_INSTANCE_T")
		.allowlist_type("VCHI_CONNECTION_T")
		.allowlist_type("size_t")
		// opaque types
		.allowlist_type("vchi_connection_t")
		.opaque_type("vchi_connection_t")
		.allowlist_type("opaque_vchi_instance_handle_t")
		.opaque_type("opaque_vchi_instance_handle_t")
		// other values
		.allowlist_var("GENCMDSERVICE_MSGFIFO_SIZE")
		.allowlist_var("GENCMD_MAX_LENGTH")
		// and generate
		.generate().expect("Could not generate bindings");

	let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
	bindings
		.write_to_file(out_path.join("bindings.rs"))
		.expect("Could not write generated bindings to file");
}
