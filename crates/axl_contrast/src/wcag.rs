use crate::Rgb;

fn linear_channel(value: u8) -> f64 {
    let srgb = (value as f64) / 255.0;
    if srgb <= 0.039_28 {
        srgb / 12.92
    } else {
        ((srgb + 0.055) / 1.055).powf(2.4)
    }
}

pub fn relative_luminance(rgb: Rgb) -> f64 {
    (0.2126 * linear_channel(rgb.r)) + (0.7152 * linear_channel(rgb.g)) + (0.0722 * linear_channel(rgb.b))
}

pub fn contrast_ratio(foreground: Rgb, background: Rgb) -> f64 {
    let l1 = relative_luminance(foreground);
    let l2 = relative_luminance(background);
    let (lighter, darker) = if l1 > l2 { (l1, l2) } else { (l2, l1) };
    (lighter + 0.05) / (darker + 0.05)
}

#[cfg(test)]
mod tests {
    use super::contrast_ratio;
    use crate::Rgb;

    #[test]
    fn contrast_black_white_is_high() {
        let ratio = contrast_ratio(
            Rgb { r: 0, g: 0, b: 0 },
            Rgb {
                r: 255,
                g: 255,
                b: 255,
            },
        );
        assert!(ratio > 20.0);
    }
}
