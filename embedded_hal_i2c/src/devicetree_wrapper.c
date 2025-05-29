#include <zephyr/kernel.h>
#include <zephyr/sys/printk.h>
#include <zephyr/drivers/i2c.h>

#define DT_LSM6DS3TR_C DT_NODELABEL(lsm6ds3tr_c)
static const struct i2c_dt_spec i2c_lsm6ds3tr_c = I2C_DT_SPEC_GET(DT_LSM6DS3TR_C);

const struct i2c_dt_spec* devicetree_get_i2c_lsm6ds3tr_c(void) { return &i2c_lsm6ds3tr_c; }