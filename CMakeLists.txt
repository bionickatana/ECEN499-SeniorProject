cmake_minimum_required(VERSION 3.13)

set(CMAKE_C_STANDARD 11)
set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# Use the SDK downloaded by the Rakefile.
set(PICO_SDK_PATH "${CMAKE_CURRENT_LIST_DIR}/deps/pico-sdk"
    CACHE PATH "Path to the Raspberry Pi Pico SDK")

include("${PICO_SDK_PATH}/external/pico_sdk_import.cmake")

project(blinky LANGUAGES C CXX ASM)

pico_sdk_init()

add_executable(blinky
    src/main.cpp
)

target_link_libraries(blinky
    pico_stdlib
)

# Generate ELF, BIN, HEX, and UF2 output files.
pico_add_extra_outputs(blinky)

# Disable USB and UART stdio for this simple GPIO example.
pico_enable_stdio_usb(blinky 0)
pico_enable_stdio_uart(blinky 0)
