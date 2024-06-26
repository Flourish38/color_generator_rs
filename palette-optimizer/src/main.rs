extern crate lib;
extern crate palette_visualizer;

use indicatif::{ProgressBar, ProgressStyle};
use itertools::Itertools;
use lib::color::*;
use lib::metric::*;
use lib::update::*;
use palette_visualizer::save_svg;
use std::{f32::INFINITY, iter::repeat_with, time::Instant};

#[allow(dead_code)]
fn breakpoint() {
    let mut buf = String::new();
    std::io::stdin().read_line(&mut buf).unwrap();
}

fn main() {
    // let bgs = [[0x00, 0x00, 0x00], [0xFF, 0xFF, 0xFF]];
    // let backgrounds = bgs.iter().map(|c| (*c).into()).collect_vec();
    let color_lut = SrgbLut::new(simulate_deutan);
    // let constraint_lut =
    //     SrgbLut::new_constraint(&backgrounds, |c1, c2| HyAB(c1, &color_lut.get(c2)));
    // let apca_constraint_lut = SrgbLut::new_constraint(&bgs.to_vec(), |c1, c2| APCA(c2, c1));

    let num_iter: u64 = 1000000000;
    let update_freq: u64 = 1000000;
    // breakpoint();
    for big_num in 0..1 {
        let mut colors: Vec<sRGB> = repeat_with(rand::random).take(200).collect_vec();
        let mut score_metric = PairDistance::new(&colors, &color_lut);
        let mut best = (-INFINITY, Vec::new());

        let start_time = Instant::now();
        let pb = ProgressBar::new(num_iter).with_style(ProgressStyle::with_template(
            "{elapsed_precise}/{duration_precise} {wide_bar} {percent:>02}% {pos}/{len} {per_sec}",
        ).unwrap());

        for _it in 0..num_iter {
            let (score, ind) = score_metric.get_min_score();
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
            let (mut index, mut new_color) = update_color_pair(&colors, ind);
            if !score_metric.test_improvement(index, &new_color) {
                (index, new_color) = update_color_pair(&colors, ind);
            }
            colors[index] = new_color;
            score_metric.update(index, &colors[index]);
            if _it % update_freq == update_freq - 1 {
                pb.inc(update_freq)
            }
        }
        pb.finish_and_clear();
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
