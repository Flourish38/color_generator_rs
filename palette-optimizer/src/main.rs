extern crate lib;

use indicatif::{ProgressBar, ProgressStyle};
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
    // let backgrounds = bgs.iter().map(|c| (*c).into()).collect_vec();
    // let color_lut = SrgbLut::new(|c| c.into());
    // let constraint_lut =
    //     SrgbLut::new_constraint(&backgrounds, |c1, c2| HyAB(c1, &color_lut.get(c2)));
    // println!("{}\t{}", constraint_lut.get(&[0xff, 0xff, 0xff]), constraint_lut.get(&[0x00, 0x00, 0x00]));
    let apca_constraint_lut = SrgbLut::new_constraint(&bgs.to_vec(), |c1, c2| APCA(c2, c1));

    let num_iter: u64 = 10000000;
    let update_freq: u64 = 1000000;

    for big_num in 0..5 {
        let mut colors: Vec<sRGB> = repeat_with(rand::random).take(1).collect_vec();
        let mut score_metric = ConstraintOnly::new(&colors, &apca_constraint_lut);
        let mut best = (-INFINITY, Vec::new());

        let start_time = Instant::now();
        let pb = ProgressBar::new(num_iter).with_style(ProgressStyle::with_template(
            "{elapsed_precise}/{duration_precise} {wide_bar} {percent:>02}% {pos}/{len} {per_sec}",
        ).unwrap());

        for _it in 0..num_iter {
            if _it % update_freq == 0 {
                pb.inc(update_freq)
            }
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
            let (mut index, mut new_color) = update_color(&colors, (i, j));
            if !score_metric.test_improvement(i, index, &new_color) {
                (index, new_color) = update_color(&colors, (i, j));
            }
            colors[index] = new_color;
            score_metric.update(index, colors[index]);
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
