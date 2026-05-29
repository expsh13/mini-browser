pub const DEFAULT_COLOR: u32 = 0x001f2937;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StyleDeclaration {
    pub declarations: Vec<Declaration>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Declaration {
    pub property: String,
    pub value: String,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ComputedStyle {
    pub color: u32,
}

impl Default for ComputedStyle {
    fn default() -> Self {
        Self {
            color: DEFAULT_COLOR,
        }
    }
}

pub fn parse_inline_style(input: &str) -> StyleDeclaration {
    let declarations = input
        .split(';')
        .filter_map(|declaration| {
            let (property, value) = declaration.split_once(':')?;
            Some(Declaration {
                property: property.trim().to_ascii_lowercase(),
                value: value.trim().to_string(),
            })
        })
        .filter(|declaration| !declaration.property.is_empty() && !declaration.value.is_empty())
        .collect();

    StyleDeclaration { declarations }
}

pub fn compute_style(
    parent: ComputedStyle,
    inline_style: Option<&StyleDeclaration>,
) -> ComputedStyle {
    let mut style = parent;

    if let Some(inline_style) = inline_style {
        for declaration in &inline_style.declarations {
            if declaration.property == "color"
                && let Some(color) = parse_color(&declaration.value)
            {
                style.color = color;
            }
        }
    }

    style
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_inline_style_declarations() {
        let style = parse_inline_style("color: red; font-size: 16px");

        assert_eq!(
            style,
            StyleDeclaration {
                declarations: vec![
                    Declaration {
                        property: "color".into(),
                        value: "red".into(),
                    },
                    Declaration {
                        property: "font-size".into(),
                        value: "16px".into(),
                    },
                ],
            }
        );
    }

    #[test]
    fn computes_color_from_inline_style() {
        let inline_style = parse_inline_style("color: #2563eb");
        let style = compute_style(ComputedStyle::default(), Some(&inline_style));

        assert_eq!(style.color, 0x002563eb);
    }

    #[test]
    fn inherits_parent_color_without_inline_color() {
        let parent = ComputedStyle { color: 0x00dc2626 };
        let style = compute_style(parent, None);

        assert_eq!(style.color, 0x00dc2626);
    }
}
