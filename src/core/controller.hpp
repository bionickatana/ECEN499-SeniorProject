#pragma once

#include <cstdint>

#include "core/protocol.hpp"
#include "core/updater.hpp"
#include "core/usb_transport.hpp"

class Controller {
public:
    Controller(uint64_t board_id, UsbTransport &transport, Rp2350Updater &updater);
    void poll();

private:
    static constexpr uint64_t kHubTargetId = 1;

    uint64_t board_id_;
    UsbTransport &transport_;
    Rp2350Updater &updater_;
    protocol::FrameParser parser_;
    protocol::Packet incoming_{};
    protocol::Packet outgoing_{};
    uint8_t frame_[protocol::kMaxEncodedFrameSize]{};

    void handle_packet(const protocol::Packet &packet);
    void send_ack(protocol::PayloadType acknowledged_type, protocol::AckStatus status, uint32_t detail);
    void send_welcome();
    void send_status();
};
