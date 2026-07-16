mod cpu;

fn main() {
    let mut cpu = cpu::Cpu::new();
    cpu.load_rom("roms/IBM Logo.ch8").unwrap();

    loop {
        let opcode = cpu.fetch();
        cpu.execute(opcode);
    }
}
