# `DMG Emulator`

![](assets/zelda.gif)

## Building

You need to provide your own boot rom.

## Tests

| Test | Pass |
| --- | :---: |
| `01-special.gb` | 👍 |
| `02-interrupts.gb` | 👍 |
| `03-op sp,hl.gb` | 👍 |
| `04-op r,imm.gb` | 👍 |
| `05-op rp.gb` | 👍 |
| `06-ld r,r.gb` | 👍 |
| `07-jr,jp,call,ret,rst.gb` | 👍 |
| `08-misc instrs.gb` | 👍 |
| `09-op r,r.gb` | 👍 |
| `10-bit ops.gb` | 👍 |
| `11-op a,(hl).gb` | 👍 |

## Tested games

| Rom | Works | Comments |
| --- | --- | --- |

## Resources

- https://github.com/AntonioND/giibiiadvance/blob/master/docs/TCAGBD.pdf
- https://gekkio.fi/files/gb-docs/gbctr.pdf
