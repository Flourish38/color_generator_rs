use std::{f32::INFINITY, iter::repeat_with, time::Instant};
use itertools::{iproduct, Itertools};
use fast_srgb8::srgb8_to_f32;
use indicatif::{ProgressBar, ProgressIterator, ProgressStyle};

#[allow(non_camel_case_types)]
type sRGB = [u8; 3];

fn as_index(c: sRGB) -> usize {
    // RGB order. Might change later.
    let mut out: usize = c[2] as usize;
    out |= (c[1] as usize) << 8;
    out |= (c[0] as usize) << 16;
    out
}

struct RGB {
    r:f32,
    g:f32,
    b:f32
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
#[derive(PartialEq, Debug, Clone, Copy, Default)]
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

impl From<sRGB> for Oklab {
    fn from(c: sRGB) -> Self {
        Into::<RGB>::into(c).into()
    }
}

#[allow(non_snake_case)]
fn HyAB (c1: &Oklab, c2: &Oklab) -> f32 {
    return (c1.L - c2.L).abs() + ((c1.a - c2.a).powi(2) + (c1.b - c2.b).powi(2)).sqrt()
}


struct SrgbLut<T> {
    data: Vec<T>
}

impl<T: Copy> SrgbLut<T> {
    fn new(f: impl Fn(sRGB) -> T) -> Self {
        let mut data = Vec::with_capacity(1<<24);
        // I wish there was an easy way to allow this to be parallel,
        // But it is fast enough that it isn't a significant issue.
        for (r, g, b) in iproduct!(0x00..=0xFF, 0x00..=0xFF, 0x00..=0xFF) {
            let c = [r, g, b];
            data.push(f(c))
        }
        Self{ data: data }
    }

    fn get(&self, c: sRGB) -> T {
        self.data[as_index(c)]
    }
}

struct DistanceMetric<T, F1: Fn(sRGB) -> T, F2: Fn(&T, &T) -> f32> {
    preprocess: F1,
    dist: F2,
}

impl<T, F1: Fn(sRGB) -> T, F2: Fn(&T, &T) -> f32> DistanceMetric<T, F1, F2>{
    fn preprocess(&self, c: sRGB) -> T {
        (self.preprocess)(c)
    }

    fn dist(&self, x:&T, y: &T) -> f32 {
        (self.dist)(x, y)
    }
}

fn get_min_score<T, F1: Fn(sRGB) -> T, F2: Fn(&T, &T) -> f32>(colors: &Vec<sRGB>, dm:&DistanceMetric<T, F1, F2>) -> (usize, usize, f32) {
    let mut output = (0, 0, -INFINITY);
    for i in 0..(colors.len()-1) {
        let c1 = dm.preprocess(colors[i]);
        for j in (i+1)..colors.len() {
            let c2 = dm.preprocess(colors[j]);
            let dist = dm.dist(&c1, &c2);
            if dist > output.2 {
                output = (i, j, dist)
            }
        }
    }
    return output;
}

fn main() {
    let colors:Vec<sRGB> = repeat_with(rand::random).take(10000).collect_vec();
    let oklab_lut: SrgbLut<Oklab> = SrgbLut::new(|c| c.into());
    let dm = DistanceMetric { 
        preprocess: |c| oklab_lut.get(c),
        dist: |x, y| HyAB(&x, &y)
    };
    let start_time = Instant::now();
    let output = get_min_score(&colors, &dm);
    println!("{:?}\t{:?}\t{:?}\t\t{:#?}", output, colors[output.0], colors[output.1], start_time.elapsed());
}
