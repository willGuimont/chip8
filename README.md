# chip8

A CHIP-8 emulator written in Rust.

## Usage

```bash
cargo run --release -- --rom <path-to-rom> --scale <scale>
# e.g.
cargo run --release -- --rom rom/PONG2 --scale 10
```

Keys are mapped to 1-4, Q-R, A-F, and Z-V.
