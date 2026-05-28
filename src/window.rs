use crate::error::BrowserError;
use crate::layout::{
    CHAR_HEIGHT, CHAR_WIDTH, FONT_SCALE, LayoutLine, MARGIN, WINDOW_HEIGHT, WINDOW_WIDTH,
};
use font8x8::UnicodeFonts;
use minifb::{Key, Window, WindowOptions};

const BACKGROUND: u32 = 0x00fffcf7;
const TEXT: u32 = 0x001f2937;

pub fn show_window(lines: &[LayoutLine]) -> Result<(), BrowserError> {
    let mut window = Window::new(
        "mini-browser",
        WINDOW_WIDTH,
        WINDOW_HEIGHT,
        WindowOptions::default(),
    )?;
    let mut buffer = vec![BACKGROUND; WINDOW_WIDTH * WINDOW_HEIGHT];

    draw_lines(&mut buffer, lines);

    while window.is_open() && !window.is_key_down(Key::Escape) {
        window.update_with_buffer(&buffer, WINDOW_WIDTH, WINDOW_HEIGHT)?;
    }

    Ok(())
}

fn draw_lines(buffer: &mut [u32], lines: &[LayoutLine]) {
    let max_y = WINDOW_HEIGHT.saturating_sub(MARGIN + CHAR_HEIGHT);

    for line in lines {
        if line.y > max_y {
            continue;
        }

        let mut x = MARGIN;

        for ch in line.text.chars() {
            draw_char(buffer, x, line.y, ch);
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
