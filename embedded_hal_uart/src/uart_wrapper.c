#include <zephyr/kernel.h>
#include <zephyr/drivers/uart.h>

#include <stdint.h>

int zephyr_uart_callback_set (const struct device *dev, uart_callback_t callback, void *user_data) {
    return uart_callback_set(dev, callback, user_data);
}