use crate::error::BrowserError;
use crate::layout::{
    CHAR_HEIGHT, CHAR_WIDTH, FONT_SCALE, LayoutRun, MARGIN, WINDOW_HEIGHT, WINDOW_WIDTH,
};
use font8x8::UnicodeFonts;
use minifb::{Key, Window, WindowOptions};

const BACKGROUND: u32 = 0x00fffcf7;
const SCROLL_STEP: usize = 30;

pub fn show_window(runs: &[LayoutRun]) -> Result<(), BrowserError> {
    let mut window = Window::new(
        "mini-browser",
        WINDOW_WIDTH,
        WINDOW_HEIGHT,
        WindowOptions::default(),
    )?;
    let mut buffer = vec![BACKGROUND; WINDOW_WIDTH * WINDOW_HEIGHT];
    let mut scroll_y = 0;
    let max_scroll_y = max_scroll_y(runs);

    while window.is_open() && !window.is_key_down(Key::Escape) {
        scroll_y = update_scroll_y(&window, scroll_y, max_scroll_y);
        buffer.fill(BACKGROUND);
        draw_runs(&mut buffer, runs, scroll_y);
        window.update_with_buffer(&buffer, WINDOW_WIDTH, WINDOW_HEIGHT)?;
    }

    Ok(())
}

fn max_scroll_y(runs: &[LayoutRun]) -> usize {
    runs.last()
        .map(|run| {
            run.y
                .saturating_add(CHAR_HEIGHT)
                .saturating_sub(WINDOW_HEIGHT - MARGIN)
        })
        .unwrap_or(0)
}

fn update_scroll_y(window: &Window, scroll_y: usize, max_scroll_y: usize) -> usize {
    if window.is_key_down(Key::Down) {
        scroll_y.saturating_add(SCROLL_STEP).min(max_scroll_y)
    } else if window.is_key_down(Key::Up) {
        scroll_y.saturating_sub(SCROLL_STEP)
    } else {
        scroll_y
    }
}

fn draw_runs(buffer: &mut [u32], runs: &[LayoutRun], scroll_y: usize) {
    let max_y = WINDOW_HEIGHT.saturating_sub(MARGIN + CHAR_HEIGHT);

    for run in runs {
        if run.y.saturating_add(CHAR_HEIGHT) < scroll_y {
            continue;
        }

        let Some(screen_y) = run.y.checked_sub(scroll_y) else {
            continue;
        };

        if screen_y > max_y {
            continue;
        }

        let mut x = run.x;

        for ch in run.text.chars() {
            draw_char(buffer, x, screen_y, ch, run.color);
            x += CHAR_WIDTH;
        }
    }
}

fn draw_char(buffer: &mut [u32], x: usize, y: usize, ch: char, color: u32) {
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

            fill_pixel(buffer, x + col * FONT_SCALE, y + row * FONT_SCALE, color);
        }
    }
}

fn fill_pixel(buffer: &mut [u32], x: usize, y: usize, color: u32) {
    for dy in 0..FONT_SCALE {
        for dx in 0..FONT_SCALE {
            let px = x + dx;
            let py = y + dy;

            if px < WINDOW_WIDTH && py < WINDOW_HEIGHT {
                buffer[py * WINDOW_WIDTH + px] = color;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::LINE_HEIGHT;

    #[test]
    fn max_scroll_y_is_zero_when_content_fits() {
        let runs = vec![LayoutRun {
            text: "Hello".into(),
            x: MARGIN,
            y: MARGIN,
            color: 0x00111111,
        }];

        assert_eq!(max_scroll_y(&runs), 0);
    }

    #[test]
    fn max_scroll_y_keeps_last_line_visible() {
        let runs = vec![LayoutRun {
            text: "Last".into(),
            x: MARGIN,
            y: WINDOW_HEIGHT + LINE_HEIGHT,
            color: 0x00111111,
        }];

        assert_eq!(max_scroll_y(&runs), LINE_HEIGHT + MARGIN + CHAR_HEIGHT);
    }
}
