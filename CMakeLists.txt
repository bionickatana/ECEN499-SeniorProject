# --- CENTRAL CONFIGURATION ---
set(MY_WS2812_PIN 16) # Status light
set(SPI_PORT spi0) # The SPI port for the thermocouples
set(SPI_SCK 2) # SPI clock pin
set(SPI_MISO 0) # SPI Pico In, MAX6675 out
set(SPI_TC01 1) # Chip Select TC1
set(SPI_TC02 3) # Chip Select TC2
set(SPI_TC03 6) # Chip Select TC3
set(SPI_TC04 7) # Chip Select TC4
# -----------------------------

cmake_minimum_required(VERSION 3.13)

set(CMAKE_C_STANDARD 11)
set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# Use the SDK downloaded by the Rakefile.
set(PICO_SDK_PATH "${CMAKE_CURRENT_LIST_DIR}/deps/pico-sdk"
    CACHE PATH "Path to the Raspberry Pi Pico SDK")

include("${PICO_SDK_PATH}/external/pico_sdk_import.cmake")

project(INL_temperature_nodes LANGUAGES C CXX ASM)

pico_sdk_init()

add_executable(INL_temperature_nodes
    src/main.cpp
    src/tasks/status_light.cpp
    src/tasks/max6675.cpp
)

target_link_libraries(INL_temperature_nodes
    pico_stdlib
    pico_status_led
    hardware_spi
)

target_compile_definitions(INL_temperature_nodes PRIVATE
    PICO_DEFAULT_WS2812_PIN=${MY_WS2812_PIN}
    TC_SPI_PORT=${SPI_PORT}
    TC_SPI_SCK=${SPI_SCK}
    TC_SPI_MISO=${SPI_MISO}
    TC_SPI_CS01=${SPI_TC01}
    TC_SPI_CS02=${SPI_TC02}
    TC_SPI_CS03=${SPI_TC03}
    TC_SPI_CS04=${SPI_TC04}

)

# Configure the application to start after the 32 KiB Picoboot3
# bootloader area. Requires memory_flash.incl in this directory.
#
# Also requires section_boot2.incl for RP2040 only.
pico_add_linker_script_override_path(
    INL_temperature_nodes
    ${CMAKE_CURRENT_LIST_DIR}
)

# Generate ELF, BIN, HEX, and UF2 output files.
pico_add_extra_outputs(INL_temperature_nodes)

# Enable USB but not UART
pico_enable_stdio_usb(INL_temperature_nodes 1)
pico_enable_stdio_uart(INL_temperature_nodes 0)
