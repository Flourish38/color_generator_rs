use fast_srgb8::srgb8_to_f32;
use indicatif::{ProgressBar, ProgressIterator, ProgressStyle};
use itertools::{enumerate, iproduct, Itertools};
use once_cell::sync::Lazy;
use rand::{
    distributions::{self, Distribution},
    thread_rng,
};
use std::{f32::INFINITY, iter::repeat_with, time::Instant};

#[allow(non_camel_case_types)]
type sRGB = [u8; 3];

fn as_index(c: &sRGB) -> usize {
    // RGB order. Might change later.
    let mut out: usize = c[2] as usize;
    out |= (c[1] as usize) << 8;
    out |= (c[0] as usize) << 16;
    out
}

fn to_string(c: &sRGB) -> String {
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
#[derive(PartialEq, Debug, Clone, Copy, Default)]
struct Oklab {
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
fn HyAB(c1: &Oklab, c2: &Oklab) -> f32 {
    return (c1.L - c2.L).abs() + ((c1.a - c2.a).powi(2) + (c1.b - c2.b).powi(2)).sqrt();
}

struct SrgbLut<T> {
    data: Vec<T>,
}

impl<T: Copy> SrgbLut<T> {
    fn new(f: impl Fn(sRGB) -> T) -> Self {
        let mut data = Vec::with_capacity(1 << 24);
        // I wish there was an easy way to allow this to be parallel,
        // But it is fast enough that it isn't a significant issue.
        for (r, g, b) in iproduct!(0x00..=0xFF, 0x00..=0xFF, 0x00..=0xFF) {
            let c = [r, g, b];
            data.push(f(c))
        }
        Self { data: data }
    }

    fn get(&self, c: &sRGB) -> T {
        self.data[as_index(c)]
    }
}

fn get_score<T, F: Fn(&T, &T) -> f32>(i: usize, pre_colors: &Vec<T>, dist: F) -> (usize, f32) {
    let c = &pre_colors[i];
    let mut score = (i, INFINITY);
    for j in (i + 1)..pre_colors.len() {
        let dist = dist(c, &pre_colors[j]);
        if dist < score.1 {
            score = (j, dist);
        }
    }
    return score;
}

fn get_scores<T, F: Fn(&T, &T) -> f32>(pre_colors: &Vec<T>, dist: &F) -> Vec<(usize, f32)> {
    let mut scores = Vec::with_capacity(pre_colors.len() - 1);
    for i in 0..(pre_colors.len() - 1) {
        scores.push(get_score(i, pre_colors, dist));
    }
    return scores;
}

fn get_min_score(scores: &Vec<(usize, f32)>) -> (usize, usize, f32) {
    let mut output = (0, 0, INFINITY);
    for (i, (j, val)) in enumerate(scores) {
        if val < &output.2 {
            output = (i, *j, *val);
        }
    }
    output
}

#[derive(Debug)]
struct ColorUpdate {
    which: Which,
    axis: Axis,
    sign: Sign,
}

#[derive(Clone, Debug)]
enum Which {
    First,
    Second,
}
#[derive(Clone, Copy, Debug)]
enum Axis {
    R = 0,
    G = 1,
    B = 2,
}
#[derive(Clone, Debug)]
enum Sign {
    Positive,
    Negative,
}

static UPDATE_SLICE: Lazy<Vec<ColorUpdate>> = Lazy::new(|| {
    iproduct!(
        [Which::First, Which::Second],
        [Axis::R, Axis::G, Axis::B],
        [Sign::Positive, Sign::Negative]
    )
    .map(|(w, a, s)| ColorUpdate {
        which: w,
        axis: a,
        sign: s,
    })
    .collect_vec()
});
static UPDATE_DISTRIBUTION: Lazy<distributions::Slice<'static, ColorUpdate>> =
    Lazy::new(|| distributions::Slice::new(UPDATE_SLICE.as_slice()).expect("Slice empty"));

fn update_color(colors: &mut Vec<sRGB>, (i, j): (usize, usize)) -> usize {
    let cu = UPDATE_DISTRIBUTION.sample(&mut thread_rng());
    let (index, num) = match cu.which {
        Which::First => match cu.sign {
            Sign::Positive if colors[i][cu.axis as usize] == 0xFF => (j, 0xFF),
            Sign::Negative if colors[i][cu.axis as usize] == 0x00 => (j, 0x01),
            Sign::Positive => (i, 0x01),
            Sign::Negative => (i, 0xFF),
        },
        Which::Second => match cu.sign {
            Sign::Positive if colors[j][cu.axis as usize] == 0xFF => (i, 0xFF),
            Sign::Negative if colors[j][cu.axis as usize] == 0x00 => (i, 0x01),
            Sign::Positive => (j, 0x01),
            Sign::Negative => (j, 0xFF),
        },
    };
    colors[index][cu.axis as usize] += num;
    index
}

fn update_scores<T, F: Fn(&T, &T) -> f32>(
    scores: &mut Vec<(usize, f32)>,
    updated_index: usize,
    pre_colors: &Vec<T>,
    dist: &F,
) {
    let c_updated = &pre_colors[updated_index];

    // Recompute scores of indexes before updated_index
    for i in 0..updated_index {
        let (prev_index, prev_score) = scores[i];
        let score = dist(c_updated, &pre_colors[i]);
        if score < prev_score {
            scores[i] = (updated_index, score);
        } else if prev_index == updated_index {
            // Have to recompute score for this element
            scores[i] = get_score(i, pre_colors, dist);
        } // else, no need to change it
    }

    // Recompute score of updated_index
    if updated_index < scores.len() {
        // scores are 1 shorter than the colors vec
        scores[updated_index] = get_score(updated_index, pre_colors, dist)
    }
}

// ProgressStyle::with_template("{elapsed_precise}/{duration_precise} {wide_bar} {percent:>02}% {pos}/{len} {per_sec}").unwrap()

fn main() {
    // let c1: sRGB = [0x00, 0x10, 0x0D];
    // let c2: sRGB = [0x03, 0x00, 0x05];
    // let r1: RGB = c1.into();
    // let r2: RGB = c2.into();
    // let ok1 = c1.into();
    // let ok2 = c2.into();
    // println!("{:?}\t{:?}\t{}", ok1, ok2, HyAB(&ok1, &ok2));
    let oklab_lut: SrgbLut<Oklab> = SrgbLut::new(|c| c.into());
    let num_iter: u64 = 1000000000;

    for big_num in 1..40 {
        let mut colors: Vec<sRGB> = repeat_with(rand::random).take(20).collect_vec();
        let mut oklab_colors: Vec<Oklab> = colors.iter().map(|c| oklab_lut.get(c)).collect_vec();
        // let t1 = Instant::now();
        let mut scores = get_scores(&oklab_colors, &HyAB);
        let mut best = (-INFINITY, Vec::new());

        let start_time = Instant::now();
        for _it in (0..num_iter).progress_with(
            ProgressBar::new(num_iter).with_style(ProgressStyle::with_template(
                "{elapsed_precise}/{duration_precise} {wide_bar} {percent:>02}% {pos}/{len} {per_sec}",
            )
            .unwrap(),
        )) {
            let (i, j, score) = get_min_score(&scores);
            if score > best.0 {
                best = (score, colors.clone());
                // println!(
                //     "{: >10}\t{}\t{}\t{}",
                //     it,
                //     score,
                //     to_string(&colors[i]),
                //     to_string(&colors[j])
                // );
            }
            let index = update_color(&mut colors, (i, j));
            oklab_colors[index] = oklab_lut.get(&colors[index]);
            update_scores(&mut scores, index, &oklab_colors, &HyAB);
        }
        println!(
            "{}:\t{:#?}\t{}\t{:?}",
            big_num,
            start_time.elapsed(),
            best.0,
            best.1.iter().map(to_string).collect_vec()
        );
    }
    // let oklab_best = best.1.iter().map(|c| From::from(*c)).collect_vec();
    // let best_scores = get_scores(&oklab_best, &HyAB);
    // let min_score = get_min_score(&best_scores);
    // println!(
    //     "{}\t{}\t{}",
    //     min_score.2,
    //     to_string(&best.1[min_score.0]),
    //     to_string(&best.1[min_score.1])
    // );
    // assert_eq!(best.0, min_score.2)
}
