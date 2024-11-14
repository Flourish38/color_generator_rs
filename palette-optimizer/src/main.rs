extern crate color_lib;
extern crate palette_visualizer;

mod metric;
mod optimizer;
mod score;
mod update;

use bitvec::{bitvec, index, vec::BitVec};
use color_lib::*;
use indicatif::{ProgressBar, ProgressStyle};
use itertools::{iproduct, Itertools};
use metric::*;
use optimizer::Optimizer;
use palette_visualizer::save_svg;
use std::{
    collections::{HashMap, HashSet, VecDeque},
    iter::repeat_with,
    ops::{Not, SubAssign},
    time::Instant,
};

#[allow(dead_code)]
fn breakpoint() {
    let mut buf = String::new();
    std::io::stdin().read_line(&mut buf).unwrap();
}

enum Color {
    Red = 0,
    Green = 1,
    Blue = 2,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Face {
    RNeg = 0,
    RPos = 1,
    GNeg = 2,
    GPos = 3,
    BNeg = 4,
    BPos = 5,
}
const R_NEG: Face = Face::RNeg;
const R_POS: Face = Face::RPos;
const G_NEG: Face = Face::GNeg;
const G_POS: Face = Face::GPos;
const B_NEG: Face = Face::BNeg;
const B_POS: Face = Face::BPos;

impl Face {
    const VARIANTS: [Face; 6] = [R_NEG, R_POS, G_NEG, G_POS, B_NEG, B_POS];

    fn offset(&self, c: sRGB) -> sRGB {
        let [r, g, b] = c;
        match self {
            Face::RNeg => [r.saturating_add(1), g, b],
            Face::RPos => [r.saturating_sub(1), g, b],
            Face::GNeg => [r, g.saturating_add(1), b],
            Face::GPos => [r, g.saturating_sub(1), b],
            Face::BNeg => [r, g, b.saturating_add(1)],
            Face::BPos => [r, g, b.saturating_sub(1)],
        }
    }

    fn is_at_limit(&self, extents: [u8; 6]) -> bool {
        let index = *self as usize;
        if self.is_pos() {
            extents[index] == 0xFF
        } else {
            extents[index] == 0x00
        }
    }

    fn is_pos(&self) -> bool {
        match self {
            Face::RNeg | Face::GNeg | Face::BNeg => false,
            Face::RPos | Face::GPos | Face::BPos => true,
        }
    }

    fn sign(&self) -> i8 {
        if self.is_pos() {
            1
        } else {
            -1
        }
    }

    fn color(&self) -> Color {
        match self {
            Face::RNeg | Face::RPos => Color::Red,
            Face::GNeg | Face::GPos => Color::Green,
            Face::BNeg | Face::BPos => Color::Blue,
        }
    }

    fn iter(&self, extents: [u8; 6]) -> impl Iterator<Item = sRGB> {
        let index = *self as usize;
        match self.color() {
            Color::Red => iproduct!(
                extents[index]..=extents[index],
                extents[G_NEG as usize]..=extents[G_POS as usize],
                extents[B_NEG as usize]..=extents[B_POS as usize]
            ),
            Color::Green => iproduct!(
                extents[R_NEG as usize]..=extents[R_POS as usize],
                extents[index]..=extents[index],
                extents[B_NEG as usize]..=extents[B_POS as usize]
            ),
            Color::Blue => iproduct!(
                extents[R_NEG as usize]..=extents[R_POS as usize],
                extents[G_NEG as usize]..=extents[G_POS as usize],
                extents[index]..=extents[index]
            ),
        }
        .map(|(r, g, b)| [r, g, b])
    }

    fn is_on_face(&self, c: sRGB, extents: [u8; 6]) -> bool {
        c[self.color() as usize] == extents[*self as usize]
    }
}

struct FaceMasks {
    r_neg: BitVec,
    r_pos: BitVec,
    g_neg: BitVec,
    g_pos: BitVec,
    b_neg: BitVec,
    b_pos: BitVec,
}

impl FaceMasks {
    fn new() -> Self {
        Self {
            r_neg: BitVec::repeat(false, 1 << 16),
            r_pos: BitVec::repeat(false, 1 << 16),
            g_neg: BitVec::repeat(false, 1 << 16),
            g_pos: BitVec::repeat(false, 1 << 16),
            b_neg: BitVec::repeat(false, 1 << 16),
            b_pos: BitVec::repeat(false, 1 << 16),
        }
    }

    fn r_index(c: sRGB) -> usize {
        ((c[1] as usize) << 8) & c[2] as usize
    }

    fn g_index(c: sRGB) -> usize {
        ((c[0] as usize) << 8) & c[2] as usize
    }

    fn b_index(c: sRGB) -> usize {
        ((c[0] as usize) << 8) & c[1] as usize
    }

    fn set(&mut self, face: Face, c: sRGB, value: bool) {
        match face {
            Face::RNeg => self.r_neg.set(Self::r_index(c), value),
            Face::RPos => self.r_pos.set(Self::r_index(c), value),
            Face::GNeg => self.g_neg.set(Self::g_index(c), value),
            Face::GPos => self.g_pos.set(Self::g_index(c), value),
            Face::BNeg => self.b_neg.set(Self::b_index(c), value),
            Face::BPos => self.b_pos.set(Self::b_index(c), value),
        }
    }

    fn get(&self, face: Face, c: sRGB, offset: isize) -> Option<bool> {
        match face {
            Face::RNeg => Self::r_index(c)
                .checked_add_signed(offset)
                .and_then(|index| self.r_neg.get(index)),
            Face::RPos => Self::r_index(c)
                .checked_add_signed(offset)
                .and_then(|index| self.r_pos.get(index)),
            Face::GNeg => Self::g_index(c)
                .checked_add_signed(offset)
                .and_then(|index| self.g_neg.get(index)),
            Face::GPos => Self::g_index(c)
                .checked_add_signed(offset)
                .and_then(|index| self.g_pos.get(index)),
            Face::BNeg => Self::b_index(c)
                .checked_add_signed(offset)
                .and_then(|index| self.b_neg.get(index)),
            Face::BPos => Self::b_index(c)
                .checked_add_signed(offset)
                .and_then(|index| self.b_pos.get(index)),
        }
        .and_then(|br| Some(*br))
    }

    fn check_behind(&self, face: Face, c: sRGB) -> bool {
        self.get(face, c, -256).unwrap_or(false)
            || self.get(face, c, -1).unwrap_or(false)
            || self.get(face, c, 0).unwrap_or(false)
    }

    fn check_ahead(&self, face: Face, c: sRGB) -> bool {
        (self.get(face, c, 256).unwrap_or(false) || self.get(face, c, 1).unwrap_or(false))
            && !self.get(face, c, 0).unwrap_or(false)
    }
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
        let mut extents = [
            c_start[0], c_start[0], c_start[1], c_start[1], c_start[2], c_start[2],
        ];
        let mut expand = [true; 6];
        let mut face_masks = FaceMasks::new();
        for face in Face::VARIANTS {
            face_masks.set(face, c_start, true);
        }
        let mut update_set = HashSet::new();
        'outer: loop {
            // These values shouldn't need to be initialized, but whatever
            let mut face = R_NEG;
            let mut face_iter = face.iter(extents);

            for f in Face::VARIANTS.iter().rev() {
                let i = *f as usize;
                if expand[i] && !f.is_at_limit(extents) {
                    expand[i] = false;
                    face = *f;
                    extents[i] = extents[i].saturating_add_signed(f.sign());
                    face_iter = f.iter(extents);
                    break;
                } else if f == Face::VARIANTS.first().unwrap() {
                    break 'outer;
                }
            }
            for c in face_iter {
                let index = as_index(&c);
                if face_masks.check_ahead(face, c) {
                    update_set.insert(face.offset(c));
                }
                if !self.inside[index] {
                    face_masks.set(face, c, false);
                    continue;
                }
                let still_inside = f(c);
                if still_inside {
                    if face_masks.check_behind(face, c) {
                        update_set.insert(c);
                    }
                    face_masks.set(face, c, false);
                } else {
                    self.inside.set(index, false);
                    self.surface.remove(&c);
                    self.edge.remove(&c);
                    self.corner.remove(&c);
                    if !face_masks.get(face, c, 0).unwrap_or(true) {
                        update_set.insert(face.offset(c));
                    }
                }

                for f in Face::VARIANTS {
                    if f.is_on_face(c, extents) {
                        if !still_inside {
                            face_masks.set(f, c, true);
                            expand[f as usize] = true;
                        }
                    }
                }
            }
        }
        println!(
            "{}:{}\t{}:{}\t{}:{}",
            extents[R_NEG as usize],
            extents[R_POS as usize],
            extents[G_NEG as usize],
            extents[G_POS as usize],
            extents[B_NEG as usize],
            extents[B_POS as usize]
        );
        println!("{}", update_set.len());
        for c in update_set {
            if !self.surface.insert(c) {
                if !self.edge.insert(c) {
                    self.corner.insert(c);
                }
            }
        }
        // println!("{:#?}", self.corner);
    }
}

fn main() {
    let bgs = [[0xFF, 0xFF, 0xFF], [0x00, 0x00, 0x00]];
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
