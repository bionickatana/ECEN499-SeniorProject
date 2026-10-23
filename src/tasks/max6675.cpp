#include "max6675.hpp"

Max6675::Max6675(spi_inst_t *spi, uint pin_cs, uint pin_sck, uint pin_miso) {
    spi_port = spi;
    cs_pin = pin_cs;
    sck_pin = pin_sck;
    miso_pin = pin_miso;
    last_run = 0;
    current_temperature = 0.0f;

    // Initialize SPI peripheral at 1 MHz (MAX6675 supports up to 4.3 MHz)
    spi_init(spi_port, 1000 * 1000);

    // Set up GPIO functions for SPI SCK and MISO (SO) pins
    gpio_set_function(sck_pin, GPIO_FUNC_SPI);
    gpio_set_function(miso_pin, GPIO_FUNC_SPI);

    // Configure Chip Select (CS) pin as an output and set it high (deselected)
    gpio_init(cs_pin);
    gpio_set_dir(cs_pin, GPIO_OUT);
    gpio_put(cs_pin, 1);
}

Max6675::~Max6675() {
    // Optional cleanup logic if needed
}

void Max6675::run(StatusObject* status_object) {
    uint32_t current_time = to_ms_since_boot(get_absolute_time());

    // Non-blocking check for the 1-second interval
    if (current_time > (last_run + MAX6675_TASK_TIMER)) {
        last_run = current_time;

        // Read 16 bits from the MAX6675 sensor over SPI
        uint8_t buffer[2] = {0};

        gpio_put(cs_pin, 0); // Assert Chip Select (active low)
        spi_read_blocking(spi_port, 0xFF, buffer, 2);
        gpio_put(cs_pin, 1); // Deassert Chip Select

        // Combine the two bytes into a 16-bit word (MSB first)
        uint16_t raw_data = (buffer[0] << 8) | buffer[1];

        // Check Bit D2: Thermocouple input open detector (1 = disconnected/open)
        bool thermocouple_open = (raw_data & 0x04) != 0;

        if (thermocouple_open) {
            // Update your status object pointer here if an error occurs, e.g.:
            *status_object = SENSOR_ERROR;
        } else {
            // Extract temperature bits (D14 down to D3) -> 12 bits, 0.25°C per LSB
            uint16_t temp_bits = (raw_data >> 3) & 0x0FFF;
            current_temperature = temp_bits * 0.25f;
        }
    }
}

float Max6675::get_temperature() const {
    return current_temperature;
}
