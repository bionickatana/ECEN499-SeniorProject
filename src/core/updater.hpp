#pragma once

#include <cstdint>

#include "core/protocol.hpp"

class Rp2350Updater {
public:
    enum class State : uint8_t { Idle = 0, Erasing = 1, Receiving = 2, Verified = 3, Failed = 4 };

    Rp2350Updater();

    protocol::AckStatus handle_install(const protocol::Packet &packet, uint32_t *detail);
    protocol::AckStatus handle_verify(const protocol::Packet &packet, uint32_t *detail);
    void poll();

    State state() const;
    uint32_t image_size() const;
    uint32_t received_size() const;

private:
    static constexpr uint32_t kStagingOffset = 0x00200000;
    static constexpr uint32_t kMetadataOffset = 0x003ff000;
    static constexpr uint32_t kStagingSize = kMetadataOffset - kStagingOffset;

    State state_ = State::Idle;
    uint32_t image_size_ = 0;
    uint32_t expected_crc_ = 0;
    uint32_t received_size_ = 0;
    uint32_t erase_offset_ = 0;
    uint32_t erase_end_ = 0;

    bool begin(uint32_t image_size, uint32_t image_crc);
    bool write_chunk(uint32_t offset, const uint8_t *data, uint32_t length);
};
