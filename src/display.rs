pub struct Display {
    framebuffer: [bool; 64 * 32],
}

impl Display {
    pub fn new() -> Self {
        Display {
            framebuffer: [false; 64 * 32],
        }
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, value: bool) -> bool {
        let index = y * 64 + x;
        let collision = self.framebuffer[index] && value;
        self.framebuffer[index] ^= value;
        collision
    }

    pub fn get_pixel(&mut self, x: usize, y: usize) -> bool {
        let index = y * 64 + x;
        self.framebuffer[index]
    }

    pub fn clear(&mut self) {
        self.framebuffer = [false; 64 * 32];
    }
}
