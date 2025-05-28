// Copyright (c) 2024 Linaro LTD
// SPDX-License-Identifier: Apache-2.0

#![no_std]
// Sigh. The check config system requires that the compiler be told what possible config values
// there might be.  This is completely impossible with both Kconfig and the DT configs, since the
// whole point is that we likely need to check for configs that aren't otherwise present in the
// build.  So, this is just always necessary.
#![allow(unexpected_cfgs)]

use log::{info, warn};

use zephyr::raw::GPIO_OUTPUT_ACTIVE;
use zephyr::time::{sleep, Duration};

#[no_mangle]
extern "C" fn rust_main() {
    unsafe {
        zephyr::set_logger().unwrap();
    }

    zephyr::printkln!("[printk] Hello world from Rust on {}", zephyr::kconfig::CONFIG_BOARD);
    info!("Hello world from Rust on {}", zephyr::kconfig::CONFIG_BOARD);

    warn!("Starting blinky");

    // let result = unsafe { c_function_42() };
    // info!("Result from C function: {}", result);

    do_blink();
}

// extern "C" {
//     fn c_function_42() -> u32;
// }

fn do_blink() {
    warn!("Inside of blinky");

    let mut led0 = zephyr::devicetree::aliases::led0::get_instance().unwrap();
    let mut led1 = zephyr::devicetree::aliases::led1::get_instance().unwrap();
    let mut gpio_token = unsafe { zephyr::device::gpio::GpioToken::get_instance().unwrap() };

    if !led0.is_ready() {
        warn!("LED is not ready");
        loop {}
    }
    if !led1.is_ready() {
        warn!("LED1 is not ready");
        loop {}
    }

    unsafe {
        led0.configure(&mut gpio_token, GPIO_OUTPUT_ACTIVE);
        led1.configure(&mut gpio_token, GPIO_OUTPUT_ACTIVE);
    }
    let duration = Duration::millis_at_least(500);
    loop {
        unsafe {
            led0.toggle_pin(&mut gpio_token);
            led1.toggle_pin(&mut gpio_token);
        }
        sleep(duration);
    }
}