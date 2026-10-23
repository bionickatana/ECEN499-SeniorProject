#pragma once

#include <stdint.h>
#include "pico/stdlib.h"
#include "hardware/spi.h"
#include "status.hpp"

#define MAX6675_TASK_TIMER 1000 // Task interval in milliseconds (1 second)

class Max6675 {
public:
    Max6675(spi_inst_t *spi, uint pin_cs, uint pin_sck, uint pin_miso);
    ~Max6675();

    void run(StatusObject* status_object);
    float get_temperature() const;

private:
    spi_inst_t *spi_port;
    uint cs_pin;
    uint sck_pin;
    uint miso_pin;
    uint32_t last_run;
    float current_temperature;
};
