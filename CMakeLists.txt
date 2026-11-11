# --- CENTRAL CONFIGURATION ---
set(MY_WS2812_PIN 16) # Status light

set(SPI_PORT spi0) # The SPI port for the thermocouples
set(SPI_SCK 2) # SPI clock pin
set(SPI_MISO 0) # SPI Pico In, MAX6675 out
set(SPI_TC01 1) # Chip Select TC1
set(SPI_TC02 6) # Chip Select TC2
set(SPI_TC03 7) # Chip Select TC3
set(SPI_TC04 8) # Chip Select TC4

set(LCD_I2C_PORT i2c1) # The I2C port for the 20x4 LCD (GPIO 14/15 map to I2C1)
set(LCD_SDA 14) # I2C data pin
set(LCD_SCL 15) # I2C clock pin
set(LCD_ADDR 0x27) # PCF8574 I2C backpack address
set(LCD_POWER_PIN 26) # Gates the MOSFET switching the LCD's ground, turning the screen on/off


set(PROGRAM_VERSION "0.1.0") # Reported during the USB protocol handshake
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
    src/core/protocol.cpp
    src/core/usb_transport.cpp
    src/core/updater.cpp
    src/core/controller.cpp
    src/tasks/status_light.cpp
    src/tasks/max6675.cpp
    src/tasks/lcd1602.cpp
)

target_include_directories(INL_temperature_nodes PRIVATE
    ${CMAKE_CURRENT_LIST_DIR}/src
)

target_link_libraries(INL_temperature_nodes
    pico_stdlib
    pico_status_led
    hardware_spi
    hardware_i2c
    hardware_flash
    pico_flash
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
    LCD_I2C_PORT=${LCD_I2C_PORT}
    LCD_SDA=${LCD_SDA}
    LCD_SCL=${LCD_SCL}
    LCD_ADDR=${LCD_ADDR}
    LCD_POWER_PIN=${LCD_POWER_PIN}
    PROGRAM_VERSION="${PROGRAM_VERSION}"

)

# The board's physical 4 MiB flash remains visible to the flash driver. This
# linker override limits only the executable image to the lower 2 MiB, leaving
# the upper half available for a staged update image.
pico_add_linker_script_override_path(
    INL_temperature_nodes
    ${CMAKE_CURRENT_LIST_DIR}
)

# Generate ELF, BIN, HEX, and UF2 output files.
pico_add_extra_outputs(INL_temperature_nodes)

# Enable USB but not UART
pico_enable_stdio_usb(INL_temperature_nodes 1)
pico_enable_stdio_uart(INL_temperature_nodes 0)
