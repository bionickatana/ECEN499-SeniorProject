#include "status_light.hpp"

StatusLight::StatusLight() {
    bool rc = status_led_init();
    hard_assert(rc);
    hard_assert(colored_status_led_supported()); // This assert fails if your board does not have WS2812 support
    last_run = 0;
}

StatusLight::~StatusLight() {
    status_led_deinit();
}

void StatusLight::run(StatusObject* status_object) {
    uint32_t current_time = to_ms_since_boot(get_absolute_time());
    if (current_time > (last_run + STATUS_LIGHT_TASK_TIMER)) {
        last_run = current_time;
        uint32_t color;
        switch(*status_object) {
        case(OK):
            color = PICO_COLORED_STATUS_LED_COLOR_FROM_RGB(0x00, 0x44, 0x00);
            break;
        case(ERROR):
            color = PICO_COLORED_STATUS_LED_COLOR_FROM_RGB(0x44, 0x00, 0x00);
            break;
        case(WARNING):
            color = PICO_COLORED_STATUS_LED_COLOR_FROM_RGB(0x44, 0x00, 0x44);
            break;
        case(SENSOR_ERROR):
            color = PICO_COLORED_STATUS_LED_COLOR_FROM_RGB(0x88, 0x44, 0x00);
            break;
        }
        colored_status_led_set_on_with_color(color);
    }
}
