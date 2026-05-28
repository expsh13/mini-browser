use crate::html::StyledText;

pub const WINDOW_WIDTH: usize = 800;
pub const WINDOW_HEIGHT: usize = 600;
pub const MARGIN: usize = 20;
pub const FONT_SCALE: usize = 2;
pub const CHAR_WIDTH: usize = 8 * FONT_SCALE;
pub const CHAR_HEIGHT: usize = 8 * FONT_SCALE;
pub const LINE_HEIGHT: usize = CHAR_HEIGHT + 6;

#[derive(Debug, PartialEq, Eq)]
pub struct LayoutLine {
    pub text: String,
    pub y: usize,
}

#[derive(Debug, PartialEq, Eq)]
pub struct LayoutRun {
    pub text: String,
    pub x: usize,
    pub y: usize,
    pub color: u32,
}

pub fn layout_text(text: &str) -> Vec<LayoutLine> {
    let max_chars = (WINDOW_WIDTH - MARGIN * 2) / CHAR_WIDTH;
    let mut lines = Vec::new();
    let mut current_line = String::new();
    let mut y = MARGIN;

    for ch in text.chars() {
        if ch == '\n' || current_line.chars().count() >= max_chars {
            lines.push(LayoutLine {
                text: current_line,
                y,
            });
            current_line = String::new();
            y += LINE_HEIGHT;
        }

        if ch != '\n' {
            current_line.push(ch);
        }
    }

    if !current_line.is_empty() {
        lines.push(LayoutLine {
            text: current_line,
            y,
        });
    }

    lines
}

pub fn layout_styled_text(pieces: &[StyledText]) -> Vec<LayoutRun> {
    let max_x = WINDOW_WIDTH.saturating_sub(MARGIN + CHAR_WIDTH);
    let mut runs = Vec::new();
    let mut current_text = String::new();
    let mut current_color = pieces.first().map(|piece| piece.color).unwrap_or(0);
    let mut current_x = MARGIN;
    let mut x = MARGIN;
    let mut y = MARGIN;

    for piece in pieces {
        for ch in piece.text.chars() {
            if ch == '\n' || x > max_x {
                push_run(&mut runs, &mut current_text, current_x, y, current_color);
                x = MARGIN;
                current_x = MARGIN;
                y += LINE_HEIGHT;
            }

            if ch == '\n' {
                continue;
            }

            if current_text.is_empty() {
                current_color = piece.color;
                current_x = x;
            } else if current_color != piece.color {
                push_run(&mut runs, &mut current_text, current_x, y, current_color);
                current_color = piece.color;
                current_x = x;
            }

            current_text.push(ch);
            x += CHAR_WIDTH;
        }
    }

    push_run(&mut runs, &mut current_text, current_x, y, current_color);

    runs
}

fn push_run(runs: &mut Vec<LayoutRun>, current_text: &mut String, x: usize, y: usize, color: u32) {
    if current_text.is_empty() {
        return;
    }

    runs.push(LayoutRun {
        text: std::mem::take(current_text),
        x,
        y,
        color,
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lays_out_text_lines() {
        let lines = layout_text("Hello\nmini browser");

        assert_eq!(
            lines,
            vec![
                LayoutLine {
                    text: "Hello".into(),
                    y: MARGIN,
                },
                LayoutLine {
                    text: "mini browser".into(),
                    y: MARGIN + LINE_HEIGHT,
                },
            ]
        );
    }

    #[test]
    fn lays_out_styled_text_runs() {
        let runs = layout_styled_text(&[
            StyledText {
                text: "Hello ".into(),
                color: 0x00111111,
            },
            StyledText {
                text: "red".into(),
                color: 0x00dc2626,
            },
        ]);

        assert_eq!(
            runs,
            vec![
                LayoutRun {
                    text: "Hello ".into(),
                    x: MARGIN,
                    y: MARGIN,
                    color: 0x00111111,
                },
                LayoutRun {
                    text: "red".into(),
                    x: MARGIN + CHAR_WIDTH * 6,
                    y: MARGIN,
                    color: 0x00dc2626,
                },
            ]
        );
    }
}
