cmake_minimum_required(VERSION 3.20.0)

find_package(Zephyr REQUIRED HINTS $ENV{ZEPHYR_BASE})
project(hello_rust_2)

rust_cargo_application()
target_sources(app PRIVATE src/cffi.c) 
