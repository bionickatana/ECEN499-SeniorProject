#include <cstdint>

#include "pico/stdlib.h"

namespace
{
    constexpr uint LED_PIN = 15;
    constexpr uint32_t BLINK_INTERVAL_MS = 500;
}

int main()
{
    gpio_init(LED_PIN);
    gpio_set_dir(LED_PIN, GPIO_OUT);

    while (true)
    {
        gpio_put(LED_PIN, 1);
        sleep_ms(BLINK_INTERVAL_MS);

        gpio_put(LED_PIN, 0);
        sleep_ms(BLINK_INTERVAL_MS);
    }

    return 0;
}
