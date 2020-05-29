extern crate minifb;
extern crate clap;
extern crate math;

use std::fs;
use minifb::{Key, Window, WindowOptions};
use std::thread;
use std::time::{Instant, Duration};
use clap::{Arg, App};
use rodio::{Sink, Source};

use crate::chip8::{Chip8, DISPLAY_WIDTH, DISPLAY_HEIGHT, NUMBER_OF_KEYS, KEY_NOT_PRESSED, KEY_PRESSED, CHIP_FREQUENCY};

mod chip8;

fn get_index(i: usize, j: usize, width: usize) -> usize {
    i + j * width
}

fn get_keys(window: &Window) -> [u8; NUMBER_OF_KEYS] {
    let mut keys= [KEY_NOT_PRESSED; NUMBER_OF_KEYS];

    if window.is_key_down(Key::Key1) {
        keys[0x1] = KEY_PRESSED;
    }
    if window.is_key_down(Key::Key2) {
        keys[0x2] = KEY_PRESSED;
    }
    if window.is_key_down(Key::Key3) {
        keys[0x3] = KEY_PRESSED;
    }
    if window.is_key_down(Key::Key4) {
        keys[0xC] = KEY_PRESSED;
    }
    if window.is_key_down(Key::Q) {
        keys[0x4] = KEY_PRESSED;
    }
    if window.is_key_down(Key::W) {
        keys[0x5] = KEY_PRESSED;
    }
    if window.is_key_down(Key::E) {
        keys[0x6] = KEY_PRESSED;
    }
    if window.is_key_down(Key::R) {
        keys[0xD] = KEY_PRESSED;
    }
    if window.is_key_down(Key::A) {
        keys[0x7] = KEY_PRESSED;
    }
    if window.is_key_down(Key::S) {
        keys[0x8] = KEY_PRESSED;
    }
    if window.is_key_down(Key::D) {
        keys[0x9] = KEY_PRESSED;
    }
    if window.is_key_down(Key::F) {
        keys[0xE] = KEY_PRESSED;
    }
    if window.is_key_down(Key::Z) {
        keys[0xA] = KEY_PRESSED;
    }
    if window.is_key_down(Key::X) {
        keys[0x0] = KEY_PRESSED;
    }
    if window.is_key_down(Key::C) {
        keys[0xB] = KEY_PRESSED;
    }
    if window.is_key_down(Key::V) {
        keys[0xF] = KEY_PRESSED;
    }

    keys
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
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
            WindowOptions::default(), )
            .unwrap_or_else(|e| {
                panic!("{}", e);
            });
        let device = rodio::default_output_device().unwrap();
        let sink = Sink::new(&device);
        let source = rodio::source::SineWave::new(440).repeat_infinite();
        sink.append(source);
        sink.pause();

        // Limit to max ~60 fps update rate
        window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));
        let mut last_time = Instant::now();

        while window.is_open() && !window.is_key_down(Key::Escape) {
            let keys = get_keys(&window);
            chip.set_keypad(keys);

            chip.tick();
            if chip.is_playing_sound() {
                sink.play();
            } else {
                sink.pause();
            }

            {
                let new_time = Instant::now();
                let elaspsed = new_time.duration_since(last_time);
                let num_steps = math::round::floor(elaspsed.as_micros() as f64 / 1_000_000.0 * CHIP_FREQUENCY, 0) as u128;
                for _ in 0..num_steps {
                    chip.step()?;
                }
                last_time = Instant::now();
            }

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
            thread::sleep(Duration::from_micros(16600));
        }
    }
    Ok(())
}
