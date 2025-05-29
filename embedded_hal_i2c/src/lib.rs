// Copyright (c) 2024 Linaro LTD
// SPDX-License-Identifier: Apache-2.0

#![no_std]
// Sigh. The check config system requires that the compiler be told what possible config values
// there might be.  This is completely impossible with both Kconfig and the DT configs, since the
// whole point is that we likely need to check for configs that aren't otherwise present in the
// build.  So, this is just always necessary.
#![allow(unexpected_cfgs)]

extern crate alloc;
use alloc::vec::Vec;

use embedded_hal::i2c::I2c;
use log::{info, warn};

use lsm6ds3tr::LSM6DS3TR;
use zephyr::raw::GPIO_OUTPUT_ACTIVE;
use zephyr::time::{sleep, Duration};
use zephyr_sys::i2c_is_ready_dt;

extern "C" {
    fn devicetree_get_i2c_lsm6ds3tr_c() -> *const zephyr_sys::i2c_dt_spec;
    fn zephyr_i2c_read_dt(spec: *const zephyr_sys::i2c_dt_spec, buf: *mut u8, num_bytes: u32) -> i32;
    fn zephyr_i2c_transfer_dt(spec: *const zephyr_sys::i2c_dt_spec, msgs: *mut zephyr_sys::i2c_msg, num_msgs: u8, addr: u16) -> i32;
}

struct I2cDevice {
    spec: *const zephyr_sys::i2c_dt_spec,
}
impl I2cDevice {
    pub fn new(spec: *const zephyr_sys::i2c_dt_spec) -> Self {
        I2cDevice { spec }
    }
    pub fn is_ready(&self) -> bool {
        unsafe { i2c_is_ready_dt(self.spec) }
    }
}
impl embedded_hal::i2c::ErrorType for I2cDevice {
    type Error = embedded_hal::i2c::ErrorKind;
}
impl embedded_hal::i2c::I2c for I2cDevice {
    fn transaction(
            &mut self,
            address: u8,
            operations: &mut [embedded_hal::i2c::Operation<'_>],
        ) -> Result<(), Self::Error> {
        let num_msgs = operations.len();
        let mut msgs = Vec::with_capacity(num_msgs);
        let mut is_last_read = None;
        for (i, op) in operations.iter_mut().enumerate() {
            let restart = if let Some(last_read) = is_last_read {
                let is_read = matches!(op, embedded_hal::i2c::Operation::Read(_));
                is_read != last_read
            } else { 
                false
            };
            let flags = if restart {
                1 << 2  // I2C_MSG_RESTART
            } else {
                0
            };
            let flags = if i == num_msgs - 1 {
                flags | (1 << 1) // I2C_MSG_STOP
            } else {
                flags
            };

            match op {
                embedded_hal::i2c::Operation::Read(buf) => {
                    msgs.push(zephyr_sys::i2c_msg {
                        flags: flags | (1 << 0), // I2C_MSG_READ
                        len: buf.len() as u32,
                        buf: buf.as_mut_ptr(),
                    });
                    is_last_read = Some(true);
                }
                embedded_hal::i2c::Operation::Write(buf) => {
                    msgs.push(zephyr_sys::i2c_msg {
                        flags: flags | (0), // I2C_MSG_WRITE
                        len: buf.len() as u32,
                        buf: buf.as_ptr() as *mut u8,
                    });
                    is_last_read = Some(false);
                }
            }
        }
        let result = unsafe { zephyr_i2c_transfer_dt(self.spec, msgs.as_mut_ptr(), msgs.len() as u8, address as u16) };
        if result < 0 {
            warn!("I2C transfer failed with error code: {}", result);
            Err(embedded_hal::i2c::ErrorKind::Other)
        } else {
            Ok(())
        }
    }
}

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

    let i2c_lsm = unsafe { devicetree_get_i2c_lsm6ds3tr_c()};
    if unsafe { i2c_is_ready_dt(i2c_lsm) } {
        info!("I2C device is ready");

        let mut i2c_device = I2cDevice::new(i2c_lsm);
        let mut buf = [0u8; 1];
        match i2c_device.transaction(unsafe { (*i2c_lsm).addr } as u8, &mut [
            embedded_hal::i2c::Operation::Write(&[0x0f]), // WHO_AM_I register
            embedded_hal::i2c::Operation::Read(&mut buf),
        ]) {
            Ok(_) => {
                info!("I2C transaction successful, data: {:02X?}", buf);

                let mut imu = LSM6DS3TR::new(lsm6ds3tr::interface::I2cInterface::new(i2c_device))
                    .with_settings(lsm6ds3tr::LsmSettings::basic());
                imu.init().expect("Failed to initialize IMU");
                let duration = Duration::millis_at_least(1000);

                loop {
                    if let (Ok(xyz_a), Ok(xyz_g)) = (imu.read_accel(), imu.read_gyro()) {
                        info!("Accelerometer data: {}", xyz_a);
                        info!("Gyroscope data: {}", xyz_g);
                        sleep(duration);

                    } else {
                        warn!("Failed to read accelerometer or gyroscope data");
                        break;
                    }
                }
            }
            Err(e) => {
                warn!("I2C transaction failed: {:?}", e);
            }
        }
    } else {
        warn!("I2C device is not ready");
    }
}
