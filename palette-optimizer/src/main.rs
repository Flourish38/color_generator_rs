extern crate color_lib;
extern crate palette_visualizer;

mod metric;
mod optimizer;
mod score;
mod update;

use color_lib::*;
use indicatif::{ProgressBar, ProgressStyle};
use itertools::Itertools;
use metric::*;
use optimizer::Optimizer;
use palette_visualizer::save_svg;
use std::{iter::repeat_with, time::Instant};

#[allow(dead_code)]
fn breakpoint() {
    let mut buf = String::new();
    std::io::stdin().read_line(&mut buf).unwrap();
}

fn main() {
    let bgs = [[0x00, 0x00, 0x00], [0xFF, 0xFF, 0xFF]];
    // let backgrounds = bgs.iter().map(|c| (*c).into()).collect_vec();
    let color_lut = SrgbLut::new(Oklab::from);
    let prot_lut = SrgbLut::new(simulate_protan);
    let deut_lut = SrgbLut::new(simulate_deutan);
    let trit_lut = SrgbLut::new(simulate_tritan);
    // let constraint_lut =
    //     SrgbLut::new_constraint(&backgrounds, |c1, c2| HyAB(c1, &color_lut.get(c2)));
    let apca_constraint_lut = SrgbLut::new_constraint(&bgs.to_vec(), |c1, c2| APCA(c2, c1));

    let num_iter: u64 = 1000000000;
    let update_freq: u64 = 1000000;
    // breakpoint();
    for big_num in 0..1 {
        let colors = repeat_with(rand::random).take(8).collect_vec();
        let mut optimizer = Optimizer::new(
            vec![
                (25.0, PairDistance::new(&colors, &color_lut)),
                (20.0, PairDistance::new(&colors, &prot_lut)),
                (20.0, PairDistance::new(&colors, &deut_lut)),
                (15.0, PairDistance::new(&colors, &trit_lut)),
            ],
            vec![(30.0, Constraint::new(&colors, &apca_constraint_lut))],
            colors,
        );

        let start_time = Instant::now();
        let pb = ProgressBar::new(num_iter).with_style(ProgressStyle::with_template(
            "{elapsed_precise}/{duration_precise} {wide_bar} {percent:>02}% {pos}/{len} {per_sec}",
        ).unwrap());

        for _it in 0..num_iter {
            optimizer.update();
            if _it % update_freq == update_freq - 1 {
                pb.inc(update_freq)
            }
        }
        pb.finish_and_clear();
        let best = optimizer.get_best();
        println!(
            "{}:\t{:#?}\t{}\t{:?}",
            big_num,
            start_time.elapsed(),
            best.0,
            best.1.iter().map(to_string).collect_vec()
        );
        // 's/[\[" #]//g'
        // https://www.atatus.com/tools/color-code-viewer#

        save_svg(format!("img_{:02}.svg", big_num), best.1).unwrap();
    }

    // breakpoint();
}
