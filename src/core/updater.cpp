#include <cstdint>
#include <cstring>
#include "pico/stdlib.h"
#include "hardware/flash.h"
#include "hardware/sync.h"

// Flash layout definitions (adjust based on your actual flash size)
#define FLASH_SECTOR_SIZE     4096
#define SLOT_A_OFFSET         0x00040000
#define SLOT_B_OFFSET         0x00120000
#define SLOT_SIZE             0x000E0000 // 896 KB per slot
#define METADATA_FLASH_OFFSET 0x001FC000

// Payload Type
#define PAYLOAD_TYPE_UPDATE   0x08

struct __attribute__((packed)) PacketHeader {
    uint8_t  target_id[8];
    uint16_t payload_type;
    uint32_t payload_length;
};

struct BootMetadata {
    uint32_t active_slot; // 0 for Slot A, 1 for Slot B
    uint32_t magic;       // Validation magic number
};

class Rp2350Updater {
public:
    Rp2350Updater(uint64_t my_target_id) : my_id(my_target_id) {}

    // Main entry point to parse incoming byte stream
    bool handle_incoming_data(const uint8_t* buffer, size_t total_len) {
        if (total_len < sizeof(PacketHeader) + 4) {
            return false; // Packet too short
        }

        PacketHeader header;
        std::memcpy(&header, buffer, sizeof(PacketHeader));

        // Validate Target ID (check if it matches ours or is broadcast)
        uint64_t packet_target_id;
        std::memcpy(&packet_target_id, header.target_id, 8);
        if (packet_target_id != my_id && packet_target_id != 0xFFFFFFFFFFFFFFFF) {
            return false; // Not meant for us
        }

        const uint8_t* payload_ptr = buffer + sizeof(PacketHeader);
        uint32_t payload_len = header.payload_length;

        // Check buffer bounds for payload + 4 bytes CRC-32 at the end
        if (total_len < sizeof(PacketHeader) + payload_len + 4) {
            return false;
        }

        // Extract received CRC-32
        uint32_t received_crc;
        std::memcpy(&received_crc, payload_ptr + payload_len, 4);

        // Verify CRC-32 over header + payload
        uint32_t calculated_crc = calculate_crc32(buffer, sizeof(PacketHeader) + payload_len);
        if (calculated_crc != received_crc) {
            return false; // CRC mismatch
        }

        // Handle based on payload type
        if (header.payload_type == PAYLOAD_TYPE_UPDATE) {
            return install_firmware(payload_ptr, payload_len);
        }

        return false; // Unknown payload type
    }

private:
    uint64_t my_id;

    // Standard CRC-32 calculator (IEEE 802.3)
    uint32_t calculate_crc32(const uint8_t* data, size_t length) {
        uint32_t crc = 0xFFFFFFFF;
        for (size_t i = 0; i < length; i++) {
            crc ^= data[i];
            for (uint8_t j = 0; j < 8; j++) {
                if (crc & 1) {
                    crc = (crc >> 1) ^ 0xEDB88320;
                } else {
                    crc >>= 1;
                }
            }
        }
        return ~crc;
    }

    // Read current boot metadata from flash
    BootMetadata get_metadata() {
        BootMetadata* meta = (BootMetadata*)(XIP_BASE + METADATA_FLASH_OFFSET);
        if (meta->magic != 0x5049434F) { // "PICO" magic
            // Default to Slot A if uninitialized
            return {0, 0x5049434F};
        }
        return *meta;
    }

    // Save updated metadata to flash
    void save_metadata(BootMetadata meta) {
        meta.magic = 0x5049434F;

        uint32_t ints = save_and_disable_interrupts();
        flash_range_erase(METADATA_FLASH_OFFSET, FLASH_SECTOR_SIZE);
        flash_range_program(METADATA_FLASH_OFFSET, (const uint8_t*)&meta, sizeof(BootMetadata));
        restore_interrupts(ints);
    }

    // Flash the new firmware into the *inactive* slot (A/B swapping)
    bool install_firmware(const uint8_t* payload, uint32_t length) {
        if (length > SLOT_SIZE) {
            return false; // Firmware too large for slot
        }

        BootMetadata meta = get_metadata();
        // Target the *inactive* slot for the update
        uint32_t target_slot_offset = (meta.active_slot == 0) ? SLOT_B_OFFSET : SLOT_A_OFFSET;

        // Erase the target slot flash sectors
        uint32_t sectors_to_erase = (length + FLASH_SECTOR_SIZE - 1) & ~(FLASH_SECTOR_SIZE - 1);

        uint32_t ints = save_and_disable_interrupts();

        flash_range_erase(target_slot_offset, sectors_to_erase);

        // Program the payload into flash in page-sized chunks (256 bytes)
        for (uint32_t i = 0; i < length; i += FLASH_PAGE_SIZE) {
            uint32_t chunk_size = (length - i < FLASH_PAGE_SIZE) ? (length - i) : FLASH_PAGE_SIZE;
            // Pad remainder of page with 0xFF if necessary
            uint8_t page_buf[FLASH_PAGE_SIZE];
            std::memset(page_buf, 0xFF, FLASH_PAGE_SIZE);
            std::memcpy(page_buf, payload + i, chunk_size);

            flash_range_program(target_slot_offset + i, page_buf, FLASH_PAGE_SIZE);
        }

        restore_interrupts(ints);

        // Switch active slot in metadata (triggering swap / rollback safety)
        meta.active_slot = (meta.active_slot == 0) ? 1 : 0;
        save_metadata(meta);

        return true;
    }
};
