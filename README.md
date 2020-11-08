# GBR

GBR is a Game Boy emulator written in Rust.

It was created for the purpose of learning Rust and the CPU emulator, keeping in mind the readability of the code.

## Useage

```bash
cargo run --release -- --rom [filename]
```

### Joypad

Game Boy|Key
---|---
Up|Arrow Up
Down|Arrow Down
Left|Arrow Left
Right|Arrow Right
A|Z
B|X
START|Enter
SELECT|Backspace

### Features

- [ ] Cartridge
  - [x] No MBC
  - [x] MBC1
  - [ ] Other
- [x] PPU
- [x] Timer
- [ ] APU
- [ ] Serial I/O

## Test

```bash
cargo run --release -- --rom ./rom/individual/cpu_instrs.gb
```

## Platform

I have only checked the operation on Windows 10.

## References

Documents 📚

- [Game Boy™ CPU Manual](http://marc.rawer.de/Gameboy/Docs/GBCPUman.pdf)
- [Pan Docs - Game Boy technical reference](https://gbdev.io/pandocs/)

Videos 🎥

- [The Game Boy, a hardware autopsy - Part 1: the CPU](https://youtu.be/RZUDEaLa5Nw)
- [The Ultimate Game Boy Talk (33c3)](https://youtu.be/HyzD8pNlpwI)

Emulators 🎮

- [Gekkio/mooneye-gb](https://github.com/Gekkio/mooneye-gb)
- [bokuweb/gopher-boy](https://github.com/bokuweb/gopher-boy)