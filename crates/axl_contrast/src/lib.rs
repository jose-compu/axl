pub mod wcag;

#[derive(Debug, Clone, Copy)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Debug, Clone)]
pub struct ContrastCheckResult {
    pub ratio: f64,
    pub passes_aa_normal_text: bool,
    pub passes_aa_large_text: bool,
    pub passes_aaa_normal_text: bool,
}

pub fn parse_hex_color(input: &str) -> Option<Rgb> {
    let trimmed = input.trim();
    let hex = trimmed.strip_prefix('#')?;
    match hex.len() {
        3 => {
            let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()?;
            let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?;
            let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?;
            Some(Rgb { r, g, b })
        }
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            Some(Rgb { r, g, b })
        }
        _ => None,
    }
}

pub fn evaluate_inline_text_contrast(foreground: &str, background: &str) -> Option<ContrastCheckResult> {
    let fg = parse_hex_color(foreground)?;
    let bg = parse_hex_color(background)?;
    let ratio = wcag::contrast_ratio(fg, bg);
    Some(ContrastCheckResult {
        ratio,
        passes_aa_normal_text: ratio >= 4.5,
        passes_aa_large_text: ratio >= 3.0,
        passes_aaa_normal_text: ratio >= 7.0,
    })
}

#[cfg(test)]
mod tests {
    use super::{evaluate_inline_text_contrast, parse_hex_color};

    #[test]
    fn parses_valid_hex_color() {
        let parsed = parse_hex_color("#A1b2C3").expect("hex should parse");
        assert_eq!(parsed.r, 0xA1);
        assert_eq!(parsed.g, 0xB2);
        assert_eq!(parsed.b, 0xC3);
    }

    #[test]
    fn rejects_invalid_hex_color() {
        assert!(parse_hex_color("A1B2C3").is_none());
        assert!(parse_hex_color("#ab").is_none());
        assert!(parse_hex_color("#zzzzzz").is_none());
    }

    #[test]
    fn parses_three_digit_hex_color() {
        let parsed = parse_hex_color("#abc").expect("short hex should parse");
        assert_eq!(parsed.r, 0xAA);
        assert_eq!(parsed.g, 0xBB);
        assert_eq!(parsed.b, 0xCC);
    }

    #[test]
    fn evaluates_inline_contrast() {
        let result = evaluate_inline_text_contrast("#000000", "#ffffff").expect("contrast should compute");
        assert!(result.ratio > 20.0);
        assert!(result.passes_aa_normal_text);
        assert!(result.passes_aa_large_text);
        assert!(result.passes_aaa_normal_text);

        let low = evaluate_inline_text_contrast("#777777", "#888888").expect("contrast should compute");
        assert!(low.ratio < 4.5);
        assert!(!low.passes_aa_normal_text);
    }
}
