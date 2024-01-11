/*
 * Copyright (C) 2015-2023 IoT.bzh Company
 * Author: Fulup Ar Foll <fulup@iot.bzh>
 *
 * Redpesk interface code/config use MIT License and can be freely copy/modified even within proprietary code
 * License: $RP_BEGIN_LICENSE$ SPDX:MIT https://opensource.org/licenses/MIT $RP_END_LICENSE$
 *
*/
use std::env;

fn main() {
    // check pkgconfig dependencies
    #[cfg(not(feature="rpm_build"))]
    system_deps::Config::new().probe().unwrap();

    // invalidate the built crate whenever the wrapper changes
    println!("cargo:rerun-if-changed=capi/capi-map.h");
    println!("cargo:rustc-link-search=/usr/local/lib64");
    if let Ok(value) = env::var("CARGO_TARGET_DIR") {
        if let Ok(profile) = env::var("PROFILE") {
            println!("cargo:rustc-link-search=crate={}{}", value, profile);
        }
    }
    let header = "
    // -----------------------------------------------------------------------
    //         <- private '_capi_map.rs' Rust/C unsafe binding ->
    // -----------------------------------------------------------------------
    //   Do not exit this file it will be regenerated automatically by cargo.
    //   Check:
    //     - build.rs for C/Rust glue options
    //     - src/capi/capi-map.h for C prototype inputs
    // -----------------------------------------------------------------------
    ";

    let _capi_map
     = bindgen::Builder::default()
        .header("capi/capi-map.h")
        .clang_arg("-I/usr/include/PCSC")
        .raw_line(header)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .derive_debug(false)
        .layout_tests(false)
        .allowlist_function("pcsc.*")
        .allowlist_var("pcsc.*")
        .allowlist_var("PCSC_.*")
        .blocklist_item("UT_.*")
        .blocklist_item("pcscCmdT")
        .blocklist_item("json_object")
        .generate()
        .expect("Unable to generate _capi-map.rs");

    _capi_map
        .write_to_file("capi/_capi-map.rs")
        .expect("Couldn't write _capi-map.rs!");
}
