use crate::error::BrowserError;
use font8x8::UnicodeFonts;
use minifb::{Key, Window, WindowOptions};

const WINDOW_WIDTH: usize = 800;
const WINDOW_HEIGHT: usize = 600;
const BACKGROUND: u32 = 0x00fffcf7;
const TEXT: u32 = 0x001f2937;
const MARGIN: usize = 20;
const FONT_SCALE: usize = 2;
const CHAR_WIDTH: usize = 8 * FONT_SCALE;
const CHAR_HEIGHT: usize = 8 * FONT_SCALE;
const LINE_HEIGHT: usize = CHAR_HEIGHT + 6;

pub fn show_window(text: &str) -> Result<(), BrowserError> {
    let mut window = Window::new(
        "mini-browser",
        WINDOW_WIDTH,
        WINDOW_HEIGHT,
        WindowOptions::default(),
    )?;
    let mut buffer = vec![BACKGROUND; WINDOW_WIDTH * WINDOW_HEIGHT];

    draw_text(&mut buffer, text);

    while window.is_open() && !window.is_key_down(Key::Escape) {
        window.update_with_buffer(&buffer, WINDOW_WIDTH, WINDOW_HEIGHT)?;
    }

    Ok(())
}

fn draw_text(buffer: &mut [u32], text: &str) {
    let max_x = WINDOW_WIDTH.saturating_sub(MARGIN + CHAR_WIDTH);
    let max_y = WINDOW_HEIGHT.saturating_sub(MARGIN + CHAR_HEIGHT);
    let mut x = MARGIN;
    let mut y = MARGIN;

    for ch in text.chars() {
        if ch == '\n' || x > max_x {
            x = MARGIN;
            y += LINE_HEIGHT;
        }

        if y > max_y {
            break;
        }

        if ch != '\n' {
            draw_char(buffer, x, y, ch);
            x += CHAR_WIDTH;
        }
    }
}

fn draw_char(buffer: &mut [u32], x: usize, y: usize, ch: char) {
    let glyph = font8x8::BASIC_FONTS
        .get(ch)
        .or_else(|| font8x8::BASIC_FONTS.get('?'));

    let Some(glyph) = glyph else {
        return;
    };

    for (row, bits) in glyph.iter().enumerate() {
        for col in 0..8 {
            if bits & (1 << col) == 0 {
                continue;
            }

            fill_pixel(buffer, x + col * FONT_SCALE, y + row * FONT_SCALE);
        }
    }
}

fn fill_pixel(buffer: &mut [u32], x: usize, y: usize) {
    for dy in 0..FONT_SCALE {
        for dx in 0..FONT_SCALE {
            let px = x + dx;
            let py = y + dy;

            if px < WINDOW_WIDTH && py < WINDOW_HEIGHT {
                buffer[py * WINDOW_WIDTH + px] = TEXT;
            }
        }
    }
}
