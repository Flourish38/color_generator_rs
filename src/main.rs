use fast_srgb8::srgb8_to_f32;
use indicatif::{ProgressBar, ProgressStyle};

#[allow(non_camel_case_types)]
type sRGB = [u8; 3];
#[allow(non_camel_case_types)]
// Bastardized name. ARGB in sRGB space.
type sARGB = u32;

struct RGB {
    r:f32,
    g:f32,
    b:f32
}

impl From<sARGB> for RGB {
    fn from(c: sARGB) -> Self {
        let r = (c >> 16) & 0xFF;
        let g = (c >> 8) & 0xFF;
        let b = c & 0xFF;

        RGB{
            r: srgb8_to_f32(r as u8),
            g: srgb8_to_f32(g as u8),
            b: srgb8_to_f32(b as u8)
        }
    }
}

impl From<sRGB> for RGB {
    fn from(c: sRGB) -> Self {
        RGB { 
            r: srgb8_to_f32(c[0]), 
            g: srgb8_to_f32(c[1]), 
            b: srgb8_to_f32(c[2]) 
        }
    }
}

#[allow(non_snake_case)]
struct Oklab {
    L: f32,
    a: f32,
    b: f32
}

impl From<RGB> for Oklab {
    fn from(c: RGB) -> Self {
        let l = 0.4122214708 * c.r + 0.5363325363 * c.g + 0.0514459929 * c.b;
        let m = 0.2119034982 * c.r + 0.6806995451 * c.g + 0.1073969566 * c.b;
        let s = 0.0883024619 * c.r + 0.2817188376 * c.g + 0.6299787005 * c.b;

        let l_ = l.cbrt();
        let m_ = m.cbrt();
        let s_ = s.cbrt();

        Oklab{
            L: 0.2104542553 * l_ + 0.7936177850 * m_ + 0.0040720468 * s_,
            a: 1.9779984951 * l_ + 2.4285922050 * m_ + 0.4505937099 * s_,
            b: 0.0259040371 * l_ + 0.7827717662 * m_ + 0.8086757660 * s_
        }
    }
}

impl From<sARGB> for Oklab {
    fn from(c: sARGB) -> Self {
        Into::<RGB>::into(c).into()
    }
}

impl From<sRGB> for Oklab {
    fn from(c: sRGB) -> Self {
        Into::<RGB>::into(c).into()
    }
}

#[allow(non_snake_case)]
fn HyAB (c1: &Oklab, c2: &Oklab) -> f32 {
    return (c1.L - c2.L).abs() + ((c1.a - c2.a).powi(2) + (c1.b - c2.b).powi(2)).sqrt()
}

fn main() {
    let mut max_diff = 0.0;
    let mut best_1: sRGB = [0, 0, 0];
    let mut best_2: sRGB = [0, 0, 0];
    let bar = ProgressBar::new(1 << 16).with_style(ProgressStyle::with_template("{elapsed_precise}/{duration_precise} {wide_bar} {percent:>02}% {pos}/{len} {per_sec}").unwrap());
    for r in 0x00..=0xFF {
        for b in 0x00..=0xFF {
            let mut best_diff = 0.0;
            let c1: sRGB = [r, 0xFF, b];
            bar.inc(1);
            let c1_lab: Oklab = c1.into();
            for r2 in 0x00..=0xFF {
                for b2 in 0x00..=0xFF {
                    let c2 = [r2, 0, b2];
                    let c2_lab: Oklab = c2.into();

                    let diff = HyAB(&c1_lab, &c2_lab);
                    
                    if diff > best_diff {
                        best_diff = diff;
                        
                        if diff > max_diff {
                            max_diff = diff;
                            best_1 = c1;
                            best_2 = c2;
                        }
                        
                    }
                }
                
                //println!("{}\t{}\t{}\t\t{}", r, 0xFF, b, best_diff)
            }
        }
    }
    

    
    

    println!("{:?}\t{:?}\t\t{}", best_1, best_2, HyAB(&best_1.into(), &best_2.into()))
}
