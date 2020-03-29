# `DMG & CGB Emulator`

Yet another Game Boy emulator with support for GB and CGB games as well as SGB palettes.

## Features

| Feature        | Support | Notes
| ---            | :---:   | ---
| GB             | 👍       | Works on most games I tested (see compatibility table below)
| Color GB (CGB) | 👍       | Not fully tested (see compatibility table below)
| Super GB (SGB) |         |
| Sound          |         |
| Cycle accuracy |         | Outside of the current scope

If you encounter a game not currentl listed in the table below that doesn't run properly, please open an issue with the title.

## Building

`TODO`

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
