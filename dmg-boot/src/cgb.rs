/// CGB boot rom.
///
/// Maps to:
/// * `0x0000 - 0x00ff`
/// * `0x0150 - 0x0900`
pub static ROM: &[u8] = include_bytes!(env!("DMG_BOOT_ROM_CGB"));

/// Returns true if the given address is part of the CGB boot ROM, false
/// otherwise.
pub fn is_boot(addr: u16) -> bool {
    matches!(addr, 0x0000..=0x00ff | 0x0150..=0x0900)
}
