***ARCHIVED***

*This project has been archived. Future work will continue in https://github.com/germangb/rust-gameboy2*

* * *

# `rust-gameboy`

**Game Boy** and **Game Boy Color** emulation in Rust.

Still a bit *buggy*, but works with most games I've tried, albeit with some minor graphical glitched showing up on some of them.

![](assets/zelda.gif)
![](assets/mario.gif)

## Features

| Feature | Support | Notes
| --- | :-----: | ---
| Cycle accuracy | ❌ | Out of scope (I might change my mind later).
| Classic GB | ✔️ | Works with most games, except the ones that require cycle accuracy.
| Color (CGB) | ✔️ | Still buggy. Working on it.
| Sound | | Still buggy. Working on it.
| Link cable | | In scope but not implemented yet.

## Debugging

The GL-based frontend has some debugging functionality (only graphics for now though).

![](assets/debug.png)

## Tests

### CPU instruction tests

```bash
cargo test cpu_instrs
```

![](assets/cpu_instrs.png)

### CPU timing tests

```bash
cargo test instr_timing
```

![](assets/instr_timing.png)

## Tested Games

| Rom | Works | Comments
| --- | ----- | ---

## License

`TODO`

## Resources

- http://problemkaputt.de/pandocs.htm
- https://gbdev.gg8.se/wiki/
- https://github.com/AntonioND/giibiiadvance/blob/master/docs/TCAGBD.pdf
- https://gekkio.fi/files/gb-docs/gbctr.pdf
- https://github.com/gbdev/awesome-gbdev
