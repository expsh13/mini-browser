#[derive(Debug, PartialEq, Eq)]
pub enum Node {
    Element(ElementNode),
    Text(String),
}

#[derive(Debug, PartialEq, Eq)]
pub struct ElementNode {
    pub tag: String,
    pub attributes: Vec<Attribute>,
    pub children: Vec<Node>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Attribute {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StyledText {
    pub text: String,
    pub color: u32,
}

pub const DEFAULT_COLOR: u32 = 0x001f2937;

pub fn html_to_text(html: &str) -> String {
    styled_text_to_string(&html_to_styled_text(html))
}

pub fn html_to_styled_text(html: &str) -> Vec<StyledText> {
    let dom = parse_html(html);
    dom_to_styled_text(&dom)
}

pub fn parse_html(html: &str) -> Node {
    let mut parser = Parser::new(html);
    let children = parser.parse_nodes(None);

    Node::Element(ElementNode {
        tag: "document".into(),
        attributes: Vec::new(),
        children,
    })
}

pub fn dom_to_styled_text(node: &Node) -> Vec<StyledText> {
    let mut pieces = Vec::new();
    collect_styled_text(node, DEFAULT_COLOR, &mut pieces);
    normalize_styled_text(&pieces)
}

fn collect_styled_text(node: &Node, current_color: u32, pieces: &mut Vec<StyledText>) {
    match node {
        Node::Text(content) => push_piece(pieces, content, current_color),
        Node::Element(element) => match element.tag.as_str() {
            "br" => push_piece(pieces, "\n", current_color),
            "p" | "h1" => {
                let color = style_color(element).unwrap_or(current_color);
                push_piece(pieces, "\n\n", color);
                collect_styled_children(&element.children, color, pieces);
                push_piece(pieces, "\n\n", color);
            }
            _ => {
                let color = style_color(element).unwrap_or(current_color);
                collect_styled_children(&element.children, color, pieces);
            }
        },
    }
}

fn collect_styled_children(children: &[Node], color: u32, pieces: &mut Vec<StyledText>) {
    for child in children {
        collect_styled_text(child, color, pieces);
    }
}

fn push_piece(pieces: &mut Vec<StyledText>, text: &str, color: u32) {
    if text.is_empty() {
        return;
    }

    if let Some(last) = pieces.last_mut()
        && last.color == color
    {
        last.text.push_str(text);
        return;
    }

    pieces.push(StyledText {
        text: text.into(),
        color,
    });
}

fn style_color(element: &ElementNode) -> Option<u32> {
    let style = element
        .attributes
        .iter()
        .find(|attr| attr.name == "style")?;

    style.value.split(';').find_map(|declaration| {
        let (name, value) = declaration.split_once(':')?;

        if name.trim().eq_ignore_ascii_case("color") {
            parse_color(value.trim())
        } else {
            None
        }
    })
}

fn parse_color(input: &str) -> Option<u32> {
    match input.to_ascii_lowercase().as_str() {
        "black" => Some(0x00111111),
        "blue" => Some(0x002563eb),
        "green" => Some(0x0016a34a),
        "red" => Some(0x00dc2626),
        "white" => Some(0x00ffffff),
        hex if hex.starts_with('#') && hex.len() == 7 => u32::from_str_radix(&hex[1..], 16).ok(),
        _ => None,
    }
}

struct Parser<'a> {
    input: &'a str,
    position: usize,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, position: 0 }
    }

    fn parse_nodes(&mut self, stop_tag: Option<&str>) -> Vec<Node> {
        let mut nodes = Vec::new();

        while !self.eof() {
            if self.starts_with("</") {
                let closing_tag = self.parse_closing_tag();
                if Some(closing_tag.as_str()) == stop_tag {
                    break;
                }
            } else if self.starts_with("<") {
                nodes.push(self.parse_element());
            } else {
                nodes.push(self.parse_text());
            }
        }

        nodes
    }

    fn parse_element(&mut self) -> Node {
        self.consume_char('<');
        let tag_content = self.consume_until('>');
        self.consume_char('>');

        let (tag, attributes) = parse_tag(&tag_content);
        let children = if tag == "br" {
            Vec::new()
        } else {
            self.parse_nodes(Some(&tag))
        };

        Node::Element(ElementNode {
            tag,
            attributes,
            children,
        })
    }

    fn parse_closing_tag(&mut self) -> String {
        self.consume_char('<');
        self.consume_char('/');
        let tag = self.consume_until('>');
        self.consume_char('>');
        tag.trim().to_ascii_lowercase()
    }

    fn parse_text(&mut self) -> Node {
        Node::Text(self.consume_until('<'))
    }

    fn starts_with(&self, prefix: &str) -> bool {
        self.input[self.position..].starts_with(prefix)
    }

    fn eof(&self) -> bool {
        self.position >= self.input.len()
    }

    fn consume_char(&mut self, expected: char) {
        if self.input[self.position..].starts_with(expected) {
            self.position += expected.len_utf8();
        }
    }

    fn consume_until(&mut self, target: char) -> String {
        let start = self.position;

        while !self.eof() && !self.input[self.position..].starts_with(target) {
            let char = self.input[self.position..]
                .chars()
                .next()
                .expect("parser position should point to a char");
            self.position += char.len_utf8();
        }

        self.input[start..self.position].to_string()
    }
}

fn parse_tag(input: &str) -> (String, Vec<Attribute>) {
    let input = input.trim();
    let tag_end = input.find(char::is_whitespace).unwrap_or(input.len());
    let tag = input[..tag_end].to_ascii_lowercase();
    let attributes = parse_attributes(&input[tag_end..]);

    (tag, attributes)
}

fn parse_attributes(mut input: &str) -> Vec<Attribute> {
    let mut attributes = Vec::new();

    loop {
        input = input.trim_start();

        if input.is_empty() {
            break;
        }

        let Some(eq_index) = input.find('=') else {
            break;
        };
        let name = input[..eq_index].trim().to_ascii_lowercase();
        input = input[eq_index + 1..].trim_start();

        let Some(first_char) = input.chars().next() else {
            break;
        };

        let (value, rest) = if first_char == '"' || first_char == '\'' {
            let quote = first_char;
            input = &input[quote.len_utf8()..];

            match input.find(quote) {
                Some(end_index) => (input[..end_index].to_string(), &input[end_index + 1..]),
                None => (input.to_string(), ""),
            }
        } else {
            let end_index = input.find(char::is_whitespace).unwrap_or(input.len());
            (input[..end_index].to_string(), &input[end_index..])
        };

        if !name.is_empty() {
            attributes.push(Attribute { name, value });
        }

        input = rest;
    }

    attributes
}

fn normalize_styled_text(pieces: &[StyledText]) -> Vec<StyledText> {
    let mut chars = Vec::new();
    let mut previous_was_space = false;
    let mut newline_count = 0;

    for piece in pieces {
        for char in piece.text.chars() {
            if char == '\n' {
                if newline_count < 2 {
                    chars.push((char, piece.color));
                }
                previous_was_space = false;
                newline_count += 1;
            } else if char.is_whitespace() {
                if !previous_was_space && newline_count == 0 {
                    chars.push((' ', piece.color));
                }
                previous_was_space = true;
            } else {
                chars.push((char, piece.color));
                previous_was_space = false;
                newline_count = 0;
            }
        }
    }

    while chars.first().is_some_and(|(char, _)| char.is_whitespace()) {
        chars.remove(0);
    }

    while chars.last().is_some_and(|(char, _)| char.is_whitespace()) {
        chars.pop();
    }

    let mut normalized = Vec::new();

    for (char, color) in chars {
        let text = char.to_string();
        push_piece(&mut normalized, &text, color);
    }

    normalized
}

fn styled_text_to_string(pieces: &[StyledText]) -> String {
    pieces
        .iter()
        .map(|piece| piece.text.as_str())
        .collect::<String>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_nested_elements() {
        let dom = parse_html("<p>Hello<br>World</p>");

        assert_eq!(
            dom,
            Node::Element(ElementNode {
                tag: "document".into(),
                attributes: Vec::new(),
                children: vec![Node::Element(ElementNode {
                    tag: "p".into(),
                    attributes: Vec::new(),
                    children: vec![
                        Node::Text("Hello".into()),
                        Node::Element(ElementNode {
                            tag: "br".into(),
                            attributes: Vec::new(),
                            children: Vec::new(),
                        }),
                        Node::Text("World".into()),
                    ],
                })],
            })
        );
    }

    #[test]
    fn parses_attributes() {
        let dom = parse_html(r#"<p style="color: red">Hello</p>"#);

        assert_eq!(
            dom,
            Node::Element(ElementNode {
                tag: "document".into(),
                attributes: Vec::new(),
                children: vec![Node::Element(ElementNode {
                    tag: "p".into(),
                    attributes: vec![Attribute {
                        name: "style".into(),
                        value: "color: red".into(),
                    }],
                    children: vec![Node::Text("Hello".into())],
                })],
            })
        );
    }

    #[test]
    fn extracts_text_from_html() {
        let html = "<html><body><h1>Hello</h1><p>mini browser</p></body></html>";

        assert_eq!(html_to_text(html), "Hello\n\nmini browser");
    }

    #[test]
    fn extracts_styled_text_from_style_attribute() {
        let html = r#"<p>Hello <span style="color: red">red</span></p>"#;

        assert_eq!(
            html_to_styled_text(html),
            vec![
                StyledText {
                    text: "Hello ".into(),
                    color: DEFAULT_COLOR,
                },
                StyledText {
                    text: "red".into(),
                    color: 0x00dc2626,
                },
            ]
        );
    }

    #[test]
    fn extracts_hex_color_from_style_attribute() {
        let html = r#"<p style="color: #2563eb">blue</p>"#;

        assert_eq!(
            html_to_styled_text(html),
            vec![StyledText {
                text: "blue".into(),
                color: 0x002563eb,
            }]
        );
    }

    #[test]
    fn converts_line_break_tags_to_newlines() {
        let html = "<p>Hello<br>mini browser</p>";

        assert_eq!(html_to_text(html), "Hello\nmini browser");
    }
}
