extern crate minifb;

use std::fs;
use minifb::{Key, Window, WindowOptions};
use crate::chip8::{Chip8, DISPLAY_WIDTH, DISPLAY_HEIGHT};

// TODO CLI param
const SCALE: usize = 10;
const WIDTH: usize = DISPLAY_WIDTH * SCALE;
const HEIGHT: usize = DISPLAY_HEIGHT * SCALE;

mod chip8;

fn get_index(i: usize, j: usize, width: usize) -> usize {
    i + j * width
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // TODO argument parsing
    // TODO receive path as argument
    let rom = fs::read("rom/test_opcode.ch8")?;
    let mut chip = Chip8::new(rom);

    {
        let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];
        buffer[10] = 0xFFFFFFFF;

        let mut window = Window::new(
            "chip8",
            WIDTH,
            HEIGHT,
            WindowOptions::default(),
        )
            .unwrap_or_else(|e| {
                panic!("{}", e);
            });

        // Limit to max ~60 fps update rate
        window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

        while window.is_open() && !window.is_key_down(Key::Escape) {
            // TODO handle keys (write to keypad memory)

            chip.tick();
            // TODO run multiple steps (according to frequency)
            // TODO error handling
            chip.step()?;
            let display = chip.get_display();

            for i in 0..DISPLAY_WIDTH {
                for j in 0..DISPLAY_HEIGHT {
                    let display_index = get_index(i, j, DISPLAY_WIDTH);
                    let pixel_value = display[display_index];
                    for di in 0..SCALE {
                        for dj in 0..SCALE {
                            let buffer_index = get_index(i * SCALE + di, j * SCALE + dj, WIDTH);
                            buffer[buffer_index] = pixel_value;
                        }
                    }
                }
            }

            // TODO error handling
            window
                .update_with_buffer(&buffer, WIDTH, HEIGHT)
                .unwrap();
        }
    }
    Ok(())
}
