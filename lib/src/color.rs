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
struct RGB {
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
#[derive(Debug, Clone, Copy, Default)]
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
        // Copied from https://bottosson.github.io/posts/oklab/#converting-from-linear-srgb-to-oklab.
        // Accessed 2024-01-30.
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

#[allow(non_snake_case)]
fn apca_luminance(c: &sRGB) -> f32 {
    const S_TRC: f32 = 2.4;
    const B_THRSH: f32 = 0.022;
    const B_CLIP: f32 = 1.414;

    let Y_c = ((c[0] as f32) / 255.0).powf(S_TRC) * 0.2126729
        + ((c[1] as f32) / 255.0).powf(S_TRC) * 0.7151522
        + ((c[2] as f32) / 255.0).powf(S_TRC) * 0.0721750;

    if Y_c < 0.0 {
        0.0
    } else if Y_c < B_THRSH {
        Y_c + (B_THRSH - Y_c).powf(B_CLIP)
    } else {
        Y_c
    }
}

// Implementation of https://github.com/Myndex/SAPC-APCA/blob/master/documentation/APCA-W3-LaTeX.md.
// Accessed 2024-03-19.
#[allow(non_snake_case)]
pub fn APCA(text: &sRGB, bg: &sRGB) -> f32 {
    const NTX: f32 = 0.57;
    const NBG: f32 = 0.56;
    const RTX: f32 = 0.62;
    const RGB: f32 = 0.65;
    const W_SCALE: f32 = 1.14;
    const W_OFFSET: f32 = 0.027;

    let Y_txt = apca_luminance(text);
    let Y_bg = apca_luminance(bg);

    let S_apc = if Y_txt < Y_bg {
        Y_bg.powf(NBG) - Y_txt.powf(NTX)
    } else {
        Y_bg.powf(RGB) - Y_txt.powf(RTX)
    } * W_SCALE;

    if S_apc.abs() < W_OFFSET {
        0.0
    } else if S_apc > 0.0 {
        100.0 * (S_apc - W_OFFSET)
    } else {
        -100.0 * (S_apc + W_OFFSET)
    }
}

#[allow(non_snake_case)]
struct LMS {
    l: f32,
    m: f32,
    s: f32,
}

impl From<RGB> for LMS {
    fn from(c: RGB) -> Self {
        // Matrix derived in src/color_derivations.jl. Result of line 29.
        LMS {
            l: 0.178824041258 * c.r + 0.43516090570 * c.g + 0.04119349692 * c.b,
            m: 0.034556423182 * c.r + 0.27155382458 * c.g + 0.03867130836 * c.b,
            s: 0.000299565576 * c.r + 0.00184308960 * c.g + 0.01467086130 * c.b,
        }
    }
}

impl From<sRGB> for LMS {
    fn from(c: sRGB) -> Self {
        Into::<RGB>::into(c).into()
    }
}

impl From<LMS> for RGB {
    fn from(c: LMS) -> Self {
        // Matrix derived in src/color_derivations.jl. Result of line 33.
        RGB {
            r: 8.09443559803237000 * c.l - 13.050431460496924 * c.m + 11.672058453917323 * c.s,
            g: -1.02485055866466830 * c.l + 5.4019313096749730 * c.m - 11.361471490598714 * c.s,
            b: -0.03652974715933317 * c.l - 0.4121628070012680 * c.m + 69.351324238208580 * c.s,
        }
    }
}

impl From<LMS> for Oklab {
    fn from(c: LMS) -> Self {
        Into::<RGB>::into(c).into()
    }
}

impl LMS {
    // Derived in src/color_derivations.jl. Result of line 116.
    fn simulate_protan(self) -> Self {
        let l = if -0.016813516536 * self.m + 0.344781556122 * self.s > 0.0 {
            2.1683061543738997 * self.m - 5.496382983183359 * self.s
        } else {
            2.18614812275877 * self.m - 5.862254192269454 * self.s
        };
        LMS {
            l: l,
            m: self.m,
            s: self.s,
        }
    }

    // Derived in src/color_derivations.jl. Result of line 117.
    fn simulate_deutan(self) -> Self {
        let m = if -0.016813516536 * self.l + 0.655178443878 * self.s > 0.0 {
            0.4611894856189028 * self.l + 2.5348740407788237 * self.s
        } else {
            0.45742554659931645 * self.l + 2.681544828202989 * self.s
        };
        LMS {
            l: self.l,
            m: m,
            s: self.s,
        }
    }

    // Derived in src/color_derivations.jl. Result of line 118.
    fn simulate_tritan(self) -> Self {
        let s = if -0.344781556121 * self.l + 0.655178443878 * self.m > 0.0 {
            -0.06010959444193691 * self.l + 0.1629902356630733 * self.m
        } else {
            -0.002574363979654964 * self.l + 0.05365769715251148 * self.m
        };
        LMS {
            l: self.l,
            m: self.m,
            s: s,
        }
    }
}

pub fn simulate_protan(c: sRGB) -> Oklab {
    Into::<LMS>::into(c).simulate_protan().into()
}
pub fn simulate_deutan(c: sRGB) -> Oklab {
    Into::<LMS>::into(c).simulate_deutan().into()
}
pub fn simulate_tritan(c: sRGB) -> Oklab {
    Into::<LMS>::into(c).simulate_tritan().into()
}

#[allow(non_snake_case)]
pub struct Oklch {
    L: f32,
    C: f32,
    h: f32,
}

impl From<Oklab> for Oklch {
    fn from(c: Oklab) -> Self {
        Self {
            L: c.L,
            C: (c.a.powi(2) + c.b.powi(2)).sqrt(),
            h: c.b.atan2(c.a),
        }
    }
}

impl From<sRGB> for Oklch {
    fn from(c: sRGB) -> Self {
        Into::<Oklab>::into(c).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;
    use itertools::iproduct;

    // Source: https://git.apcacontrast.com/documentation/README
    // Accessed 2023-03-19.
    #[test]
    fn test_apca() {
        let c_888: sRGB = [0x88, 0x88, 0x88];
        let c_fff: sRGB = [0xff, 0xff, 0xff];
        let c_000: sRGB = [0x00, 0x00, 0x00];
        let c_aaa: sRGB = [0xaa, 0xaa, 0xaa];
        let c_123: sRGB = [0x11, 0x22, 0x33];
        let c_def: sRGB = [0xdd, 0xee, 0xff];
        let c_444: sRGB = [0x44, 0x44, 0x44];
        let c_234: sRGB = [0x22, 0x33, 0x44];

        // This epsilon is exactly correct, since the result never exceeds 2^7 and f32 has 24 mantissa bits.
        let eps = 2.0_f32.powi(-17);

        assert_abs_diff_eq!(63.056469930209424, APCA(&c_888, &c_fff), epsilon = eps);
        assert_abs_diff_eq!(68.54146436644962, APCA(&c_fff, &c_888), epsilon = eps);

        assert_abs_diff_eq!(58.146262578561334, APCA(&c_000, &c_aaa), epsilon = eps);
        assert_abs_diff_eq!(56.24113336839742, APCA(&c_aaa, &c_000), epsilon = eps);

        assert_abs_diff_eq!(91.66830811481631, APCA(&c_123, &c_def), epsilon = eps);
        assert_abs_diff_eq!(93.06770049484275, APCA(&c_def, &c_123), epsilon = eps);

        assert_abs_diff_eq!(8.32326136957393, APCA(&c_123, &c_444), epsilon = eps);
        assert_abs_diff_eq!(7.526878460278154, APCA(&c_444, &c_123), epsilon = eps);

        // Low-contrast
        assert_abs_diff_eq!(1.7512243099356113, APCA(&c_123, &c_234), epsilon = eps);
        assert_abs_diff_eq!(1.6349191031377903, APCA(&c_234, &c_123), epsilon = eps);
    }

    #[test]
    fn test_lms_roundtrip() {
        for (r, g, b) in iproduct!(0x00..=0xFF, 0x00..=0xFF, 0x00..=0xFF) {
            let c = [r, g, b];
            let c_rgb: RGB = c.into();
            let c_lms: LMS = c.into(); // This goes through RGB first
            let c_lms_rgb: RGB = c_lms.into();

            // This epsilon is more than sufficient for my needs, and probably close to perfect.
            let eps = 2.0_f32.powi(-19);
            assert_abs_diff_eq!(&c_rgb.r, &c_lms_rgb.r, epsilon = eps);
            assert_abs_diff_eq!(&c_rgb.g, &c_lms_rgb.g, epsilon = eps);
            assert_abs_diff_eq!(&c_rgb.b, &c_lms_rgb.b, epsilon = eps);
        }
    }
}
