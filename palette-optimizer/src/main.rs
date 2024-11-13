extern crate color_lib;
extern crate palette_visualizer;

mod metric;
mod optimizer;
mod score;
mod update;

use bitvec::{bitvec, vec::BitVec};
use color_lib::*;
use indicatif::{ProgressBar, ProgressStyle};
use itertools::{iproduct, Itertools};
use metric::*;
use optimizer::Optimizer;
use palette_visualizer::save_svg;
use std::{
    collections::{HashMap, HashSet, VecDeque},
    iter::repeat_with,
    ops::Not,
    time::Instant,
};

#[allow(dead_code)]
fn breakpoint() {
    let mut buf = String::new();
    std::io::stdin().read_line(&mut buf).unwrap();
}

struct Constrained_sRGB {
    inside: BitVec,
    surface: HashSet<sRGB>,
    edge: HashSet<sRGB>,
    corner: HashSet<sRGB>,
}

impl Constrained_sRGB {
    fn new() -> Self {
        Self {
            inside: BitVec::repeat(true, 1 << 24),
            surface: Self::surfaces(),
            edge: Self::edges(),
            corner: Self::corners(),
        }
    }

    fn surfaces() -> HashSet<sRGB> {
        iproduct!(0x00..=0x00, 0x00..=0xFF, 0x00..=0xFF)
            .chain(iproduct!(0xFF..=0xFF, 0x00..=0xFF, 0x00..=0xFF))
            .chain(iproduct!(0x00..=0xFF, 0x00..=0x00, 0x00..=0xFF))
            .chain(iproduct!(0x00..=0xFF, 0xFF..=0xFF, 0x00..=0xFF))
            .chain(iproduct!(0x00..=0xFF, 0x00..=0xFF, 0x00..=0x00))
            .chain(iproduct!(0x00..=0xFF, 0x00..=0xFF, 0xFF..=0xFF))
            .map(|(r, g, b)| [r, g, b])
            .collect()
    }

    fn edges() -> HashSet<sRGB> {
        iproduct!(0x00..=0x00, 0x00..=0x00, 0x00..=0xFF)
            .chain(iproduct!(0x00..=0x00, 0xFF..=0xFF, 0x00..=0xFF))
            .chain(iproduct!(0xFF..=0xFF, 0x00..=0x00, 0x00..=0xFF))
            .chain(iproduct!(0xFF..=0xFF, 0xFF..=0xFF, 0x00..=0xFF))
            .chain(iproduct!(0x00..=0x00, 0x00..=0xFF, 0x00..=0x00))
            .chain(iproduct!(0x00..=0x00, 0x00..=0xFF, 0xFF..=0xFF))
            .chain(iproduct!(0xFF..=0xFF, 0x00..=0xFF, 0x00..=0x00))
            .chain(iproduct!(0xFF..=0xFF, 0x00..=0xFF, 0xFF..=0xFF))
            .chain(iproduct!(0x00..=0xFF, 0x00..=0x00, 0x00..=0x00))
            .chain(iproduct!(0x00..=0xFF, 0x00..=0x00, 0xFF..=0xFF))
            .chain(iproduct!(0x00..=0xFF, 0xFF..=0xFF, 0x00..=0x00))
            .chain(iproduct!(0x00..=0xFF, 0xFF..=0xFF, 0xFF..=0xFF))
            .map(|(r, g, b)| [r, g, b])
            .collect()
    }

    fn corners() -> HashSet<sRGB> {
        iproduct!([0x00, 0xFF], [0x00, 0xFF], [0x00, 0xFF])
            .map(|(r, g, b)| [r, g, b])
            .collect()
    }

    fn apply_constraint(&mut self, c_start: sRGB, f: impl Fn(sRGB) -> bool) {
        // cool spiral algorithm time!!!
        let start_index = as_index(&c_start);
        if !self.inside[start_index] || f(c_start) {
            return;
        }
        self.inside.set(start_index, false);
        let (mut r_min, mut r_max, mut g_min, mut g_max, mut b_min, mut b_max) = (
            c_start[0], c_start[0], c_start[1], c_start[1], c_start[2], c_start[2],
        );
        let (mut r_down, mut r_up, mut g_down, mut g_up, mut b_down, mut b_up) =
            (true, true, true, true, true, true);
        loop {
            let face_iter = if b_up && b_max != 0xFF {
                b_up = false;
                b_max = b_max.saturating_add(1);
                iproduct!(r_min..=r_max, g_min..=g_max, b_max..=b_max)
            } else if b_down && b_min != 0x00 {
                b_down = false;
                b_min = b_min.saturating_sub(1);
                iproduct!(r_min..=r_max, g_min..=g_max, b_min..=b_min)
            } else if g_up && g_max != 0xFF {
                g_up = false;
                g_max = g_max.saturating_add(1);
                iproduct!(r_min..=r_max, g_max..=g_max, b_min..=b_max)
            } else if g_down && g_min != 0x00 {
                g_down = false;
                g_min = g_min.saturating_sub(1);
                iproduct!(r_min..=r_max, g_min..=g_min, b_min..=b_max)
            } else if r_up && r_max != 0xFF {
                r_up = false;
                r_max = r_max.saturating_add(1);
                iproduct!(r_max..=r_max, g_min..=g_max, b_min..=b_max)
            } else if r_down && r_min != 0x00 {
                r_down = false;
                r_min = r_min.saturating_sub(1);
                iproduct!(r_min..=r_min, g_min..=g_max, b_min..=b_max)
            } else {
                return;
            };
            for (r, g, b) in face_iter {
                let c = [r, g, b];
                let index = as_index(&c);
                if !self.inside[index] || f(c) {
                    continue;
                }
                self.inside.set(index, false);
                self.surface.remove(&c);
                self.edge.remove(&c);
                self.corner.remove(&c);
                if r == r_min {
                    r_down = true;
                }
                if r == r_max {
                    r_up = true;
                }
                if g == g_min {
                    g_down = true;
                }
                if g == g_max {
                    g_up = true;
                }
                if b == b_min {
                    b_down = true;
                }
                if b == b_max {
                    b_up = true;
                }
            }
        }
    }
}

fn main() {
    let bgs = [[0x00, 0x00, 0x00], [0xFF, 0xFF, 0xFF]];
    // let backgrounds = bgs.iter().map(|c| (*c).into()).collect_vec();
    // let color_lut = SrgbLut::new(Oklab::from);
    // let prot_lut = SrgbLut::new(simulate_protan);
    // let deut_lut = SrgbLut::new(simulate_deutan);
    // let trit_lut = SrgbLut::new(simulate_tritan);
    // // let constraint_lut =
    // //     SrgbLut::new_constraint(&backgrounds, |c1, c2| HyAB(c1, &color_lut.get(c2)));
    // let apca_constraint_lut = SrgbLut::new_constraint(&bgs.to_vec(), |c1, c2| APCA(c2, c1));

    let apca_thresh = 45.0;

    let mut start_time = Instant::now();

    let mut c_sRGB = Constrained_sRGB::new();

    println!(
        "{:#?}:\t{}\t{}\t{}\t{}",
        start_time.elapsed(),
        c_sRGB.inside.count_ones(),
        c_sRGB.surface.len(),
        c_sRGB.edge.len(),
        c_sRGB.corner.len()
    );

    start_time = Instant::now();

    c_sRGB.apply_constraint(bgs[0], |c| APCA(&c, &bgs[0]) > apca_thresh);

    println!(
        "{:#?}:\t{}\t{}\t{}\t{}",
        start_time.elapsed(),
        c_sRGB.inside.count_ones(),
        c_sRGB.surface.len(),
        c_sRGB.edge.len(),
        c_sRGB.corner.len()
    );

    start_time = Instant::now();

    c_sRGB.apply_constraint(bgs[1], |c| APCA(&c, &bgs[1]) > apca_thresh);

    println!(
        "{:#?}:\t{}\t{}\t{}\t{}",
        start_time.elapsed(),
        c_sRGB.inside.count_ones(),
        c_sRGB.surface.len(),
        c_sRGB.edge.len(),
        c_sRGB.corner.len()
    );

    // let mut output_colors = vec![];
    // let start_time = Instant::now();
    // while within_constraint.len() > 0 {
    //     let pb = ProgressBar::new(within_constraint.len().try_into().unwrap()).with_style(
    //         ProgressStyle::with_template(
    //             "{elapsed_precise}/{duration_precise} {wide_bar} {percent:>02}% {pos}/{len} {per_sec}",
    //         )
    //         .unwrap(),
    //     );

    //     let least_invalidated = within_constraint
    //         .par_iter()
    //         .map(|(srgb1, c1)| {
    //             let mut invalidated = 0;
    //             for (srgb2, c2) in within_constraint.iter() {
    //                 if srgb1 == srgb2 {
    //                     continue;
    //                 }
    //                 if HyAB(c1, c2) < hyab_thresh {
    //                     invalidated += 1;
    //                 }
    //             }
    //             pb.inc(1);
    //             ((*srgb1, *c1), invalidated)
    //         })
    //         .reduce(
    //             || (([0, 0, 0], Oklab::default()), usize::MAX),
    //             |old, new| {
    //                 if new.1 < old.1 {
    //                     new
    //                 } else {
    //                     old
    //                 }
    //             },
    //         );

    //     pb.finish();
    //     println!(
    //         "{}: {}",
    //         to_string(&least_invalidated.0 .0),
    //         least_invalidated.1
    //     );

    //     output_colors.push(least_invalidated.0 .0);

    //     within_constraint = within_constraint
    //         .into_iter()
    //         .filter(|(_, c1)| HyAB(c1, &least_invalidated.0 .1) >= hyab_thresh)
    //         .collect();
    // }

    // println!(
    //     "{:#?}\t{:?}",
    //     start_time.elapsed(),
    //     output_colors.iter().map(to_string).collect_vec()
    // );

    // save_svg(format!("img_{:02}.svg", 98), output_colors).unwrap();

    // let num_iter: u64 = 10000000;
    // let broad_iter = num_iter / 2;
    // let small_iter: u64 = 10000;
    // let update_freq: u64 = 1000000;
    // // breakpoint();
    // for big_num in 0..1 {
    //     let colors = repeat_with(rand::random).take(128).collect_vec();
    //     let mut optimizer = Optimizer::new(
    //         vec![
    //             (25.0, PairDistance::new(&colors, &color_lut)),
    //             // (20.0, PairDistance::new(&colors, &prot_lut)),
    //             // (20.0, PairDistance::new(&colors, &deut_lut)),
    //             // (15.0, PairDistance::new(&colors, &trit_lut)),
    //         ],
    //         vec![(30.0, Constraint::new(&colors, &apca_constraint_lut))],
    //         colors,
    //     );

    //     let start_time = Instant::now();
    //     let pb = ProgressBar::new(num_iter).with_style(ProgressStyle::with_template(
    //         "{elapsed_precise}/{duration_precise} {wide_bar} {percent:>02}% {pos}/{len} {per_sec}",
    //     ).unwrap());

    //     let mut counter = 0;
    //     let mut best_score = optimizer.get_best_score();
    //     for it in 0..num_iter {
    //         optimizer.update();
    //         if it >= broad_iter {
    //             counter += 1;
    //             if optimizer.get_best_score() > best_score {
    //                 counter = 0;
    //                 best_score = optimizer.get_best_score();
    //             } else if counter >= small_iter {
    //                 counter = 0;
    //                 optimizer.restore_best();
    //             }
    //         }
    //         if it % update_freq == update_freq - 1 {
    //             pb.inc(update_freq)
    //         }
    //     }
    //     pb.finish_and_clear();
    //     let best = optimizer.get_best();
    //     println!(
    //         "{}:\t{:#?}\t{}\t{:?}",
    //         big_num,
    //         start_time.elapsed(),
    //         best.0,
    //         best.1.iter().map(to_string).collect_vec()
    //     );
    //     // 's/[\[" #]//g'
    //     // https://www.atatus.com/tools/color-code-viewer#

    //     save_svg(format!("img_{:02}.svg", big_num), best.1).unwrap();
    // }

    // breakpoint();
}
