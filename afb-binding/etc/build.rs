/*
 * Copyright (C) 2015-2023 IoT.bzh Company
 * Author: Fulup Ar Foll <fulup@iot.bzh>
 *
 * Redpesk interface code/config use MIT License and can be freely copy/modified even within proprietary code
 * License: $RP_BEGIN_LICENSE$ SPDX:MIT https://opensource.org/licenses/MIT $RP_END_LICENSE$
 *
*/

fn main() {
    // check pkgconfig dependencies
    #[cfg(not(feature="rpm_build"))]
    system_deps::Config::new().probe().unwrap();

    println!("cargo:rustc-link-search=/usr/local/lib64");
    println!("cargo:rustc-link-arg=-lpcscd-glue");
    println!("cargo:rustc-link-arg=-lpcsclite");
}
