#pragma once

#include <stdint.h>
#include "pico/stdlib.h"
#include "hardware/i2c.h"
#include "status.hpp"

#define LCD_TASK_TIMER 500 // Task interval in milliseconds
#define LCD_COLS 16
#define LCD_ROWS 2
#define LCD_CHANNELS 4 // Two temperature channels packed per row

class Lcd1602 {
public:
    Lcd1602(i2c_inst_t *i2c, uint pin_sda, uint pin_scl, uint8_t addr, uint pin_power);
    ~Lcd1602();

    void set_temperature(uint8_t channel, float temperature); // channel is 0-3, two per LCD row
    void power_on(); // Switches the ground-side MOSFET on, enabling the screen
    void power_off(); // Switches the ground-side MOSFET off, disabling the screen
    void run(StatusObject* status_object);

private:
    i2c_inst_t *i2c_port;
    uint8_t i2c_addr;
    uint pin_power;
    uint32_t last_run;
    float temperatures[LCD_CHANNELS];
    bool dirty;

    void write_nibble(uint8_t nibble, bool is_data);
    void write_byte(uint8_t data, bool is_data);
    void write_command(uint8_t cmd);
    void set_cursor(uint8_t row, uint8_t col);
    void print_line(uint8_t row, const char* text);
};
