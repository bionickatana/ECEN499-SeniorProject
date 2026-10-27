#include "core/usb_transport.hpp"

#include "pico/stdlib.h"

size_t UsbTransport::read(uint8_t *buffer, size_t capacity) {
    size_t count = 0;
    while (count < capacity) {
        const int character = getchar_timeout_us(0);
        if (character == PICO_ERROR_TIMEOUT) break;
        if (character < 0) break;
        buffer[count++] = static_cast<uint8_t>(character);
    }
    return count;
}

bool UsbTransport::write(const uint8_t *buffer, size_t length) {
    for (size_t i = 0; i < length; ++i) {
        putchar_raw(static_cast<char>(buffer[i]));
    }
    return true;
}
