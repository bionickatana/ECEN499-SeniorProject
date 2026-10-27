#include <stdio.h>
#include "pico/stdlib.h"
#include "hardware/spi.h"
#include "hardware/flash.h"
#include "pico/flash.h"

#include "core/controller.hpp"
#include "core/updater.hpp"
#include "core/usb_transport.hpp"
#include "tasks/status_light.hpp"
#include "tasks/max6675.hpp"


int main() {
    stdio_init_all();
    hard_assert(flash_safe_execute_core_init());

    uint8_t board_id_bytes[FLASH_UNIQUE_ID_SIZE_BYTES];
    flash_get_unique_id(board_id_bytes);
    const uint64_t board_id = protocol::read_u64_le(board_id_bytes);

    // Global SPI setup //
    // Note that the configuration for the pins is in the CMakeLists.txt file.
    spi_init(TC_SPI_PORT, 1000 * 1000);

    // 2. Assign the GPIO functions to SPI mode for SCK and MISO
    gpio_set_function(TC_SPI_SCK, GPIO_FUNC_SPI);
    gpio_set_function(TC_SPI_MISO, GPIO_FUNC_SPI);

    // Global status object setup //
    StatusObject status = OK;


    // Status Light setup //
    StatusLight statusLight;

    // Sensor setup //
    Max6675 thermocouple01(TC_SPI_PORT, TC_SPI_CS01, TC_SPI_SCK, TC_SPI_MISO);
    Max6675 thermocouple02(TC_SPI_PORT, TC_SPI_CS02, TC_SPI_SCK, TC_SPI_MISO);
    Max6675 thermocouple03(TC_SPI_PORT, TC_SPI_CS03, TC_SPI_SCK, TC_SPI_MISO);
    Max6675 thermocouple04(TC_SPI_PORT, TC_SPI_CS04, TC_SPI_SCK, TC_SPI_MISO);

    // The controller owns several protocol buffers; static storage avoids
    // exhausting the small startup stack on the RP2350.
    static UsbTransport usb_transport;
    static Rp2350Updater updater;
    static Controller controller(board_id, usb_transport, updater);


    while(true) {
        controller.poll();
        updater.poll();
        thermocouple01.run(&status);
        thermocouple02.run(&status);
        thermocouple03.run(&status);
        thermocouple04.run(&status);
        statusLight.run(&status);
    }
}
