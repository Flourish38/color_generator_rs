extern crate lib;

use indicatif::{ProgressBar, ProgressIterator, ProgressStyle};
use itertools::Itertools;
use lib::color::*;
use lib::metric::*;
use lib::update::*;
use std::{f32::INFINITY, iter::repeat_with, time::Instant};

fn main() {
    // let c1: sRGB = [0x00, 0x10, 0x0D];
    // let c2: sRGB = [0x03, 0x00, 0x05];
    // let r1: RGB = c1.into();
    // let r2: RGB = c2.into();
    // let ok1 = c1.into();
    // let ok2 = c2.into();
    // println!("{:?}\t{:?}\t{}", ok1, ok2, HyAB(&ok1, &ok2));
    // let c2 = [0xfe, 0xff, 0x00].into();
    // let mut max_dist = ([0x00, 0x00, 0x00], 0.0);
    // for (r, g, b) in iproduct!(0x00..=0xFF, 0x00..=0xFF, 0x00..=0xFF) {
    //     let c = [r, g, b];
    //     let dist = HyAB(&c.into(), &c2);
    //     if dist > max_dist.1 {
    //         max_dist = (c, dist);
    //     }
    // }
    // println!("{}\t{}", max_dist.1, to_string(&max_dist.0))
    let bgs = [[0x00, 0x00, 0x00], [0xFF, 0xFF, 0xFF]];
    let backgrounds = bgs.iter().map(|c| (*c).into()).collect_vec();
    let color_lut = SrgbLut::new(|c| c.into());
    let constraint_lut =
        SrgbLut::new_constraint(&backgrounds, |c1, c2| HyAB(c1, &color_lut.get(c2)));
    // println!("{}\t{}", constraint_lut.get(&[0xff, 0xff, 0xff]), constraint_lut.get(&[0x00, 0x00, 0x00]));

    let num_iter: u64 = 1000000000;

    for big_num in 0..1 {
        let mut colors: Vec<sRGB> = repeat_with(rand::random).take(20).collect_vec();
        let mut score_metric = ConstrainedDistance::new(&colors, &color_lut, &constraint_lut);
        let mut best = (-INFINITY, Vec::new());

        let start_time = Instant::now();
        for _it in (0..num_iter).progress_with(
            ProgressBar::new(num_iter).with_style(ProgressStyle::with_template(
                "{elapsed_precise}/{duration_precise} {wide_bar} {percent:>02}% {pos}/{len} {per_sec}",
            )
            .unwrap(),
        )) {
            let (i, j, score) = score_metric.get_min_score();
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
            score_metric.update(index, colors[index]);
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
