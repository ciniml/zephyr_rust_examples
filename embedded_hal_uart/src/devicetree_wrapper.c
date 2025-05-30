#include <zephyr/kernel.h>
#include <zephyr/sys/printk.h>
#include <zephyr/drivers/uart.h>

#define DT_UART0 DT_NODELABEL(uart0)
static const struct device* uart0 = DEVICE_DT_GET(DT_UART0);

const struct device* devicetree_get_uart0(void) { return uart0; }