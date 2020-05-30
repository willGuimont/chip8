use rand::prelude::*;

pub const CHIP_FREQUENCY: f64 = 500.0;

const NUMBER_OF_REGISTER: usize = 16;

const MEMORY_SIZE: usize = 0xFFF;
const START_PROGRAM_SPACE: usize = 0x200;

const STACK_SIZE: usize = 16;

pub const NUMBER_OF_KEYS: usize = 16;
pub const KEY_PRESSED: u8 = 0xFF;
pub const KEY_NOT_PRESSED: u8 = 0x00;

pub const DISPLAY_WIDTH: usize = 64;
pub const DISPLAY_HEIGHT: usize = 32;
pub const DISPLAY_SIZE: usize = DISPLAY_WIDTH * DISPLAY_HEIGHT;
pub const PIXEL_ON: u32 = 0xFFFF_FFFF;
pub const PIXEL_OFF: u32 = 0;

const FONT_SET_ADDRESS_START: usize = 0x050;
const NUMBER_OF_CHARACTERS: usize = 16;
const NUMBER_OF_BYTES_PER_CHARACTER: usize = 5;
const FONT_SET_SIZE: usize = NUMBER_OF_CHARACTERS * NUMBER_OF_BYTES_PER_CHARACTER;
const FONT_SET: [u8; FONT_SET_SIZE] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];

/*
 * nnn => lowest 12 bits of instruction
 * n   => lowest 4 bits of instruction
 * x   => lower 4 bits of high byte of instruction
 * y   => upper 4 bits of low byte of instruction
 * kk  => lowest byte of instruction
 */
#[derive(Debug)]
enum Instruction {
    // 00E0 Clear display
    Clear,
    // 00EE Return from subroutine, set PC=top stack, sp -= 1
    Return,
    // 1nnn Jump to location nnn
    Jump(u16),
    // 2nnn Call subroutune at location nnn
    Call(u16),
    // 3xkk Skip next instruction if Vx = kk (PC += 2)
    SkipNextIfEqualByte(usize, u8),
    // 4xkk Skip next instruction if Vx != kk (PC += 2)
    SkipNextIfNotEqualByte(usize, u8),
    // 5xy0 Skip next instruction if Vx = Vy (PC += 2)
    SkipNextIfEqualRegister(usize, usize),
    // 6xkk Set Vx = kk
    LoadByte(usize, u8),
    // 7xkk Set Vx = Vx + kk
    AddByte(usize, u8),
    // 8xy0 Set Vx += Vy
    LoadRegister(usize, usize),
    // 8xy1 Set V[x] = Vx OR Vy (bitwise)
    Or(usize, usize),
    // 8xy2 Set Vx = Vx AND Vy (bitwise)
    And(usize, usize),
    // 8xy3 Set V[x] = Vx XOR Vy (bitwise)
    Xor(usize, usize),
    // 8xy4 Set Vx = Vx + Vy, set VF = carry
    AddRegister(usize, usize),
    // 8xy5 Set Vx = Vx - Vy, VF = NOT borrow (Vx > Vy, then VF is set to 1, otherwise 0)
    Sub(usize, usize),
    // 8xy6 CHIP-48: If the least-significant bit of Vx is 1, then VF is set to 1, otherwise 0. Then Vx is divided by 2
    // This opcode has multiple possible implementation (it was undocumented in CHIP-8). The CHIP-48 implementation is chosen
    ShiftRight(usize, usize),
    // 9xy7 Set Vx = Vy - Vx, set VF = NOT borrow. If Vy > Vx, then VF is set to 1, otherwise 0
    SubFrom(usize, usize),
    // 8xyE If the most-significant bit of Vx is 1, then VF is set to 1, otherwise to 0. Then Vx is multiplied by 2.
    // This opcode has multiple possible implementation (it was undocumented in CHIP-8). The CHIP-48 implementation is chosen
    ShiftLeft(usize, usize),
    // 9xy0 Skip next instruction if Vx != Vy (PC += 2)
    SkipNextIfNotEqualRegister(usize, usize),
    // Annn Set I = nnn
    SetIndex(u16),
    // Bnnn Jump to location nnn + V0
    JumpOf(u16),
    // Cxkk Set Vx = random byte(0-255) AND kk
    Random(usize, u8),
    /*
     * Dxyn Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision.
     *
     * The interpreter reads n bytes from memory,
     * starting at the address stored in I.
     * These bytes are then displayed as sprites on screen at coordinates (Vx, Vy).
     * Sprites are XORed onto the existing screen.
     * If this causes any pixels to be erased, VF is set to 1,
     * otherwise it is set to 0. If the sprite is positioned so part of it is
     * outside the coordinates of the display, it wraps around to the opposite side of the screen.
     */
    DisplaySprite(usize, usize, u8),
    // Ex9E Skip next instruction if key with the value of Vx is pressed. (PX += 2)
    SkipIfKeyPressed(usize),
    // ExA1 Skip next instruction if key with the value of Vx is not pressed.
    SkipIfNotKeyPressed(usize),
    // Fx07 Set Vx = delay timer value.
    LoadTimer(usize),
    // Fx0A Wait for a key press, store the value of the key in Vx.
    WaitKeyPress(usize),
    // Fx15 Set delay timer = Vx.
    SetTimer(usize),
    // Fx18 Set sound timer = Vx.
    SetSoundTimer(usize),
    // Fx1E Set I = I + Vx.
    AddIndex(usize),
    // Fx29 Set I = location of sprite for digit Vx.
    LoadSpriteLocationIndex(usize),
    // Fx33 Store BCD representation of Vx in memory locations I, I+1, and I+2.
    BinaryCodedDecimal(usize),
    // Fx55 Store registers V0 through Vx in memory starting at location I.
    StoreRegisters(usize),
    // Fx65 Read registers V0 through Vx from memory starting at location I.
    ReadRegisters(usize),
}

pub struct Chip8 {
    // Registers
    registers: [u8; NUMBER_OF_REGISTER],
    // 0x000 - 0x1FF reserved for interpreter
    // | 0x50-0xA0 16 characters 0 to F
    // 0x200 - 0xFFF Program / Data Space
    memory: [u8; MEMORY_SIZE],
    // Index register
    index: u16,
    // Program counter
    program_counter: u16,
    // Stack
    stack: [u16; STACK_SIZE],
    // Index of the top of the stack
    stack_pointer: usize,
    // Decrease (if non-zero) at rate of 60 Hz
    delay_timer: u8,
    // Decrease (if non-zero) at rate of 60 Hz
    // Play sound while non-zero
    sound_timer: u8,
    /*
     * Keypad
     * 1 2 3 C
     * 4 5 6 D
     * 7 8 9 E
     * A 0 B F
     * Keyboard
     * 1 2 3 4
     * q w e r
     * a s d f
     * z x c v
     */
    keypad: [u8; NUMBER_OF_KEYS],
    display: [u32; DISPLAY_SIZE],
}

impl Chip8 {
    pub fn new(rom: Vec<u8>) -> Self {
        let mut memory = [0u8; MEMORY_SIZE];
        for (i, font_data) in FONT_SET.iter().enumerate() {
            memory[FONT_SET_ADDRESS_START + i] = *font_data;
        }
        for (i, rom_data) in rom.iter().enumerate() {
            memory[START_PROGRAM_SPACE + i] = *rom_data;
        }

        Chip8 {
            registers: [0u8; NUMBER_OF_REGISTER],
            memory,
            index: 0,
            program_counter: START_PROGRAM_SPACE as u16,
            stack: [0; STACK_SIZE],
            stack_pointer: 0,
            delay_timer: 0,
            sound_timer: 0,
            keypad: [0; NUMBER_OF_KEYS],
            display: [0; DISPLAY_SIZE],
        }
    }

    pub fn get_display(&self) -> [u32; DISPLAY_SIZE] {
        self.display
    }

    pub fn tick(&mut self) {
        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
    }

    pub fn step(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let opcode = self.fetch();
        let instruction = Self::decode(opcode).ok_or(format!("invalid opcode {:X}", opcode))?;
        self.execute(instruction);
        Ok(())
    }

    pub fn set_keypad(&mut self, keys: [u8; 16]) {
        self.keypad = keys;
    }

    pub fn is_playing_sound(&self) -> bool {
        self.sound_timer > 0
    }

    fn fetch(&mut self) -> u16 {
        let higher_byte = self.memory[self.program_counter as usize];
        let lower_byte = self.memory[(self.program_counter + 1) as usize];
        self.program_counter += 2;

        ((higher_byte as u16) << 8) | (lower_byte as u16)
    }

    fn decode(opcode: u16) -> Option<Instruction> {
        let nnn = Self::get_nnn(opcode);
        let n = Self::get_n(opcode);
        let x = Self::get_x(opcode);
        let y = Self::get_y(opcode);
        let kk = Self::get_kk(opcode);
        match opcode {
            0x00E0 => Some(Instruction::Clear),
            0x00EE => Some(Instruction::Return),
            0x1000..=0x1FFF => Some(Instruction::Jump(nnn)),
            0x2000..=0x2FFF => Some(Instruction::Call(nnn)),
            0x3000..=0x3FFF => Some(Instruction::SkipNextIfEqualByte(x, kk)),
            0x4000..=0x4FFF => Some(Instruction::SkipNextIfNotEqualByte(x, kk)),
            0x5000..=0x5FFF => {
                if opcode.trailing_zeros() >= 4 {
                    Some(Instruction::SkipNextIfEqualRegister(x, y))
                } else {
                    None
                }
            }
            0x6000..=0x6FFF => Some(Instruction::LoadByte(x, kk)),
            0x7000..=0x7FFF => Some(Instruction::AddByte(x, kk)),
            0x8000..=0x8FFF => {
                match opcode & 0xF {
                    0x0 => Some(Instruction::LoadRegister(x, y)),
                    0x1 => Some(Instruction::Or(x, y)),
                    0x2 => Some(Instruction::And(x, y)),
                    0x3 => Some(Instruction::Xor(x, y)),
                    0x4 => Some(Instruction::AddRegister(x, y)),
                    0x5 => Some(Instruction::Sub(x, y)),
                    0x6 => Some(Instruction::ShiftRight(x, y)),
                    0x7 => Some(Instruction::SubFrom(x, y)),
                    0xE => Some(Instruction::ShiftLeft(x, y)),
                    _ => None
                }
            }
            0x9000..=0x9FFF => {
                if opcode.trailing_zeros() >= 4 {
                    Some(Instruction::SkipNextIfNotEqualRegister(x, y))
                } else {
                    None
                }
            }
            0xA000..=0xAFFF => Some(Instruction::SetIndex(nnn)),
            0xB000..=0xBFFF => Some(Instruction::JumpOf(nnn)),
            0xC000..=0xCFFF => Some(Instruction::Random(x, kk)),
            0xD000..=0xDFFF => Some(Instruction::DisplaySprite(x, y, n)),
            0xE000..=0xEFFF => {
                match opcode & 0xFF {
                    0x9E => Some(Instruction::SkipIfKeyPressed(x)),
                    0xA1 => Some(Instruction::SkipIfNotKeyPressed(x)),
                    _ => None
                }
            }
            0xF000..=0xFFFF => {
                match opcode & 0xFF {
                    0x07 => Some(Instruction::LoadTimer(x)),
                    0x0A => Some(Instruction::WaitKeyPress(x)),
                    0x15 => Some(Instruction::SetTimer(x)),
                    0x18 => Some(Instruction::SetSoundTimer(x)),
                    0x1E => Some(Instruction::AddIndex(x)),
                    0x29 => Some(Instruction::LoadSpriteLocationIndex(x)),
                    0x33 => Some(Instruction::BinaryCodedDecimal(x)),
                    0x55 => Some(Instruction::StoreRegisters(x)),
                    0x65 => Some(Instruction::ReadRegisters(x)),
                    _ => None
                }
            }
            _ => None
        }
    }

    fn get_nnn(opcode: u16) -> u16 {
        opcode & 0xFFF
    }

    fn get_n(opcode: u16) -> u8 {
        (opcode & 0xF) as u8
    }

    fn get_x(opcode: u16) -> usize {
        ((opcode >> 8) & 0xF) as usize
    }

    fn get_y(opcode: u16) -> usize {
        ((opcode >> 4) & 0xF) as usize
    }

    fn get_kk(opcode: u16) -> u8 {
        (opcode & 0xFF) as u8
    }

    fn execute(&mut self, instruction: Instruction) {
        match instruction {
            Instruction::Clear => {
                self.display = [0; DISPLAY_SIZE];
            }
            Instruction::Return => {
                self.program_counter = self.stack[self.stack_pointer];
                self.stack_pointer -= 1;
            }
            Instruction::Jump(nnn) => {
                self.program_counter = nnn;
            }
            Instruction::Call(nnn) => {
                self.stack_pointer += 1;
                self.stack[self.stack_pointer] = self.program_counter;
                self.program_counter = nnn;
            }
            Instruction::SkipNextIfEqualByte(x, kk) => {
                if self.registers[x] == kk {
                    self.program_counter += 2;
                }
            }
            Instruction::SkipNextIfNotEqualByte(x, kk) => {
                if self.registers[x] != kk {
                    self.program_counter += 2;
                }
            }
            Instruction::SkipNextIfEqualRegister(x, y) => {
                if self.registers[x] == self.registers[y] {
                    self.program_counter += 2;
                }
            }
            Instruction::LoadByte(x, kk) => {
                self.registers[x] = kk;
            }
            Instruction::AddByte(x, kk) => {
                self.registers[x] = self.registers[x].wrapping_add(kk);
            }
            Instruction::LoadRegister(x, y) => {
                self.registers[x] = self.registers[y];
            }
            Instruction::Or(x, y) => {
                self.registers[x] |= self.registers[y];
            }
            Instruction::And(x, y) => {
                self.registers[x] &= self.registers[y];
            }
            Instruction::Xor(x, y) => {
                self.registers[x] ^= self.registers[y];
            }
            Instruction::AddRegister(x, y) => {
                let x_value = self.registers[x] as u16;
                let y_value = self.registers[y] as u16;
                let sum = x_value + y_value;
                self.registers[0xF] = if sum > 255 {
                    1
                } else {
                    0
                };
                self.registers[x] = sum as u8;
            }
            Instruction::Sub(x, y) => {
                let x_value = self.registers[x];
                let y_value = self.registers[y];

                self.registers[0xF] = if x_value > y_value {
                    1
                } else {
                    0
                };

                self.registers[x] = x_value.wrapping_sub(y_value);
            }
            Instruction::ShiftRight(x, _y) => {
                let value = self.registers[x];

                self.registers[0xF] = if value & 0b1 > 0 {
                    1
                } else {
                    0
                };

                self.registers[x] = value >> 1;
            }
            Instruction::SubFrom(x, y) => {
                let x_value = self.registers[x];
                let y_value = self.registers[y];

                self.registers[0xF] = if y_value > x_value {
                    1
                } else {
                    0
                };

                self.registers[x] = y_value.wrapping_sub(x_value);
            }
            Instruction::ShiftLeft(x, _y) => {
                let value = self.registers[x];

                self.registers[0xF] = if value & 0x80 > 0 {
                    1
                } else {
                    0
                };

                self.registers[x] = value << 1;
            }
            Instruction::SkipNextIfNotEqualRegister(x, y) => {
                if self.registers[x] != self.registers[y] {
                    self.program_counter += 2;
                }
            }
            Instruction::SetIndex(nnn) => {
                self.index = nnn;
            }
            Instruction::JumpOf(nnn) => {
                self.program_counter = nnn + self.registers[0] as u16;
            }
            Instruction::Random(x, kk) => {
                let rng = rand::thread_rng().next_u32() as u8;
                self.registers[x] = rng & kk;
            }
            Instruction::DisplaySprite(x, y, n) => {
                let vx = self.registers[x] as usize;
                let vy = self.registers[y] as usize;
                let get_index = |i, j| { ((vx + i) % DISPLAY_WIDTH) + ((vy + j) % DISPLAY_HEIGHT) * DISPLAY_WIDTH };

                let mut collided = false;
                let index = self.index as usize;
                for j in 0..n as usize {
                    let mut mask = 0x80;
                    let sprite_line = self.memory[index + j];
                    for i in 0..8 as usize {
                        let pixel_value = if sprite_line & mask > 0 {
                            PIXEL_ON
                        } else {
                            PIXEL_OFF
                        };
                        let pre = self.display[get_index(i, j)];
                        self.display[get_index(i, j)] ^= pixel_value;
                        if pre == PIXEL_ON && self.display[get_index(i, j)] == PIXEL_OFF {
                            collided = true;
                        }
                        mask >>= 1;
                    }
                }
                self.registers[0xF] = if collided {
                    1
                } else {
                    0
                }
            }
            Instruction::SkipIfKeyPressed(x) => {
                let key = self.registers[x] as usize;
                if self.keypad[key] == KEY_PRESSED {
                    self.program_counter += 2;
                }
            }
            Instruction::SkipIfNotKeyPressed(x) => {
                let key = self.registers[x] as usize;
                if self.keypad[key] == KEY_NOT_PRESSED {
                    self.program_counter += 2;
                }
            }
            Instruction::LoadTimer(x) => {
                self.registers[x] = self.delay_timer;
            }
            Instruction::WaitKeyPress(x) => {
                for (k, status) in self.keypad.iter().enumerate() {
                    if *status == KEY_PRESSED {
                        self.registers[x] = k as u8;
                        return;
                    }
                }

                self.program_counter -= 2;
            }
            Instruction::SetTimer(x) => {
                self.delay_timer = self.registers[x];
            }
            Instruction::SetSoundTimer(x) => {
                self.sound_timer = self.registers[x];
            }
            Instruction::AddIndex(x) => {
                self.index += self.registers[x] as u16;
            }
            Instruction::LoadSpriteLocationIndex(x) => {
                let vx = self.registers[x];
                self.index = (FONT_SET_ADDRESS_START + (vx as usize) * NUMBER_OF_BYTES_PER_CHARACTER) as u16;
            }
            Instruction::BinaryCodedDecimal(x) => {
                let vx: Vec<char> = format!("{:0>3}", self.registers[x]).chars().collect();
                let hundreds: u8 = vx[0].to_string().parse().unwrap();
                let tens: u8 = vx[1].to_string().parse().unwrap();
                let ones: u8 = vx[2].to_string().parse().unwrap();

                let index = self.index as usize;
                self.memory[index] = hundreds;
                self.memory[index + 1] = tens;
                self.memory[index + 2] = ones;
            }
            Instruction::StoreRegisters(x) => {
                for i in 0..=x as usize {
                    self.memory[(self.index as usize) + i] = self.registers[i];
                }
            }
            Instruction::ReadRegisters(x) => {
                for i in 0..=x as usize {
                    self.registers[i] = self.memory[(self.index as usize) + i];
                }
            }
        };
    }
}
