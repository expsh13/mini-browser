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

pub fn html_to_text(html: &str) -> String {
    let dom = parse_html(html);
    dom_to_text(&dom)
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

pub fn dom_to_text(node: &Node) -> String {
    let mut text = String::new();
    collect_text(node, &mut text);
    normalize_text(&text)
}

fn collect_text(node: &Node, text: &mut String) {
    match node {
        Node::Text(content) => text.push_str(content),
        Node::Element(element) => match element.tag.as_str() {
            "br" => text.push('\n'),
            "p" | "h1" => {
                push_block_break(text);
                collect_children(&element.children, text);
                push_block_break(text);
            }
            _ => collect_children(&element.children, text),
        },
    }
}

fn collect_children(children: &[Node], text: &mut String) {
    for child in children {
        collect_text(child, text);
    }
}

fn push_block_break(text: &mut String) {
    if !text.ends_with('\n') {
        text.push('\n');
    }
    text.push('\n');
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
    let mut parts = input.trim().split_whitespace();
    let tag = parts.next().unwrap_or("").to_ascii_lowercase();
    let attributes = parts.filter_map(parse_attribute).collect();

    (tag, attributes)
}

fn parse_attribute(input: &str) -> Option<Attribute> {
    let (name, value) = input.split_once('=')?;
    let value = value.trim_matches('"').trim_matches('\'').to_string();

    Some(Attribute {
        name: name.to_ascii_lowercase(),
        value,
    })
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
        let dom = parse_html(r#"<p style="color:red">Hello</p>"#);

        assert_eq!(
            dom,
            Node::Element(ElementNode {
                tag: "document".into(),
                attributes: Vec::new(),
                children: vec![Node::Element(ElementNode {
                    tag: "p".into(),
                    attributes: vec![Attribute {
                        name: "style".into(),
                        value: "color:red".into(),
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
    fn converts_line_break_tags_to_newlines() {
        let html = "<p>Hello<br>mini browser</p>";

        assert_eq!(html_to_text(html), "Hello\nmini browser");
    }
}
