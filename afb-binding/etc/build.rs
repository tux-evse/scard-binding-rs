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
    #[cfg(not(feature = "rpm_build"))]
    system_deps::Config::new().probe().unwrap();

    println!("cargo:rustc-link-search=/usr/local/lib64");
    if let Ok(value) = env::var("CARGO_TARGET_DIR") {
        if let Ok(profile) = env::var("PROFILE") {
            println!("cargo:rustc-link-search=crate={}{}", value, profile);
        }
    }
    println!("cargo:rustc-link-arg=-lpcscd-glue");
    println!("cargo:rustc-link-arg=-lpcsclite");
}
