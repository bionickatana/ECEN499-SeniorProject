#include "core/controller.hpp"

#include <cstring>

#ifndef PROGRAM_VERSION
#define PROGRAM_VERSION "unknown"
#endif

namespace {
constexpr char kProgramVersion[] = PROGRAM_VERSION;
}

Controller::Controller(uint64_t board_id, UsbTransport &transport, Rp2350Updater &updater)
    : board_id_(board_id), transport_(transport), updater_(updater) {}

void Controller::poll() {
    uint8_t bytes[64];
    const size_t count = transport_.read(bytes, sizeof(bytes));
    for (size_t i = 0; i < count; ++i) {
        if (parser_.push(bytes[i], &incoming_)) handle_packet(incoming_);
    }
}

void Controller::handle_packet(const protocol::Packet &packet) {
    if (packet.target_id != board_id_ && packet.target_id != protocol::kBroadcastTargetId) return;

    uint32_t detail = 0;
    switch (packet.type) {
        case protocol::PayloadType::Hello:
            send_welcome();
            break;
        case protocol::PayloadType::Status:
            send_status();
            break;
        case protocol::PayloadType::Install:
            send_ack(packet.type, updater_.handle_install(packet, &detail), detail);
            break;
        case protocol::PayloadType::Verify:
            send_ack(packet.type, updater_.handle_verify(packet, &detail), detail);
            break;
        case protocol::PayloadType::Reboot:
            // A verified image remains staged until an immutable boot manager is installed.
            send_ack(packet.type, protocol::AckStatus::Unsupported, 0);
            break;
        default:
            send_ack(packet.type, protocol::AckStatus::Unsupported, 0);
            break;
    }
}

void Controller::send_ack(protocol::PayloadType acknowledged_type, protocol::AckStatus status,
                          uint32_t detail) {
    outgoing_ = {};
    outgoing_.target_id = kHubTargetId;
    outgoing_.type = protocol::PayloadType::Ack;
    outgoing_.payload_length = 7;
    protocol::write_u16_le(outgoing_.payload, static_cast<uint16_t>(acknowledged_type));
    outgoing_.payload[2] = static_cast<uint8_t>(status);
    protocol::write_u32_le(outgoing_.payload + 3, detail);

    size_t length = 0;
    if (protocol::encode_frame(outgoing_, frame_, sizeof(frame_), &length)) transport_.write(frame_, length);
}

void Controller::send_welcome() {
    const size_t version_length = std::strlen(kProgramVersion);
    if (version_length > protocol::kMaxPayloadSize - 9) return;

    outgoing_ = {};
    outgoing_.target_id = kHubTargetId;
    outgoing_.type = protocol::PayloadType::Welcome;
    outgoing_.payload_length = static_cast<uint32_t>(9 + version_length);
    protocol::write_u64_le(outgoing_.payload, board_id_);
    outgoing_.payload[8] = static_cast<uint8_t>(version_length);
    std::memcpy(outgoing_.payload + 9, kProgramVersion, version_length);

    size_t length = 0;
    if (protocol::encode_frame(outgoing_, frame_, sizeof(frame_), &length)) transport_.write(frame_, length);
}

void Controller::send_status() {
    outgoing_ = {};
    outgoing_.target_id = kHubTargetId;
    outgoing_.type = protocol::PayloadType::Status;
    outgoing_.payload_length = 9;
    outgoing_.payload[0] = static_cast<uint8_t>(updater_.state());
    protocol::write_u32_le(outgoing_.payload + 1, updater_.received_size());
    protocol::write_u32_le(outgoing_.payload + 5, updater_.image_size());

    size_t length = 0;
    if (protocol::encode_frame(outgoing_, frame_, sizeof(frame_), &length)) transport_.write(frame_, length);
}
