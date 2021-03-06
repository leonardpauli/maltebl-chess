#![allow(clippy::many_single_char_names)]
use druid::piet::Color;

pub struct ColorUtil;

impl ColorUtil {
    pub fn hsl(h: f64, s: f64, l: f64) -> Color {
        Self::rbg8t(Self::hsl_to_rgb(h, s, l))
    }
    pub const fn rbg8t((r, g, b): (u8, u8, u8)) -> Color {
        Color::rgb8(r, g, b)
    }
    // https://pauljmiller.com/posts/druid-widget-tutorial.html
    fn hue_to_rgb(p: f64, q: f64, t: f64) -> f64 {
        let mut t = t;
        if t < 0. {
            t += 1.
        }
        if t > 1. {
            t -= 1.
        };
        if t < 1. / 6. {
            p + (q - p) * 6. * t
        } else if t < 1. / 2. {
            q
        } else if t < 2. / 3. {
            p + (q - p) * (2. / 3. - t) * 6.
        } else {
            p
        }
    }

    fn hsl_to_rgb(h: f64, s: f64, l: f64) -> (u8, u8, u8) {
        let r;
        let g;
        let b;

        if s == 0.0 {
            // achromatic
            r = l;
            g = l;
            b = l;
        } else {
            let q = if l < 0.5 { l * (1. + s) } else { l + s - l * s };

            let p = 2. * l - q;
            r = Self::hue_to_rgb(p, q, h + 1. / 3.);
            g = Self::hue_to_rgb(p, q, h);
            b = Self::hue_to_rgb(p, q, h - 1. / 3.);
        }

        (
            (r * 255.).round() as u8,
            (g * 255.).round() as u8,
            (b * 255.).round() as u8,
        )
    }
}
