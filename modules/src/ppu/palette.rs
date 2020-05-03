pub type Color = [u8; 3];

macro_rules! palette {
    ($(pub const $name:ident : [Color; 4] = $colors:expr ;)*) => {
        $(pub const $name: [Color; 4] = $colors;)*

        /// Return an iterator over the built-in palettes.
        pub fn palettes() -> impl Iterator<Item=[Color; 4]> {
            vec![$($colors,)*].into_iter()
        }
    }
}

palette! {
    pub const GRAYSCALE: [Color; 4] = [
        [0xff, 0xff, 0xff],
        [0xaa, 0xaa, 0xaa],
        [0x55, 0x55, 0x55],
        [0x00, 0x00, 0x00],
    ];
    pub const MUDDYSAND: [Color; 4] = [
        [0xe6, 0xd6, 0x9c],
        [0xb4, 0xa5, 0x6a],
        [0x7b, 0x71, 0x62],
        [0x39, 0x38, 0x29],
    ];
    pub const DMG: [Color; 4] = [
        [0x7e, 0x84, 0x16],
        [0x57, 0x7b, 0x46],
        [0x38, 0x5d, 0x49],
        [0x2e, 0x46, 0x3d],
    ];
}
