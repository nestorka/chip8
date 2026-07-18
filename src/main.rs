use minifb::{Key, Window, WindowOptions};

mod cpu;
mod display;

const WIDTH: usize = 64;
const HEIGHT: usize = 32;
const SCALE: usize = 10;

fn main() {
    let mut window = Window::new(
        "Chip-8",
        WIDTH * SCALE,
        HEIGHT * SCALE,
        WindowOptions::default(),
    )
    .unwrap();

    let mut cpu = cpu::Cpu::new();
    cpu.load_rom("roms/IBM Logo.ch8").unwrap();

    let mut buffer: Vec<u32> = vec![0; WIDTH * SCALE * HEIGHT * SCALE];

    while window.is_open() && !window.is_key_down(Key::Escape) {
        cpu.update_keys(&window.get_keys());
        let opcode = cpu.fetch();
        cpu.execute(opcode);

        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let pixel = cpu.display.get_pixel(x, y);
                let color = if pixel { 0xFFFFFF } else { 0x000000 };

                for row in 0..SCALE {
                    for col in 0..SCALE {
                        buffer[(y * SCALE + row) * WIDTH * SCALE + (x * SCALE + col)] = color;
                    }
                }
            }
        }

        window
            .update_with_buffer(&buffer, WIDTH * SCALE, HEIGHT * SCALE)
            .unwrap();
    }
}
