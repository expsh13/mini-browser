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
}
