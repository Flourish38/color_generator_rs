use itertools::{iproduct, Itertools};
use once_cell::sync::Lazy;
use rand::{distributions, distributions::Distribution, thread_rng};

use crate::color::sRGB;

struct ColorPairUpdate {
    which: Which,
    cu: ColorUpdate,
}

struct ColorUpdate {
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
#[derive(Clone, Debug, PartialEq)]
enum Sign {
    Positive,
    Negative,
}

fn color_update(mut c: sRGB, cu: &ColorUpdate) -> sRGB {
    let axis = cu.axis as usize;
    let num = if (cu.sign == Sign::Positive && c[axis] != 0xFF) || c[axis] == 0x00 {
        1
    } else {
        -1
    };
    c[axis] = u8::wrapping_add_signed(c[axis], num);
    c
}

static UPDATE_SLICE: Lazy<Vec<ColorUpdate>> = Lazy::new(|| {
    iproduct!(
        [Axis::R, Axis::G, Axis::B],
        [Sign::Positive, Sign::Negative]
    )
    .map(|(a, s)| ColorUpdate { axis: a, sign: s })
    .collect_vec()
});
static UPDATE_DISTRIBUTION: Lazy<distributions::Slice<'static, ColorUpdate>> =
    Lazy::new(|| distributions::Slice::new(UPDATE_SLICE.as_slice()).expect("Slice empty"));

pub fn update_color(colors: &Vec<sRGB>, i: usize) -> sRGB {
    let cu = UPDATE_DISTRIBUTION.sample(&mut thread_rng());
    color_update(colors[i], cu)
}

static UPDATE_PAIR_SLICE: Lazy<Vec<ColorPairUpdate>> = Lazy::new(|| {
    iproduct!(
        [Which::First, Which::Second],
        [Axis::R, Axis::G, Axis::B],
        [Sign::Positive, Sign::Negative]
    )
    .map(|(w, a, s)| ColorPairUpdate {
        which: w,
        cu: ColorUpdate { axis: a, sign: s },
    })
    .collect_vec()
});
static UPDATE_PAIR_DISTRIBUTION: Lazy<distributions::Slice<'static, ColorPairUpdate>> =
    Lazy::new(|| distributions::Slice::new(UPDATE_PAIR_SLICE.as_slice()).expect("Slice empty"));

pub fn update_color_pair(colors: &Vec<sRGB>, (i, j): (usize, usize)) -> (usize, sRGB) {
    let cu = UPDATE_PAIR_DISTRIBUTION.sample(&mut thread_rng());
    let index = match cu.which {
        Which::First => i,
        Which::Second => j,
    };
    (index, color_update(colors[index], &cu.cu))
}
