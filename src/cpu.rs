use std::fs;

pub struct Cpu {
    pub registers: [u8; 16],
    pub index_register: u16,
    pub program_counter: u16,
    pub memory: [u8; 4096],
    pub level_stack: [u16; 16],
    pub stack_pointer: u8,
    pub delay_timer: u8,
    pub sound_timer: u8,
}

impl Cpu {
    pub fn new() -> Self {
        Cpu {
            registers: [0; 16],
            index_register: 0,
            program_counter: 0x200,
            memory: [0; 4096],
            level_stack: [0; 16],
            stack_pointer: 0,
            delay_timer: 0,
            sound_timer: 0,
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
                    0x0 => self.op_ld(opcode),
                    0x1 => self.op_or(opcode),
                    0x2 => self.op_and(opcode),
                    _ => eprintln!("Unknown opcode: {:04X}", opcode),
                }
            }
            _ => eprint!("Unknown opcode : {:04X}", opcode),
        }
    }

    fn op_cls(&mut self) {}

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
}
