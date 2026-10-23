#pragma once

#include "pico/stdlib.h"
#include "pico/status_led.h"

#include "status.hpp"

#define STATUS_LIGHT_TASK_TIMER 10 // Task timer in ms

class StatusLight{
private:
    uint32_t last_run;

public:
    StatusLight();
    ~StatusLight();
    void run(StatusObject* status_object); // Function to call every main loop of task list
};
