use crate::display::Display;
use rand::Rng;
use std::fs;

const FONTSET: [u8; 80] = [
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
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

pub struct Cpu {
    pub registers: [u8; 16],
    pub index_register: u16,
    pub program_counter: u16,
    pub memory: [u8; 4096],
    pub level_stack: [u16; 16],
    pub stack_pointer: u8,
    pub delay_timer: u8,
    pub sound_timer: u8,
    pub display: Display,
    pub keys: [bool; 16],
}

impl Cpu {
    pub fn new() -> Self {
        let mut memory = [0u8; 4096];
        for (i, &byte) in FONTSET.iter().enumerate() {
            memory[i] = byte;
        }

        Cpu {
            registers: [0; 16],
            index_register: 0,
            program_counter: 0x200,
            memory,
            level_stack: [0; 16],
            stack_pointer: 0,
            delay_timer: 0,
            sound_timer: 0,
            display: Display::new(),
            keys: [false; 16],
        }
    }

    pub fn load_rom(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let file = fs::read(path)?;

        if self.program_counter as usize + file.len() >= 4096 {
            return Err("Out of memory".into());
        }

        for (j, c) in file.into_iter().enumerate() {
            self.memory[j + self.program_counter as usize] = c;
        }

        Ok(())
    }

    pub fn fetch(&mut self) -> u16 {
        let opcode = ((self.memory[self.program_counter as usize] as u16) << 8)
            | self.memory[self.program_counter as usize + 1] as u16;
        self.program_counter += 2;
        opcode
    }

    pub fn execute(&mut self, opcode: u16) {
        match opcode {
            0x00E0 => self.op_cls(),
            0x00EE => self.op_ret(),
            0x1000..=0x1FFF => self.op_jp(opcode),
            0x2000..=0x2FFF => self.op_call(opcode),
            0x3000..=0x3FFF => self.op_se(opcode),
            0x4000..=0x4FFF => self.op_sne(opcode),
            0x5000..=0x5FFF => self.op_se_regs(opcode),
            0x6000..=0x6FFF => self.op_ld(opcode),
            0x7000..=0x7FFF => self.op_add(opcode),
            0x8000..=0x8FFF => {
                let last_nibble = opcode & 0x0F;
                match last_nibble {
                    0x0 => self.op_ld_regs(opcode),
                    0x1 => self.op_or(opcode),
                    0x2 => self.op_and(opcode),
                    0x3 => self.op_xor(opcode),
                    0x4 => self.op_add_regs(opcode),
                    0x5 => self.op_sub(opcode),
                    0x6 => self.op_shr(opcode),
                    0x7 => self.op_subn(opcode),
                    0xE => self.op_shl(opcode),
                    _ => eprintln!("Unknown opcode: {:04X}", opcode),
                }
            }
            0x9000..=0x9FFF => self.op_sne_regs(opcode),
            0xA000..=0xAFFF => self.op_ldi(opcode),
            0xB000..=0xBFFF => self.op_jp_v0(opcode),
            0xC000..=0xCFFF => self.op_rnd(opcode),
            0xD000..=0xDFFF => self.op_drw(opcode),
            0xE000..=0xEFFF => {
                let last_byte = opcode & 0x00FF;
                match last_byte {
                    0x9E => self.op_skp(opcode),
                    0xA1 => self.op_sknp(opcode),
                    _ => eprintln!("Unknown opcode: {:04X}", opcode),
                }
            }
            0xF000..=0xFFFF => {
                let last_byte = opcode & 0x0FF;
                match last_byte {
                    0x07 => self.op_ldxdt(opcode),
                    0x0A => self.op_ldxk(opcode),
                    0x15 => self.op_lddt(opcode),
                    0x18 => self.op_ldst(opcode),
                    0x1E => self.op_addi(opcode),
                    0x29 => self.op_ldf(opcode),
                    0x33 => self.op_lbd(opcode),
                    0x55 => self.op_ldix(opcode),
                    0x65 => self.op_ldxi(opcode),
                    _ => eprintln!("Unknown opcode: {:04X}", opcode),
                }
            }
            _ => eprint!("Unknown opcode : {:04X}", opcode),
        }
    }

    pub fn update_keys(&mut self, keys: &[minifb::Key]) {
        self.keys = [false; 16];
        for key in keys {
            let index = match key {
                minifb::Key::X => Some(0x0),
                minifb::Key::Key1 => Some(0x1),
                minifb::Key::Key2 => Some(0x2),
                minifb::Key::Key3 => Some(0x3),
                minifb::Key::Q => Some(0x4),
                minifb::Key::W => Some(0x5),
                minifb::Key::E => Some(0x6),
                minifb::Key::A => Some(0x7),
                minifb::Key::S => Some(0x8),
                minifb::Key::D => Some(0x9),
                minifb::Key::Z => Some(0xA),
                minifb::Key::C => Some(0xB),
                minifb::Key::Key4 => Some(0xC),
                minifb::Key::R => Some(0xD),
                minifb::Key::F => Some(0xE),
                minifb::Key::V => Some(0xF),
                _ => None,
            };
            if let Some(i) = index {
                self.keys[i] = true;
            }
        }
    }

    fn op_cls(&mut self) {
        self.display.clear();
    }

    fn op_call(&mut self, opcode: u16) {
        self.level_stack[self.stack_pointer as usize] = self.program_counter;
        self.stack_pointer += 1;
        self.program_counter = opcode & 0x0FFF;
    }

    fn op_ret(&mut self) {
        self.stack_pointer -= 1;
        self.program_counter = self.level_stack[self.stack_pointer as usize];
    }

    fn op_jp(&mut self, opcode: u16) {
        self.program_counter = opcode & 0x0FFF;
    }

    fn op_se(&mut self, opcode: u16) {
        let x = (opcode >> 8) & 0x0F;
        let kk = opcode & 0xFF;

        if self.registers[x as usize] == kk as u8 {
            self.program_counter += 2;
        }
    }

    fn op_sne(&mut self, opcode: u16) {
        let x = (opcode >> 8) & 0x0F;
        let kk = opcode & 0xFF;

        if self.registers[x as usize] != kk as u8 {
            self.program_counter += 2;
        }
    }

    fn op_se_regs(&mut self, opcode: u16) {
        let x = (opcode >> 8) & 0x0F;
        let y = (opcode >> 4) & 0x0F;

        if self.registers[x as usize] == self.registers[y as usize] {
            self.program_counter += 2;
        }
    }

    fn op_add(&mut self, opcode: u16) {
        let x = (opcode >> 8) & 0x0F;
        let kk = opcode & 0xFF;
        self.registers[x as usize] = self.registers[x as usize].wrapping_add(kk as u8);
    }

    fn op_ld(&mut self, opcode: u16) {
        let x = (opcode >> 8) & 0x0F;
        let kk = opcode & 0xFF;
        self.registers[x as usize] = kk as u8;
    }

    fn op_ld_regs(&mut self, opcode: u16) {
        let x = (opcode >> 8) & 0x0F;
        let y = (opcode >> 4) & 0x0F;
        self.registers[x as usize] = self.registers[y as usize];
    }

    fn op_or(&mut self, opcode: u16) {
        let x = (opcode >> 8) & 0x0F;
        let y = (opcode >> 4) & 0x0F;

        self.registers[x as usize] |= self.registers[y as usize];
    }

    fn op_and(&mut self, opcode: u16) {
        let x = (opcode >> 8) & 0x0F;
        let y = (opcode >> 4) & 0x0F;

        self.registers[x as usize] &= self.registers[y as usize];
    }

    fn op_xor(&mut self, opcode: u16) {
        let x = (opcode >> 8) & 0x0F;
        let y = (opcode >> 4) & 0x0F;

        self.registers[x as usize] ^= self.registers[y as usize];
    }

    fn op_add_regs(&mut self, opcode: u16) {
        let x = (opcode >> 8) & 0x0F;
        let y = (opcode >> 4) & 0x0F;

        let (result, overflow) =
            self.registers[x as usize].overflowing_add(self.registers[y as usize]);
        self.registers[x as usize] = result;
        self.registers[0xF] = overflow as u8;
    }

    fn op_sub(&mut self, opcode: u16) {
        let x = (opcode >> 8) & 0x0F;
        let y = (opcode >> 4) & 0x0F;

        let (result, overflow) =
            self.registers[x as usize].overflowing_sub(self.registers[y as usize]);
        self.registers[x as usize] = result;
        self.registers[0xF] = !overflow as u8;
    }

    fn op_shr(&mut self, opcode: u16) {
        let x = (opcode >> 8) & 0x0F;

        self.registers[0xF] = self.registers[x as usize] & 0x1;
        self.registers[x as usize] >>= 1;
    }

    fn op_subn(&mut self, opcode: u16) {
        let x = (opcode >> 8) & 0x0F;
        let y = (opcode >> 4) & 0x0F;

        let (result, overflow) =
            self.registers[y as usize].overflowing_sub(self.registers[x as usize]);
        self.registers[x as usize] = result;
        self.registers[0xF] = !overflow as u8;
    }

    fn op_shl(&mut self, opcode: u16) {
        let x = (opcode >> 8) & 0x0F;
        self.registers[0xF] = (self.registers[x as usize] >> 7) & 0x1;
        self.registers[x as usize] <<= 1;
    }

    fn op_sne_regs(&mut self, opcode: u16) {
        let x = (opcode >> 8) & 0x0F;
        let y = (opcode >> 4) & 0x0F;

        if self.registers[x as usize] != self.registers[y as usize] {
            self.program_counter += 2;
        }
    }

    fn op_ldi(&mut self, opcode: u16) {
        self.index_register = opcode & 0x0FFF;
    }

    fn op_jp_v0(&mut self, opcode: u16) {
        self.program_counter = (opcode & 0x0FFF) + self.registers[0] as u16;
    }

    fn op_rnd(&mut self, opcode: u16) {
        let kk = opcode & 0xFF;
        let x = (opcode >> 8) & 0x0F;

        let random: u8 = rand::thread_rng().gen_range(0..=255);

        self.registers[x as usize] = kk as u8 & random;
    }

    fn op_drw(&mut self, opcode: u16) {
        let x = self.registers[((opcode >> 8) & 0x0F) as usize] as u16;
        let y = self.registers[((opcode >> 4) & 0x0F) as usize] as u16;
        let nibble = opcode & 0x0F;

        for byte_index in 0..nibble {
            let byte = self.memory[self.index_register as usize + byte_index as usize];
            for bit_position in 0..8 {
                let bit = (byte >> (7 - bit_position)) & 0x1 == 1;

                let collision = self.display.set_pixel(
                    ((x + bit_position) % 64) as usize,
                    ((y + byte_index) % 32) as usize,
                    bit,
                );

                if collision {
                    self.registers[0xF] = 1;
                }
            }
        }
    }

    fn op_skp(&mut self, opcode: u16) {
        let x = ((opcode >> 8) & 0x0F) as usize;
        if self.keys[self.registers[x] as usize] {
            self.program_counter += 2;
        }
    }

    fn op_sknp(&mut self, opcode: u16) {
        let x = ((opcode >> 8) & 0x0F) as usize;
        if !self.keys[self.registers[x] as usize] {
            self.program_counter += 2;
        }
    }

    fn op_ldxdt(&mut self, opcode: u16) {
        let x = ((opcode >> 8) & 0x0F) as usize;
        self.registers[x] = self.delay_timer;
    }

    fn op_ldxk(&mut self, opcode: u16) {
        let x = ((opcode >> 8) & 0x0F) as usize;

        for (i, &pressed) in self.keys.iter().enumerate() {
            if pressed {
                self.registers[x] = i as u8;
                return;
            }
        }

        //repeat if no key is pressed
        self.program_counter -= 2;
    }

    fn op_lddt(&mut self, opcode: u16) {
        let x = ((opcode >> 8) & 0x0F) as usize;
        self.delay_timer = self.registers[x];
    }

    fn op_ldst(&mut self, opcode: u16) {
        let x = ((opcode >> 8) & 0x0F) as usize;
        self.sound_timer = self.registers[x];
    }

    fn op_addi(&mut self, opcode: u16) {
        let x = ((opcode >> 8) & 0x0F) as usize;
        self.index_register = self.index_register.wrapping_add(self.registers[x] as u16);
    }

    fn op_ldf(&mut self, opcode: u16) {
        let x = ((opcode >> 8) & 0x0F) as usize;
        self.index_register = (self.registers[x] as u16) * 5;
    }

    fn op_lbd(&mut self, opcode: u16) {
        let x = ((opcode >> 8) & 0x0F) as usize;
        let value = self.registers[x];

        self.memory[self.index_register as usize] = value / 100;
        self.memory[self.index_register as usize + 1] = (value / 10) % 10;
        self.memory[self.index_register as usize + 2] = value % 10;
    }

    fn op_ldix(&mut self, opcode: u16) {
        let x = ((opcode >> 8) & 0x0F) as usize;

        for i in 0..=x {
            self.memory[self.index_register as usize + i] = self.registers[i];
        }
    }

    fn op_ldxi(&mut self, opcode: u16) {
        let x = ((opcode >> 8) & 0x0F) as usize;

        for i in 0..=x {
            self.registers[i] = self.memory[self.index_register as usize + i];
        }
    }
}
