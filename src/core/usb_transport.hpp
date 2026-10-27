#pragma once

#include <cstddef>
#include <cstdint>

class UsbTransport {
public:
    size_t read(uint8_t *buffer, size_t capacity);
    bool write(const uint8_t *buffer, size_t length);
};
