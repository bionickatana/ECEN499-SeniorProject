//! Shared application services.
//!
//! Services hold shared state that multiple plugins need to read or update.
//! Hardware drivers do not belong here; place hardware access in `board.rs` or a
//! plugin that owns that peripheral.

pub mod sensor_data;
pub mod status;
