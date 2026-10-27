#include "core/protocol.hpp"

#include <cstring>

namespace protocol {

uint16_t read_u16_le(const uint8_t *data) {
    return static_cast<uint16_t>(data[0]) |
           (static_cast<uint16_t>(data[1]) << 8);
}

uint32_t read_u32_le(const uint8_t *data) {
    return static_cast<uint32_t>(data[0]) |
           (static_cast<uint32_t>(data[1]) << 8) |
           (static_cast<uint32_t>(data[2]) << 16) |
           (static_cast<uint32_t>(data[3]) << 24);
}

uint64_t read_u64_le(const uint8_t *data) {
    uint64_t value = 0;
    for (uint8_t i = 0; i < 8; ++i) {
        value |= static_cast<uint64_t>(data[i]) << (i * 8);
    }
    return value;
}

void write_u16_le(uint8_t *data, uint16_t value) {
    data[0] = static_cast<uint8_t>(value);
    data[1] = static_cast<uint8_t>(value >> 8);
}

void write_u32_le(uint8_t *data, uint32_t value) {
    for (uint8_t i = 0; i < 4; ++i) {
        data[i] = static_cast<uint8_t>(value >> (i * 8));
    }
}

void write_u64_le(uint8_t *data, uint64_t value) {
    for (uint8_t i = 0; i < 8; ++i) {
        data[i] = static_cast<uint8_t>(value >> (i * 8));
    }
}

uint32_t crc32(const uint8_t *data, size_t length) {
    uint32_t crc = 0xffffffffu;
    for (size_t i = 0; i < length; ++i) {
        crc ^= data[i];
        for (uint8_t bit = 0; bit < 8; ++bit) {
            crc = (crc & 1u) ? (crc >> 1u) ^ 0xedb88320u : crc >> 1u;
        }
    }
    return ~crc;
}

static bool cobs_encode(const uint8_t *input, size_t input_length, uint8_t *output,
                        size_t output_capacity, size_t *output_length) {
    if (output_capacity == 0) return false;
    size_t read = 0;
    size_t write = 1;
    size_t code_index = 0;
    uint8_t code = 1;
    while (read < input_length) {
        if (input[read] == 0) {
            if (code_index >= output_capacity) return false;
            output[code_index] = code;
            code_index = write++;
            code = 1;
        } else {
            if (write >= output_capacity) return false;
            output[write++] = input[read];
            if (++code == 0xff) {
                if (code_index >= output_capacity) return false;
                output[code_index] = code;
                code_index = write++;
                code = 1;
            }
        }
        ++read;
    }
    if (code_index >= output_capacity) return false;
    output[code_index] = code;
    *output_length = write;
    return true;
}

static bool cobs_decode(const uint8_t *input, size_t input_length, uint8_t *output,
                        size_t output_capacity, size_t *output_length) {
    size_t read = 0;
    size_t write = 0;
    while (read < input_length) {
        const uint8_t code = input[read++];
        if (code == 0 || read + code - 1 > input_length) return false;
        for (uint8_t i = 1; i < code; ++i) {
            if (write >= output_capacity) return false;
            output[write++] = input[read++];
        }
        if (code != 0xff && read < input_length) {
            if (write >= output_capacity) return false;
            output[write++] = 0;
        }
    }
    *output_length = write;
    return true;
}

bool encode_frame(const Packet &packet, uint8_t *output, size_t output_capacity,
                  size_t *output_length) {
    if (packet.payload_length > kMaxPayloadSize || output_capacity < 2) return false;
    // Protocol processing is single-threaded in the controller loop. Keep this
    // large work area out of the constrained embedded call stack.
    static uint8_t raw[kMaxPacketSize];
    write_u64_le(raw, packet.target_id);
    write_u16_le(raw + 8, static_cast<uint16_t>(packet.type));
    write_u32_le(raw + 10, packet.payload_length);
    std::memcpy(raw + kHeaderSize, packet.payload, packet.payload_length);
    const size_t raw_length = kHeaderSize + packet.payload_length;
    write_u32_le(raw + raw_length, crc32(raw, raw_length));

    size_t encoded_length = 0;
    if (!cobs_encode(raw, raw_length + kCrcSize, output, output_capacity - 1, &encoded_length)) {
        return false;
    }
    output[encoded_length++] = 0;
    *output_length = encoded_length;
    return true;
}

bool decode_frame(const uint8_t *encoded, size_t encoded_length, Packet *packet) {
    static uint8_t raw[kMaxPacketSize];
    size_t raw_length = 0;
    if (!cobs_decode(encoded, encoded_length, raw, sizeof(raw), &raw_length) ||
        raw_length < kHeaderSize + kCrcSize) {
        return false;
    }
    const uint32_t payload_length = read_u32_le(raw + 10);
    if (payload_length > kMaxPayloadSize || raw_length != kHeaderSize + payload_length + kCrcSize) {
        return false;
    }
    const uint32_t received_crc = read_u32_le(raw + kHeaderSize + payload_length);
    if (crc32(raw, kHeaderSize + payload_length) != received_crc) return false;

    packet->target_id = read_u64_le(raw);
    packet->type = static_cast<PayloadType>(read_u16_le(raw + 8));
    packet->payload_length = payload_length;
    std::memcpy(packet->payload, raw + kHeaderSize, payload_length);
    return true;
}

bool FrameParser::push(uint8_t byte, Packet *packet) {
    if (byte == 0) {
        const bool complete = !overflowed_ && encoded_length_ != 0 &&
                              decode_frame(encoded_, encoded_length_, packet);
        reset();
        return complete;
    }
    if (encoded_length_ >= sizeof(encoded_)) {
        overflowed_ = true;
        return false;
    }
    encoded_[encoded_length_++] = byte;
    return false;
}

void FrameParser::reset() {
    encoded_length_ = 0;
    overflowed_ = false;
}

}  // namespace protocol
