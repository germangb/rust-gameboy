/// GB boot rom.
///
/// Maps to:
/// * `0x0000 - 0x00ff`
pub static ROM: &[u8] = include_bytes!(env!("DMG_BOOT_GB_ROM"));

/// Returns true if the given address is part of the GB boot ROM, false
/// otherwise.
pub fn is_boot(addr: u16) -> bool {
    matches!(addr, 0x0000..=0x00ff)
}
