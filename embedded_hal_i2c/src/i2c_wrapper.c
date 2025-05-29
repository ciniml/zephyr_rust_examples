#include <zephyr/kernel.h>
#include <zephyr/sys/printk.h>
#include <zephyr/drivers/i2c.h>

#include <stdint.h>

int zephyr_i2c_read_dt(const struct i2c_dt_spec * spec, uint8_t * buf, uint32_t num_bytes )
{
    return i2c_read_dt(spec, buf, num_bytes);
}


int zephyr_i2c_transfer(const struct device *dev, struct i2c_msg *msgs, uint8_t num_msgs, uint16_t addr)
{
    return i2c_transfer(dev, msgs, num_msgs, addr);
}
int zephyr_i2c_transfer_dt(const struct i2c_dt_spec * spec, struct i2c_msg *msgs, uint8_t num_msgs, uint16_t addr)
{
    return i2c_transfer(spec->bus, msgs, num_msgs, addr);
}