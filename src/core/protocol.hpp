#pragma once

#include <cstddef>
#include <cstdint>

namespace protocol {

constexpr uint64_t kBroadcastTargetId = 0;
constexpr size_t kHeaderSize = 14;
constexpr size_t kCrcSize = 4;
constexpr size_t kMaxPayloadSize = 1024;
constexpr size_t kMaxPacketSize = kHeaderSize + kMaxPayloadSize + kCrcSize;
constexpr size_t kMaxEncodedFrameSize = kMaxPacketSize + (kMaxPacketSize / 254) + 2;

enum class PayloadType : uint16_t {
    Hello = 0x01,
    Welcome = 0x02,
    Ack = 0x03,
    ReadingRequest = 0x04,
    ReadingResponse = 0x05,
    Status = 0x06,
    Verify = 0x07,
    Install = 0x08,
    Reboot = 0x09,
};

enum class AckStatus : uint8_t {
    Ok = 0,
    BadPayload = 1,
    Busy = 2,
    Rejected = 3,
    VerifyFailed = 4,
    Unsupported = 5,
};

struct Packet {
    uint64_t target_id;
    PayloadType type;
    uint32_t payload_length;
    uint8_t payload[kMaxPayloadSize];
};

uint16_t read_u16_le(const uint8_t *data);
uint32_t read_u32_le(const uint8_t *data);
uint64_t read_u64_le(const uint8_t *data);
void write_u16_le(uint8_t *data, uint16_t value);
void write_u32_le(uint8_t *data, uint32_t value);
void write_u64_le(uint8_t *data, uint64_t value);

uint32_t crc32(const uint8_t *data, size_t length);

// Encodes a packet as COBS(packet || CRC-32) followed by a zero delimiter.
bool encode_frame(const Packet &packet, uint8_t *output, size_t output_capacity,
                  size_t *output_length);

// Decodes a COBS frame without its trailing zero delimiter.
bool decode_frame(const uint8_t *encoded, size_t encoded_length, Packet *packet);

class FrameParser {
public:
    // Consumes one transport byte. Returns true only when a complete valid packet is produced.
    bool push(uint8_t byte, Packet *packet);
    void reset();

private:
    uint8_t encoded_[kMaxEncodedFrameSize]{};
    size_t encoded_length_ = 0;
    bool overflowed_ = false;
};

}  // namespace protocol
