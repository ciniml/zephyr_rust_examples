#include <zephyr/kernel.h>
#include <zephyr/sys/printk.h>
#include <zephyr/drivers/gpio.h>

#include <stdio.h>

#define SLEEP_TIME_MS   1000

#define LED0_NODE DT_ALIAS(led0)
#define LED1_NODE DT_ALIAS(led1)

static const struct gpio_dt_spec led0 = GPIO_DT_SPEC_GET(LED0_NODE, gpios);
static const struct gpio_dt_spec led1 = GPIO_DT_SPEC_GET(LED1_NODE, gpios);

int main(void)
{
    int ret = 0;

    if (!gpio_is_ready_dt(&led0) || !gpio_is_ready_dt(&led1)) {
		return 0;
	}

    ret = gpio_pin_configure_dt(&led0, GPIO_OUTPUT_ACTIVE);
	if (ret < 0) {
		return 0;
	}
    ret = gpio_pin_configure_dt(&led1, GPIO_OUTPUT_ACTIVE);
	if (ret < 0) {
		return 0;
	}
    
    while (1) {
        printf("Hello World!\n");
		gpio_pin_toggle_dt(&led0);
		gpio_pin_toggle_dt(&led1);
		
		k_msleep(SLEEP_TIME_MS);
	}
	return 0;
}