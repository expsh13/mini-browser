pub fn html_to_text(html: &str) -> String {
    let mut text = String::new();
    let mut tag = String::new();
    let mut in_tag = false;

    for char in html.chars() {
        match char {
            '<' => {
                in_tag = true;
                tag.clear();
            }
            '>' if in_tag => {
                in_tag = false;
                apply_tag(&mut text, &tag);
            }
            _ if in_tag => tag.push(char),
            _ => text.push(char),
        }
    }

    normalize_text(&text)
}

fn apply_tag(text: &mut String, tag: &str) {
    let tag_name = tag
        .trim()
        .trim_start_matches('/')
        .split_whitespace()
        .next()
        .unwrap_or("")
        .to_ascii_lowercase();

    match tag_name.as_str() {
        "br" => text.push('\n'),
        "p" | "h1" => {
            if !text.ends_with('\n') {
                text.push('\n');
            }
            text.push('\n');
        }
        _ => text.push(' '),
    }
}

fn normalize_text(text: &str) -> String {
    let mut normalized = String::new();
    let mut previous_was_space = false;
    let mut newline_count = 0;

    for char in text.chars() {
        if char == '\n' {
            if newline_count < 2 {
                normalized.push('\n');
            }
            previous_was_space = false;
            newline_count += 1;
        } else if char.is_whitespace() {
            if !previous_was_space && newline_count == 0 {
                normalized.push(' ');
            }
            previous_was_space = true;
        } else {
            normalized.push(char);
            previous_was_space = false;
            newline_count = 0;
        }
    }

    normalized.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_text_from_html() {
        let html = "<html><body><h1>Hello</h1><p>mini browser</p></body></html>";

        assert_eq!(html_to_text(html), "Hello\n\nmini browser");
    }

    #[test]
    fn converts_line_break_tags_to_newlines() {
        let html = "<p>Hello<br>mini browser</p>";

        assert_eq!(html_to_text(html), "Hello\nmini browser");
    }
}
