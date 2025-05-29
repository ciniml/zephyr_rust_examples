// Copyright (c) 2024 Linaro LTD
// SPDX-License-Identifier: Apache-2.0

#![no_std]
// Sigh. The check config system requires that the compiler be told what possible config values
// there might be.  This is completely impossible with both Kconfig and the DT configs, since the
// whole point is that we likely need to check for configs that aren't otherwise present in the
// build.  So, this is just always necessary.
#![allow(unexpected_cfgs)]

use log::{info, warn};

use zephyr::time::{sleep, Duration};

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[no_mangle]
extern "C" fn rust_main() {
    unsafe {
        zephyr::set_logger().unwrap();
    }

    zephyr::printkln!("[printk] Hello world from Rust on {}", zephyr::kconfig::CONFIG_BOARD);
    info!("40 + 2 = {}", unsafe { add_two_numbers(40, 2) });
}
