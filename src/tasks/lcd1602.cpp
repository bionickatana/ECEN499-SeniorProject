#include <stdio.h>
#include <string.h>
#include "lcd1602.hpp"

// PCF8574 I2C backpack bit layout: P0=RS, P1=RW, P2=EN, P3=Backlight, P4-P7=D4-D7
#define LCD_RS 0x01
#define LCD_EN 0x04
#define LCD_BACKLIGHT 0x08

Lcd1602::Lcd1602(i2c_inst_t *i2c, uint pin_sda, uint pin_scl, uint8_t addr, uint pin_power) {
    i2c_port = i2c;
    i2c_addr = addr;
    this->pin_power = pin_power;
    last_run = 0;
    dirty = true;
    for (uint8_t channel = 0; channel < LCD_CHANNELS; channel++) {
        temperatures[channel] = 0.0f;
    }

    gpio_init(pin_power);
    gpio_set_dir(pin_power, GPIO_OUT);
    power_on();
    sleep_ms(50); // Let the LCD's power rail stabilize before driving I2C

    i2c_init(i2c_port, 100 * 1000);
    gpio_set_function(pin_sda, GPIO_FUNC_I2C);
    gpio_set_function(pin_scl, GPIO_FUNC_I2C);
    gpio_pull_up(pin_sda);
    gpio_pull_up(pin_scl);

    // Standard HD44780 4-bit mode init sequence, driven through the PCF8574 expander
    sleep_ms(50);
    write_nibble(0x03 << 4, false);
    sleep_ms(5);
    write_nibble(0x03 << 4, false);
    sleep_ms(1);
    write_nibble(0x03 << 4, false);
    sleep_ms(1);
    write_nibble(0x02 << 4, false); // Switch to 4-bit mode
    sleep_ms(1);

    write_command(0x28); // 4-bit, 2-line, 5x8 dots
    write_command(0x0C); // Display on, cursor off, blink off
    write_command(0x06); // Entry mode: increment, no shift
    write_command(0x01); // Clear display
    sleep_ms(2);
}

Lcd1602::~Lcd1602() {
    // Optional cleanup logic if needed
}

void Lcd1602::write_nibble(uint8_t nibble, bool is_data) {
    uint8_t out = (nibble & 0xF0) | LCD_BACKLIGHT | (is_data ? LCD_RS : 0);
    uint8_t latch_high = out | LCD_EN;
    uint8_t latch_low = out & ~LCD_EN;

    i2c_write_blocking(i2c_port, i2c_addr, &latch_high, 1, false);
    sleep_us(1);
    i2c_write_blocking(i2c_port, i2c_addr, &latch_low, 1, false);
    sleep_us(50);
}

void Lcd1602::write_byte(uint8_t data, bool is_data) {
    write_nibble(data & 0xF0, is_data);
    write_nibble((data << 4) & 0xF0, is_data);
}

void Lcd1602::write_command(uint8_t cmd) {
    write_byte(cmd, false);
}

void Lcd1602::set_cursor(uint8_t row, uint8_t col) {
    static const uint8_t row_offsets[LCD_ROWS] = {0x00, 0x40};
    write_command(0x80 | (row_offsets[row] + col));
}

void Lcd1602::print_line(uint8_t row, const char* text) {
    set_cursor(row, 0);
    size_t len = strlen(text);
    for (uint8_t col = 0; col < LCD_COLS; col++) {
        char c = (col < len) ? text[col] : ' ';
        write_byte((uint8_t)c, true);
    }
}

void Lcd1602::power_on() {
    gpio_put(pin_power, true);
}

void Lcd1602::power_off() {
    gpio_put(pin_power, false);
}

void Lcd1602::set_temperature(uint8_t channel, float temperature) {
    if (channel >= LCD_CHANNELS) {
        return;
    }
    temperatures[channel] = temperature;
    dirty = true;
}

void Lcd1602::run(StatusObject* status_object) {
    uint32_t current_time = to_ms_since_boot(get_absolute_time());

    if (dirty && current_time > (last_run + LCD_TASK_TIMER)) {
        last_run = current_time;
        dirty = false;

        // Two channels per row, each field "N:XXX" (rounded to the nearest degree)
        for (uint8_t row = 0; row < LCD_ROWS; row++) {
            uint8_t left_channel = row * 2;
            uint8_t right_channel = row * 2 + 1;

            char line[LCD_COLS + 1];
            snprintf(line, sizeof(line), "%d:%4.0f %d:%4.0f",
                left_channel + 1, temperatures[left_channel],
                right_channel + 1, temperatures[right_channel]);
            print_line(row, line);
        }
    }
}
