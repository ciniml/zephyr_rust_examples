cmake_minimum_required(VERSION 3.20.0)

find_package(Zephyr REQUIRED HINTS $ENV{ZEPHYR_BASE})
project(embedded_hal_i2c)

rust_cargo_application()
target_sources(app PRIVATE src/devicetree_wrapper.c 
                           src/i2c_wrapper.c)