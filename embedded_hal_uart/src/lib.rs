// Copyright (c) 2024 Linaro LTD
// SPDX-License-Identifier: Apache-2.0

#![no_std]
// Sigh. The check config system requires that the compiler be told what possible config values
// there might be.  This is completely impossible with both Kconfig and the DT configs, since the
// whole point is that we likely need to check for configs that aren't otherwise present in the
// build.  So, this is just always necessary.
#![allow(unexpected_cfgs)]

use embedded_io::{Read, Write};
use log::{debug, info, warn};

use zephyr::time::{sleep, Duration};
use zephyr_sys::{ENOSYS, ENOTSUP};
use zephyr::Error;

extern "C" {
    fn devicetree_get_uart0() -> *const zephyr_sys::device;
    fn zephyr_uart_callback_set(dev: *const zephyr_sys::device, callback: unsafe extern "C" fn(dev: *const zephyr_sys::device, evt: *mut zephyr_sys::uart_event, user_data: *mut core::ffi::c_void), user_data: *mut core::ffi::c_void) -> i32;
}

unsafe extern "C" fn uart_cb(_dev: *const zephyr_sys::device, evt: *mut zephyr_sys::uart_event, user_data: *mut core::ffi::c_void) {
    if user_data.is_null() || evt.is_null() {
        return;
    }

    let sender = unsafe { &mut *(user_data as *mut zephyr::sync::channel::Sender<zephyr_sys::uart_event>) };
    let evt = unsafe { &*evt };
    let evt = zephyr_sys::uart_event {
        type_: evt.type_,
        data: zephyr_sys::uart_event_uart_event_data {
            tx: evt.data.tx,
            rx: evt.data.rx,
            rx_buf: evt.data.rx_buf,
            rx_stop: evt.data.rx_stop,
            bindgen_union_field: evt.data.bindgen_union_field,
        },
    };
    sender.send(evt).ok();
}

struct UartDevice {
    device: *const zephyr_sys::device,
    sender: zephyr::sync::channel::Sender<zephyr_sys::uart_event>,
    receiver: zephyr::sync::channel::Receiver<zephyr_sys::uart_event>,
}
impl UartDevice {
    pub fn new(device: *const zephyr_sys::device) -> Self {
        let (sender, receiver) = zephyr::sync::channel::bounded(16);
        Self { 
            device ,
            sender,
            receiver,
        }
    }

    pub fn configure(&mut self, baudrate: u32) -> Result<(), ZephyrDeviceError> {
        unsafe {
            let mut config = zephyr_sys::uart_config {
                baudrate: 0,
                parity: 0,
                stop_bits: 0,
                flow_ctrl: 0,
                data_bits: 0,
            };
            check_error(zephyr_sys::uart_config_get(self.device, &mut config))?;
            config.baudrate = baudrate;
            check_error(zephyr_sys::uart_configure(self.device, &config))?;
        }
        Ok(())
    }
}

#[derive(Debug)]
struct ZephyrDeviceError {
    pub inner: zephyr::Error,
}
impl From<zephyr::Error> for ZephyrDeviceError {
    fn from(inner: zephyr::Error) -> Self {
        ZephyrDeviceError { inner }
    }
}
impl embedded_io::Error for ZephyrDeviceError {
    fn kind(&self) -> embedded_io::ErrorKind {
        match self.inner.0 {
            zephyr_sys::ENOSYS => embedded_io::ErrorKind::Unsupported,
            zephyr_sys::ENOTSUP => embedded_io::ErrorKind::NotFound,
            _ => embedded_io::ErrorKind::Other, // Default case
        }
    }
}
impl core::fmt::Display for ZephyrDeviceError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self.inner.0 {
            zephyr_sys::ENOSYS => write!(f, "ENOSYS"),
            zephyr_sys::ENOTSUP => write!(f, "ENOTSUP"),
            _ => write!(f, "Zephyr error: {}", self.inner.0),
        }
    }
}
fn check_error(result: i32) -> Result<(), ZephyrDeviceError> {
    if result < 0 {
        let error = zephyr::Error((-result) as u32);
        Err(error.into())
    } else {
        Ok(())
    }
}

impl embedded_io::ErrorType for UartDevice {
    type Error = ZephyrDeviceError;
}
impl embedded_io::Read for UartDevice {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        unsafe {
            // Set the callback to receive UART events
            let user_data = &mut self.sender as *mut _ as *mut core::ffi::c_void;
            info!("user_data: {:?}", user_data);
            check_error(zephyr_uart_callback_set(self.device, uart_cb, user_data))?;
        }
        unsafe {
            check_error(zephyr_sys::uart_rx_enable(self.device, buf.as_mut_ptr(), buf.len(), 10000))?;
        }
        let mut bytes_received = 0;
        loop {
            match self.receiver.recv() {
                Ok(evt) => {
                    match evt.type_ {
                        zephyr_sys::uart_event_type_UART_RX_RDY => {
                            let rx = unsafe { evt.data.rx.as_ref() };
                            bytes_received = rx.offset + rx.len;
                            unsafe {
                                check_error(zephyr_sys::uart_rx_disable(self.device))
                                    .inspect_err(|e| {
                                        warn!("Failed to disable UART RX: {:?}", e);
                                    })
                                    .ok();
                            }
                        }
                        zephyr_sys::uart_event_type_UART_RX_BUF_RELEASED => {
                            debug!("UART RX buffer released - {} bytes received", bytes_received);
                            break;
                        }
                        type_ => {
                            debug!("UART event: {:?}", type_);
                        }
                    }
                }
                Err(_) => {
                    // No events received, continue waiting
                }
            }
        }
        Ok(bytes_received)
    }
}
impl embedded_io::Write for UartDevice {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        unsafe {
            check_error(zephyr_sys::uart_tx(self.device, buf.as_ptr(), buf.len(), -1))?;
        }
        Ok(buf.len())
    }
    fn flush(&mut self) -> Result<(), Self::Error> {
        // No-op for UART
        Ok(())
    }
}

#[no_mangle]
extern "C" fn rust_main() {
    unsafe {
        zephyr::set_logger().unwrap();
    }

    info!("Hello world from Rust on {}", zephyr::kconfig::CONFIG_BOARD);

    let uart = unsafe { devicetree_get_uart0() };
    if ! unsafe { zephyr_sys::device_is_ready(uart)} {
        warn!("UART device is not ready");
        return;
    }

    let mut uart = UartDevice::new(uart);
    // uart.configure(115200)
    //     .inspect_err(|e| {
    //         warn!("Failed to configure UART: {}", e);
    //     })
    //     .unwrap();
    let mut buf = [0u8; 64];
    loop {
        match uart.read(&mut buf) {
            Ok(size) if size > 0 => {
                info!("Read {} bytes: {:?}", size, &buf[..size]);
                match uart.write(&buf[..size]) {
                    Ok(_) => info!("Wrote {} bytes back", size),
                    Err(e) => warn!("Failed to write back: {:?}", e),
                }
            }
            Ok(_) => {
                // No data read
            }
            Err(e) => warn!("Error reading from UART: {:?}", e),
        }
        let duration = Duration::millis_at_least(100);
        sleep(duration);
    }
}
