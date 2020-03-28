/// GB boot rom.
///
/// Maps to:
/// * `0x0000 - 0x00ff`
pub static BOOT_ROM_GB: &[u8] = include_bytes!(env!("BOOT_ROM_GB"));

/// CGB boot rom.
///
/// Maps to:
/// * `0x0000 - 0x00ff`
/// * `0x0150 - 0x0900`
pub static BOOT_ROM_CGB: &[u8] = include_bytes!(env!("BOOT_ROM_CGB"));

/// Returns true if the given address is part of the GB boot ROM, false
/// otherwise.
pub fn is_gb_addr(addr: u16) -> bool {
    matches!(addr, 0x0000..=0x00ff)
}

/// Returns true if the given address is part of the CGB boot ROM, false
/// otherwise.
pub fn is_cgb_addr(addr: u16) -> bool {
    matches!(addr, 0x0000..=0x00ff | 0x0150..=0x0900)
}
