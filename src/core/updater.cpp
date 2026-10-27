#include "core/updater.hpp"

#include <cstring>

#include "hardware/flash.h"
#include "pico/flash.h"
#include "pico/platform.h"

namespace {
constexpr uint8_t kInstallBegin = 0;
constexpr uint8_t kInstallChunk = 1;

struct FlashOperation {
    uint32_t offset;
    const uint8_t *page;
};

void __no_inline_not_in_flash_func(erase_staging_sector)(void *parameter) {
    const auto *operation = static_cast<const FlashOperation *>(parameter);
    flash_range_erase(operation->offset, FLASH_SECTOR_SIZE);
}

void __no_inline_not_in_flash_func(program_staging_page)(void *parameter) {
    const auto *operation = static_cast<const FlashOperation *>(parameter);
    flash_range_program(operation->offset, operation->page, FLASH_PAGE_SIZE);
}

bool run_flash_operation(void (*operation)(void *), FlashOperation *parameters) {
    return flash_safe_execute(operation, parameters, 1000) == PICO_OK;
}
}  // namespace

Rp2350Updater::Rp2350Updater() = default;

bool Rp2350Updater::begin(uint32_t image_size, uint32_t image_crc) {
    if (image_size == 0 || image_size > kStagingSize) return false;
    image_size_ = image_size;
    expected_crc_ = image_crc;
    received_size_ = 0;
    erase_offset_ = kStagingOffset;
    erase_end_ = kStagingOffset + ((image_size + FLASH_SECTOR_SIZE - 1) & ~(FLASH_SECTOR_SIZE - 1));
    state_ = State::Erasing;
    return true;
}

bool Rp2350Updater::write_chunk(uint32_t offset, const uint8_t *data, uint32_t length) {
    if (state_ != State::Receiving || length == 0 || offset != received_size_ ||
        length > image_size_ - received_size_) {
        return false;
    }
    uint8_t page[FLASH_PAGE_SIZE];
    uint32_t consumed = 0;
    while (consumed < length) {
        const uint32_t page_offset = (offset + consumed) & ~(FLASH_PAGE_SIZE - 1);
        const uint32_t within_page = (offset + consumed) - page_offset;
        const uint32_t copy_length = (length - consumed < FLASH_PAGE_SIZE - within_page)
                                         ? length - consumed
                                         : FLASH_PAGE_SIZE - within_page;
        if (within_page != 0 || copy_length != FLASH_PAGE_SIZE) {
            std::memcpy(page, reinterpret_cast<const void *>(XIP_BASE + kStagingOffset + page_offset),
                        FLASH_PAGE_SIZE);
        } else {
            std::memset(page, 0xff, sizeof(page));
        }
        std::memcpy(page + within_page, data + consumed, copy_length);
        FlashOperation operation{kStagingOffset + page_offset, page};
        if (!run_flash_operation(program_staging_page, &operation)) return false;
        consumed += copy_length;
    }
    received_size_ += length;
    return true;
}

protocol::AckStatus Rp2350Updater::handle_install(const protocol::Packet &packet, uint32_t *detail) {
    if (packet.payload_length < 1) return protocol::AckStatus::BadPayload;
    if (packet.payload[0] == kInstallBegin) {
        if (packet.payload_length != 9 || !begin(protocol::read_u32_le(packet.payload + 1),
                                                 protocol::read_u32_le(packet.payload + 5))) {
            return protocol::AckStatus::Rejected;
        }
        *detail = image_size_;
        return protocol::AckStatus::Ok;
    }
    if (packet.payload[0] == kInstallChunk) {
        if (packet.payload_length <= 5) return protocol::AckStatus::BadPayload;
        const uint32_t offset = protocol::read_u32_le(packet.payload + 1);
        if (state_ == State::Erasing) return protocol::AckStatus::Busy;
        if (!write_chunk(offset, packet.payload + 5, packet.payload_length - 5)) {
            *detail = received_size_;
            return protocol::AckStatus::Rejected;
        }
        *detail = received_size_;
        return protocol::AckStatus::Ok;
    }
    return protocol::AckStatus::BadPayload;
}

protocol::AckStatus Rp2350Updater::handle_verify(const protocol::Packet &packet, uint32_t *detail) {
    if (packet.payload_length != 4 || state_ != State::Receiving || received_size_ != image_size_) {
        return protocol::AckStatus::Rejected;
    }
    const uint32_t supplied_crc = protocol::read_u32_le(packet.payload);
    if (supplied_crc != expected_crc_ ||
        protocol::crc32(reinterpret_cast<const uint8_t *>(XIP_BASE + kStagingOffset), image_size_) != expected_crc_) {
        state_ = State::Failed;
        return protocol::AckStatus::VerifyFailed;
    }
    state_ = State::Verified;
    *detail = image_size_;
    return protocol::AckStatus::Ok;
}

void Rp2350Updater::poll() {
    if (state_ != State::Erasing) return;
    if (erase_offset_ < erase_end_) {
        FlashOperation operation{erase_offset_, nullptr};
        if (!run_flash_operation(erase_staging_sector, &operation)) {
            state_ = State::Failed;
            return;
        }
        erase_offset_ += FLASH_SECTOR_SIZE;
    }
    if (erase_offset_ >= erase_end_) state_ = State::Receiving;
}

Rp2350Updater::State Rp2350Updater::state() const { return state_; }
uint32_t Rp2350Updater::image_size() const { return image_size_; }
uint32_t Rp2350Updater::received_size() const { return received_size_; }
