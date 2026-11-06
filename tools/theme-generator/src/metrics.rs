use crate::{Rgb, color_math};

#[derive(Debug)]
pub struct ThemeMetrics {
    background_lightness: f64,
    background_chroma: f64,
    background_hue: Option<f64>,
    text_contrast: f64,
}

impl ThemeMetrics {
    pub fn calculate(foreground: Rgb, background: Rgb) -> Result<Self, &'static str> {
        let (background_lightness, background_chroma, background_hue) =
            background_oklch(background);
        let metrics = Self {
            background_lightness,
            background_chroma,
            background_hue,
            text_contrast: color_math::contrast(
                [foreground.0, foreground.1, foreground.2],
                [background.0, background.1, background.2],
            ),
        };
        metrics.validate()?;
        Ok(metrics)
    }

    fn validate(&self) -> Result<(), &'static str> {
        if !(0.0..=100.0).contains(&self.background_lightness) {
            return Err("background lightness must be finite and in 0..=100");
        }
        if !self.background_chroma.is_finite() || self.background_chroma < 0.0 {
            return Err("background chroma must be finite and nonnegative");
        }
        match self.background_hue {
            None if self.background_chroma == 0.0 => {}
            Some(hue) if self.background_chroma > 0.0 && (0.0..360.0).contains(&hue) => {}
            _ => {
                return Err("chromatic backgrounds need a finite hue in 0..360; grays have no hue");
            }
        }
        if !(1.0..=21.0).contains(&self.text_contrast) {
            return Err("text contrast must be finite and in 1..=21");
        }
        Ok(())
    }
}

fn background_oklch(Rgb(red, green, blue): Rgb) -> (f64, f64, Option<f64>) {
    let [r, g, b] = color_math::linear_srgb([red, green, blue]);
    if red == green && green == blue {
        // Avoid matrix roundoff in neutral colors, including the black and white endpoints.
        return (100.0 * r.cbrt(), 0.0, None);
    }

    // Björn Ottosson's 2021-01-25 linear-sRGB matrices:
    // https://bottosson.github.io/posts/oklab/#converting-from-linear-srgb-to-oklab
    let l = (0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b).cbrt();
    let m = (0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b).cbrt();
    let s = (0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b).cbrt();
    let lightness = 100.0 * (0.2104542553 * l + 0.7936177850 * m - 0.0040720468 * s);
    let a = 1.9779984951 * l - 2.4285922050 * m + 0.4505937099 * s;
    let b = 0.0259040371 * l + 0.7827717662 * m - 0.8086757660 * s;
    let hue = b.atan2(a).to_degrees().rem_euclid(360.0);
    // A tiny negative angle may round up to 360 after rem_euclid.
    let hue = if hue >= 360.0 || hue == 0.0 { 0.0 } else { hue };
    (lightness, a.hypot(b), Some(hue))
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case(Rgb(255, 0, 0), 62.7955360615, 0.2576833077, 29.2338851923)]
    #[case(Rgb(0, 255, 0), 86.6439611536, 0.2948272403, 142.4953388878)]
    #[case(Rgb(0, 0, 255), 45.2013718385, 0.3132143717, 264.0520206381)]
    #[case(Rgb(30, 30, 46), 24.2866940356, 0.030357354, 283.910973405)]
    fn converts_encoded_srgb_to_oklch(
        #[case] background: Rgb,
        #[case] lightness: f64,
        #[case] chroma: f64,
        #[case] hue: f64,
    ) {
        let actual = ThemeMetrics::calculate(background, background).unwrap();
        assert!((actual.background_lightness - lightness).abs() < 1e-6);
        assert!((actual.background_chroma - chroma).abs() < 1e-9);
        assert!((actual.background_hue.unwrap() - hue).abs() < 1e-6);
        assert_eq!(actual.text_contrast, 1.0);
    }

    #[test]
    fn all_exact_grays_have_zero_chroma_and_no_hue() {
        let mut previous_lightness = -1.0;
        for value in 0..=255 {
            let gray = Rgb(value, value, value);
            let actual = ThemeMetrics::calculate(Rgb(0, 0, 0), gray).unwrap();
            assert_eq!(actual.background_chroma, 0.0);
            assert_eq!(actual.background_hue, None);
            assert!(actual.background_lightness > previous_lightness);
            previous_lightness = actual.background_lightness;
            if value == 0 || value == 255 {
                assert_eq!(
                    actual.background_lightness,
                    if value == 0 { 0.0 } else { 100.0 }
                );
                assert_eq!(actual.text_contrast, if value == 0 { 1.0 } else { 21.0 });
            }
        }
    }

    #[test]
    fn weakly_colored_backgrounds_keep_chroma_and_hue() {
        let actual = ThemeMetrics::calculate(Rgb(76, 79, 105), Rgb(239, 241, 245)).unwrap();
        assert!((0.0..0.015).contains(&actual.background_chroma));
        assert!(actual.background_hue.is_some());
        assert!((actual.text_contrast - 7.061999557).abs() < 1e-9);
    }

    #[rstest]
    #[case(-0.1, 0.0, None, 1.0)]
    #[case(100.1, 0.0, None, 1.0)]
    #[case(f64::NAN, 0.0, None, 1.0)]
    #[case(f64::INFINITY, 0.0, None, 1.0)]
    #[case(50.0, -0.1, None, 1.0)]
    #[case(50.0, f64::NAN, None, 1.0)]
    #[case(50.0, f64::INFINITY, Some(20.0), 1.0)]
    #[case(50.0, 0.02, None, 1.0)]
    #[case(50.0, 0.0, Some(20.0), 1.0)]
    #[case(50.0, 0.02, Some(-0.1), 1.0)]
    #[case(50.0, 0.02, Some(360.0), 1.0)]
    #[case(50.0, 0.02, Some(f64::NAN), 1.0)]
    #[case(50.0, 0.02, Some(f64::INFINITY), 1.0)]
    #[case(50.0, 0.0, None, 0.9)]
    #[case(50.0, 0.0, None, 21.1)]
    #[case(50.0, 0.0, None, f64::NAN)]
    #[case(50.0, 0.0, None, f64::INFINITY)]
    fn rejects_invalid_metrics(
        #[case] background_lightness: f64,
        #[case] background_chroma: f64,
        #[case] background_hue: Option<f64>,
        #[case] text_contrast: f64,
    ) {
        let metrics = ThemeMetrics {
            background_lightness,
            background_chroma,
            background_hue,
            text_contrast,
        };
        assert!(metrics.validate().is_err());
    }
}
