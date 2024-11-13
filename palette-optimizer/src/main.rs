extern crate color_lib;
extern crate palette_visualizer;

mod metric;
mod optimizer;
mod score;
mod update;

use color_lib::*;
use indicatif::{ProgressBar, ProgressStyle};
use itertools::{iproduct, Itertools};
use metric::*;
use optimizer::Optimizer;
use palette_visualizer::save_svg;
use rayon::prelude::*;
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

#[derive(Clone, Copy)]
enum Status {
    Inside,
    Face,
    Edge,
    Corner,
    Outside,
}

impl Status {
    fn upgrade(self) -> Self {
        match self {
            Self::Inside => Self::Face,
            Self::Face => Self::Edge,
            Self::Edge | Self::Corner => Self::Corner,
            Self::Outside => Self::Outside,
        }
    }
}

fn flood_fill(
    within_constraint: &HashMap<sRGB, Status>,
    c_start: sRGB,
    f: impl Fn(sRGB) -> bool,
) -> Vec<(sRGB, Status)> {
    let mut visited = HashSet::from([c_start]);
    let mut queued =
        VecDeque::from([(c_start, within_constraint.get(&c_start).unwrap().upgrade())]);
    let mut changes = vec![];
    while let Some((c, status)) = queued.pop_front() {
        if f(c) {
            changes.push((c, Status::Outside));
            for (sign, axis) in iproduct!([1, -1], [0, 1, 2]) {
                let mut new_c = c;
                new_c[axis] = u8::saturating_add_signed(new_c[axis], sign);
                match (visited.contains(&new_c), within_constraint.get(&new_c)) {
                    (true, _) => continue,
                    (_, None) => continue,
                    (_, Some(new_status)) => {
                        queued.push_back((new_c, new_status.upgrade()));
                        visited.insert(new_c);
                    }
                };
            }
        } else {
            changes.push((c, status));
        }
    }
    changes
}

fn apply_changes(within_constraint: &mut HashMap<sRGB, Status>, changes: Vec<(sRGB, Status)>) {
    for (c, s) in changes {
        if let Status::Outside = s {
            within_constraint.remove(&c);
        } else {
            within_constraint.insert(c, s);
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
    let apca_thresh = 48.0;
    let hyab_thresh = 25.0;

    println!("{:?}", Instant::now());

    let mut within_constraint: HashMap<sRGB, Status> =
        iproduct!(0x00..=0xFF, 0x00..=0xFF, 0x00..=0xFF)
            .map(|(r, g, b)| {
                let srgb = [r, g, b];
                let r_surface = match r {
                    0x00 | 0xFF => 1,
                    _ => 0,
                };
                let g_surface = match g {
                    0x00 | 0xFF => 1,
                    _ => 0,
                };
                let b_surface = match b {
                    0x00 | 0xFF => 1,
                    _ => 0,
                };
                let surfaces = r_surface + g_surface + b_surface;
                let status = match surfaces {
                    0 => Status::Inside,
                    1 => Status::Face,
                    2 => Status::Edge,
                    3 => Status::Corner,
                    _ => unreachable!(),
                };
                (srgb, status)
            })
            .collect();

    println!("{:?}", Instant::now());

    for bg in bgs {
        let start_time = Instant::now();
        let changes = flood_fill(&within_constraint, bg, |c| APCA(&c, &bg) < apca_thresh);
        println!("{:#?}\t{} changes", start_time.elapsed(), changes.len());
        apply_changes(&mut within_constraint, changes);
    }

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
