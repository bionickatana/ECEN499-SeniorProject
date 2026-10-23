#include <stdio.h>
#include "pico/stdlib.h"
#include "hardware/spi.h"

#include "tasks/status_light.hpp"
#include "tasks/max6675.hpp"


int main() {
    stdio_init_all();

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


    while(true) {
        statusLight.run(&status);
    }
}
