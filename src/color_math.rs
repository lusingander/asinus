pub fn linear_srgb(rgb: [u8; 3]) -> [f64; 3] {
    rgb.map(|value| {
        let value = f64::from(value) / 255.0;
        if value <= 0.04045 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    })
}

/// WCAG 2.2 contrast ratio for two opaque sRGB colors.
pub fn contrast(a: [u8; 3], b: [u8; 3]) -> f64 {
    let a = luminance(a);
    let b = luminance(b);
    (a.max(b) + 0.05) / (a.min(b) + 0.05)
}

fn luminance(rgb: [u8; 3]) -> f64 {
    let [r, g, b] = linear_srgb(rgb);
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case([0, 0, 0], [255, 255, 255], 21.0)]
    #[case([119, 119, 119], [255, 255, 255], 4.478089453577214)]
    #[case([10, 10, 10], [255, 255, 255], 19.79814571052481)]
    #[case([88, 91, 112], [30, 30, 46], 2.4566019820283453)]
    fn contrast_uses_linear_srgb_and_is_symmetric(
        #[case] a: [u8; 3],
        #[case] b: [u8; 3],
        #[case] expected: f64,
    ) {
        assert!((contrast(a, b) - expected).abs() < 0.000001);
        assert_eq!(contrast(a, b), contrast(b, a));
        assert_eq!(contrast(a, a), 1.0);
    }
}
