use minifb::{Scale, Window, WindowOptions};

const WHITE_COLOR: u32 = 0xFFFFFF;
const BLACK_COLOR: u32 = 0x000000;
pub struct Display {
    width: usize,
    height: usize,
    buffer: Vec<u8>, // store 0  or 1 (drawn)
    rendering_buffer: Vec<u32>,
    window: Window
}

impl Display {
    pub fn new( title: &str, width: usize, height: usize, scale: Scale) -> Display {
        Display {
            width,
            height,
            buffer: vec![0; width * height],
            rendering_buffer: vec![0xFFFFFF; width * height],
            window: Window::new(title, width, height, WindowOptions {
                scale,
                ..WindowOptions::default()
            }).unwrap(),
        }
    }

    pub fn is_open(&self) -> bool {
        self.window.is_open()
    }

    pub fn clear(&mut self) {
        self.buffer = vec![0; self.width * self.height];
        self.rendering_buffer = vec![0xFFFFFF; self.width * self.height];
    }

    pub fn update(&mut self) {
        self.window.update_with_buffer(&self.rendering_buffer, self.width, self.height).unwrap();
    }

    pub fn draw(&mut self, bytes_to_draw: Vec<u8>, x: u8, y: u8) -> bool {
        let mut is_collisions = false;

        for (row, bytes) in bytes_to_draw.iter().enumerate() {
            for bit in 0..8 {
                let offset = (y as usize + row) * self.width + (x as usize+ bit);
                let pixel: u8 = (bytes >> (7 - bit)) & 1;

                is_collisions = is_collisions || self.draw_pixel(pixel, offset)
            }
        }

        return is_collisions;
    }

    fn draw_pixel(&mut self, pixel: u8, offset: usize) -> bool {
        let existing_pixel = self.buffer[offset];
        let new_pixel = existing_pixel ^ pixel;

        self.buffer[offset] = new_pixel;
        self.rendering_buffer[offset] = if new_pixel == 1 {BLACK_COLOR} else {WHITE_COLOR};

        // return is there a collision or not
        return existing_pixel == 1 && pixel == 1;
    }
}