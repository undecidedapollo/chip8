use std::cell::RefCell;

const SCREEN_WIDTH: u8 = 64;
const SCREEN_HEIGHT: u8 = 32;
// 1 bit so 64 * 32 / 8 (1 byte = 8 pixels horizontally)
const SCREEN_BUFFER_SIZE_FULL: usize = (SCREEN_WIDTH as usize) * (SCREEN_HEIGHT as usize);
const SCREEN_BUFFER_SIZE_COMPRESSED: usize = SCREEN_BUFFER_SIZE_FULL / 8;

pub trait Chip8Screen {
    fn draw_sprite(&self, x: u8, y: u8, sprite: &[u8]) -> bool;
    fn clear(&self);
}

pub struct Screen {
    pub buffer: Box<RefCell<[u8; SCREEN_BUFFER_SIZE_COMPRESSED]>>,
    pub pending_draw: RefCell<bool>,
}

impl Screen {
    pub fn new() -> Screen {
        let screen = Screen {
            buffer: Box::new(RefCell::new([0; SCREEN_BUFFER_SIZE_COMPRESSED])),
            pending_draw: RefCell::new(false),
        };
        return screen;
    }

    pub fn mark_drawn(&self) {
        self.pending_draw.replace(false);
    }

    pub fn is_pending_draw(&self) -> bool {
        return self.pending_draw.borrow().clone();
    }

    pub fn draw_as_string(&self) -> String {
        let mut str = String::with_capacity(SCREEN_BUFFER_SIZE_FULL + SCREEN_HEIGHT as usize); // Add extra space for the newline
        let buffer = self.buffer.borrow();
        for y in 0_usize..32 {
            for x in 0_usize..64 {
                let val = buffer[(y * 64 + x) / 8];
                let bit = usize::from(x) % 8;
                let mask = 1 << 7 - bit;
                let val = val & mask != 0;
                str.push(if val { '█' } else { ' ' });
            }
            str.push('\n');
        }
        return str;
    }
}
impl Chip8Screen for Screen {
    // Each row is a byte, with each bit representing a pixel, this is the same as the buffer
    fn draw_sprite(&self, x: u8, y: u8, sprite: &[u8]) -> bool {
        self.pending_draw.replace(true);
        let x = x % SCREEN_WIDTH;
        let y: u16 = (y as u16 % SCREEN_HEIGHT as u16) * SCREEN_WIDTH as u16;
        let mut was_unset = false;
        let mut buffer = self.buffer.borrow_mut();
        for row in 0..sprite.len() {
            for bit in 0..8 {
                let row_offset = (row as usize * SCREEN_WIDTH as usize) as usize;
                let index = (usize::from(y) + row_offset + usize::from(x + bit)) / 8;
                let bit_offset = usize::from(x + bit) % 8;
                let mask = 1 << (7 - bit_offset);

                if index >= buffer.len() {
                    return false;
                }
                let val_before = buffer[index] & mask != 0;
                let sprite_mask = 1 << (7 - bit);
                let sprite_val = (sprite[row as usize] & sprite_mask) >> (7 - bit);
                let sprite_adjusted = sprite_val << (7 - bit_offset);
                // println!(
                //     "x: {}, y: {}, row: {}, bit: {}, mask: {}, index: {}, bit_offset: {}, row_offset: {}, sprite_mask: {}, sprite_val: {}, sprite_adjusted: {}",
                //     x, y, row, bit, mask, index, bit_offset, row_offset, sprite_mask, sprite_val, sprite_adjusted
                // );
                // println!(
                //     "row: {}, bit: {}, mask: {}, sprite_mask: {}, sprite_val: {}",
                //     row, bit, mask, sprite_mask, sprite_val
                // );
                buffer[index] ^= mask & sprite_adjusted;
                let val_after = buffer[index] & mask != 0;
                was_unset = was_unset | (val_before && !val_after);
            }
        }
        return was_unset;
    }

    fn clear(&self) {
        self.buffer.borrow_mut().fill(0);
    }
}
