extern crate minifb;
extern crate clap;

use std::fs;
use minifb::{Key, Window, WindowOptions};
use clap::{Arg, App};

use crate::chip8::{Chip8, DISPLAY_WIDTH, DISPLAY_HEIGHT, NUMBER_OF_KEYS, KEY_NOT_PRESSED, KEY_PRESSED};

mod chip8;

fn get_index(i: usize, j: usize, width: usize) -> usize {
    i + j * width
}

fn get_keys(window: &Window) -> [u8; NUMBER_OF_KEYS] {
    let mut keys= [KEY_NOT_PRESSED; NUMBER_OF_KEYS];

    if window.is_key_down(Key::Key1) {
        keys[0] = KEY_PRESSED;
    }
    if window.is_key_down(Key::Key2) {
        keys[1] = KEY_PRESSED;
    }
    if window.is_key_down(Key::Key3) {
        keys[2] = KEY_PRESSED;
    }
    if window.is_key_down(Key::Key4) {
        keys[3] = KEY_PRESSED;
    }
    if window.is_key_down(Key::Q) {
        keys[4] = KEY_PRESSED;
    }
    if window.is_key_down(Key::W) {
        keys[5] = KEY_PRESSED;
    }
    if window.is_key_down(Key::E) {
        keys[6] = KEY_PRESSED;
    }
    if window.is_key_down(Key::R) {
        keys[7] = KEY_PRESSED;
    }
    if window.is_key_down(Key::A) {
        keys[8] = KEY_PRESSED;
    }
    if window.is_key_down(Key::S) {
        keys[9] = KEY_PRESSED;
    }
    if window.is_key_down(Key::D) {
        keys[10] = KEY_PRESSED;
    }
    if window.is_key_down(Key::F) {
        keys[11] = KEY_PRESSED;
    }
    if window.is_key_down(Key::Z) {
        keys[12] = KEY_PRESSED;
    }
    if window.is_key_down(Key::X) {
        keys[13] = KEY_PRESSED;
    }
    if window.is_key_down(Key::C) {
        keys[14] = KEY_PRESSED;
    }
    if window.is_key_down(Key::V) {
        keys[15] = KEY_PRESSED;
    }

    keys
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // TODO error handling
    let matches = App::new("chip8")
        .version("1.0")
        .author("William Guimont-Martin")
        .about("Chip8 emulator written in Rust")
        .arg(Arg::with_name("scale")
            .long("scale")
            .takes_value(true)
            .default_value("10")
            .required(true)
            .help("Render scaling"))
        .arg(Arg::with_name("rom")
            .long("rom")
            .takes_value(true)
            .required(true)
            .help("Rom path"))
        .get_matches();

    let scale: usize = matches.value_of("scale").ok_or("Invalid scale")?
        .parse::<usize>().unwrap();
    let rom_path = matches.value_of("rom").ok_or("No ROM")?;

    let width: usize = DISPLAY_WIDTH * scale;
    let height: usize = DISPLAY_HEIGHT * scale;

    let rom = fs::read(rom_path)?;
    let mut chip = Chip8::new(rom);

    {
        let mut buffer: Vec<u32> = vec![0; width * height];
        buffer[10] = 0xFFFF_FFFF;

        let mut window = Window::new(
            "chip8",
            width,
            height,
            WindowOptions::default(),
        )
            .unwrap_or_else(|e| {
                panic!("{}", e);
            });

        // Limit to max ~60 fps update rate
        window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

        while window.is_open() && !window.is_key_down(Key::Escape) {
            let keys = get_keys(&window);
            chip.set_keypad(keys);

            // TODO make sound
            chip.tick();
            // TODO run multiple steps (according to frequency)
            chip.step()?;
            let display = chip.get_display();

            for i in 0..DISPLAY_WIDTH {
                for j in 0..DISPLAY_HEIGHT {
                    let display_index = get_index(i, j, DISPLAY_WIDTH);
                    let pixel_value = display[display_index];
                    for di in 0..scale {
                        for dj in 0..scale {
                            let buffer_index = get_index(i * scale + di, j * scale + dj, width);
                            buffer[buffer_index] = pixel_value;
                        }
                    }
                }
            }

            window
                .update_with_buffer(&buffer, width, height)
                .unwrap();
        }
    }
    Ok(())
}
