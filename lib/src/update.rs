use itertools::{iproduct, Itertools};
use once_cell::sync::Lazy;
use rand::{distributions, distributions::Distribution, thread_rng};

use crate::color::sRGB;

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

pub fn update_color(colors: &Vec<sRGB>, (i, j): (usize, usize)) -> (usize, sRGB) {
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
    let mut out_color = colors[index];
    out_color[cu.axis as usize] += num;
    (index, out_color)
}
