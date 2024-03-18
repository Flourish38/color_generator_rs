use fast_srgb8::srgb8_to_f32;

#[allow(non_camel_case_types)]
pub type sRGB = [u8; 3];

pub fn as_index(c: &sRGB) -> usize {
    // RGB order. Might change later.
    let mut out: usize = c[2] as usize;
    out |= (c[1] as usize) << 8;
    out |= (c[0] as usize) << 16;
    out
}

pub fn to_string(c: &sRGB) -> String {
    format!("#{:06x}", as_index(c)).to_uppercase()
}

#[derive(Debug)]
pub struct RGB {
    r: f32,
    g: f32,
    b: f32,
}

impl From<sRGB> for RGB {
    fn from(c: sRGB) -> Self {
        RGB {
            r: srgb8_to_f32(c[0]),
            g: srgb8_to_f32(c[1]),
            b: srgb8_to_f32(c[2]),
        }
    }
}

#[allow(non_snake_case)]
#[derive(PartialEq, Debug, Clone, Copy, Default)]
pub struct Oklab {
    L: f32,
    a: f32,
    b: f32,
}

// This is a scale factor to make it roughly line up with CIELAB.
// It's actually completely optional, but I think it makes the numbers nicer
const OKLAB_SCALE: f32 = 100.0;
impl From<RGB> for Oklab {
    fn from(c: RGB) -> Self {
        let l = 0.4122214708 * c.r + 0.5363325363 * c.g + 0.0514459929 * c.b;
        let m = 0.2119034982 * c.r + 0.6806995451 * c.g + 0.1073969566 * c.b;
        let s = 0.0883024619 * c.r + 0.2817188376 * c.g + 0.6299787005 * c.b;

        let l_ = l.cbrt();
        let m_ = m.cbrt();
        let s_ = s.cbrt();

        Oklab {
            L: (0.2104542553 * OKLAB_SCALE) * l_ + (0.7936177850 * OKLAB_SCALE) * m_
                - (0.0040720468 * OKLAB_SCALE) * s_,
            a: (1.9779984951 * OKLAB_SCALE) * l_ - (2.4285922050 * OKLAB_SCALE) * m_
                + (0.4505937099 * OKLAB_SCALE) * s_,
            b: (0.0259040371 * OKLAB_SCALE) * l_ + (0.7827717662 * OKLAB_SCALE) * m_
                - (0.8086757660 * OKLAB_SCALE) * s_,
        }
    }
}

impl From<sRGB> for Oklab {
    fn from(c: sRGB) -> Self {
        Into::<RGB>::into(c).into()
    }
}

#[allow(non_snake_case)]
pub fn HyAB(c1: &Oklab, c2: &Oklab) -> f32 {
    return (c1.L - c2.L).abs() + ((c1.a - c2.a).powi(2) + (c1.b - c2.b).powi(2)).sqrt();
}
