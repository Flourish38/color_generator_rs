use crate::color::*;
use crate::score::*;
use itertools::Itertools;

use crate::color::{as_index, sRGB};
use itertools::iproduct;

pub struct SrgbLut<T> {
    data: Vec<T>,
}

impl<T: Copy> SrgbLut<T> {
    pub fn new(f: impl Fn(sRGB) -> T) -> Self {
        let mut data = Vec::with_capacity(1 << 24);
        // I wish there was an easy way to allow this to be parallel,
        // But it is fast enough that it isn't a significant issue.
        for (r, g, b) in iproduct!(0x00..=0xFF, 0x00..=0xFF, 0x00..=0xFF) {
            let c = [r, g, b];
            data.push(f(c))
        }
        Self { data: data }
    }

    pub fn get(&self, c: &sRGB) -> T {
        self.data[as_index(c)]
    }
}

impl SrgbLut<f32> {
    pub fn new_constraint<T2>(backgrounds: &Vec<T2>, f: impl Fn(&T2, &sRGB) -> f32) -> Self {
        Self::new(|c| {
            backgrounds
                .iter()
                .map(|bg| f(bg, &c))
                .min_by(|x, y| x.partial_cmp(y).unwrap())
                .unwrap()
        })
    }
}

pub trait ScoreMetric<T: ScoreIndex> {
    fn get_min_score(&self) -> (f32, T);

    fn update(&mut self, updated_index: usize, updated_color: &sRGB);

    fn test_improvement(&self, updated_index: usize, updated_color: &sRGB) -> bool;
}

// pub struct ConstrainedDistance<'a, 'b> {
//     color_lut: &'a SrgbLut<Oklab>,
//     pre_colors: Vec<Oklab>,
//     constraint_lut: &'b SrgbLut<f32>,
//     pre_constraints: Vec<f32>,
//     scores: Vec<(usize, f32)>,
// }
// // TODO: implement keyed priority queue - ish thing for storing scores.
// // It is definitely the correct data structure.

// impl<'a, 'b> ConstrainedDistance<'a, 'b> {
//     pub fn new(
//         colors: &Vec<sRGB>,
//         color_lut: &'a SrgbLut<Oklab>,
//         constraint_lut: &'b SrgbLut<f32>,
//     ) -> ConstrainedDistance<'a, 'b> {
//         let pre_colors = colors.iter().map(|c| color_lut.get(c)).collect_vec();
//         let pre_constraints = colors.iter().map(|c| constraint_lut.get(c)).collect_vec();
//         let scores = get_scores_constrained(&pre_colors, &pre_constraints, &HyAB);
//         ConstrainedDistance {
//             color_lut: color_lut,
//             pre_colors: pre_colors,
//             constraint_lut: constraint_lut,
//             pre_constraints: pre_constraints,
//             scores: scores,
//         }
//     }
// }

// impl<'a, 'b> ScoreMetric<(usize, usize)> for ConstrainedDistance<'a, 'b> {
//     fn get_min_score(&self) -> (usize, usize, f32) {
//         get_min_score(&self.scores)
//     }

//     fn update(&mut self, i: usize, color: sRGB) {
//         self.pre_colors[i] = self.color_lut.get(&color);
//         self.pre_constraints[i] = self.constraint_lut.get(&color);
//         update_scores_constrained(
//             &mut self.scores,
//             i,
//             &self.pre_colors,
//             &self.pre_constraints,
//             &HyAB,
//         );
//     }

//     fn test_improvement(&self, i: usize, updated_index: usize, updated_color: &sRGB) -> bool {
//         let (j, prev_score) = self.scores[i];
//         if i == j {
//             return self.constraint_lut.get(&updated_color) > prev_score;
//         }
//         let new_color = self.color_lut.get(&updated_color);
//         let other_index = if i == updated_index { j } else { i };
//         let other_color = self.pre_colors[other_index];
//         let new_score = HyAB(&new_color, &other_color);
//         return new_score > prev_score;
//     }
// }

// pub struct ConstraintOnly<'a> {
//     constraint_lut: &'a SrgbLut<f32>,
//     scores: Vec<f32>,
// }

// impl<'a> ConstraintOnly<'a> {
//     pub fn new(colors: &Vec<sRGB>, constraint_lut: &'a SrgbLut<f32>) -> Self {
//         let scores = colors.iter().map(|c| constraint_lut.get(c)).collect_vec();
//         ConstraintOnly {
//             constraint_lut: constraint_lut,
//             scores: scores,
//         }
//     }
// }

// impl<'a> ScoreMetric for ConstraintOnly<'a> {
//     fn get_min_score(&self) -> (usize, usize, f32) {
//         let (i, val) = enumerate(&self.scores)
//             .min_by(|x, y| x.1.partial_cmp(y.1).unwrap())
//             .unwrap();
//         (i, i, *val)
//     }

//     fn update(&mut self, i: usize, color: sRGB) {
//         self.scores[i] = self.constraint_lut.get(&color);
//     }

//     fn test_improvement(&self, i: usize, updated_index: usize, updated_color: &sRGB) -> bool {
//         let _ = i;
//         self.constraint_lut.get(updated_color) > self.scores[updated_index]
//     }
// }

pub struct Constraint<'a> {
    constraint_lut: &'a SrgbLut<f32>,
    scores: Scores<usize>,
}

impl<'a> Constraint<'a> {
    pub fn new(colors: &Vec<sRGB>, constraint_lut: &'a SrgbLut<f32>) -> Self {
        let data = colors.iter().map(|c| constraint_lut.get(c)).collect_vec();
        Constraint {
            constraint_lut: constraint_lut,
            scores: Scores::new(&data),
        }
    }
}

impl<'a> ScoreMetric<usize> for Constraint<'a> {
    fn get_min_score(&self) -> (f32, usize) {
        self.scores.get_min_score()
    }

    fn update(&mut self, updated_index: usize, updated_color: &sRGB) {
        self.scores
            .update(updated_index, self.constraint_lut.get(updated_color));
    }

    fn test_improvement(&self, _updated_index: usize, updated_color: &sRGB) -> bool {
        self.constraint_lut.get(updated_color) > self.scores.get_min_score().0
    }
}

pub struct PairDistance<'a> {
    color_lut: &'a SrgbLut<Oklab>,
    pre_colors: Vec<Oklab>,
    pre_scores: Vec<(f32, usize)>,
    scores: Scores<(usize, usize)>,
}

impl<'a> PairDistance<'a> {
    pub fn new(colors: &Vec<sRGB>, color_lut: &'a SrgbLut<Oklab>) -> Self {
        let pre_colors = colors.iter().map(|c| color_lut.get(c)).collect_vec();
        let pre_scores = get_pair_scores(&pre_colors);
        let scores = Scores::new_pairs(&pre_scores);
        Self {
            color_lut: color_lut,
            pre_colors: pre_colors,
            pre_scores: pre_scores,
            scores: scores,
        }
    }

    fn update_pair_score(&mut self, i: usize) {
        let (val, ind) = get_pair_score(i, &self.pre_colors);
        self.pre_scores[i] = (val, ind);
        self.scores.update((i, ind), val);
    }
}

impl<'a> ScoreMetric<(usize, usize)> for PairDistance<'a> {
    fn get_min_score(&self) -> (f32, (usize, usize)) {
        self.scores.get_min_score()
    }

    fn update(&mut self, updated_index: usize, updated_color: &sRGB) {
        let new_color = self.color_lut.get(updated_color);
        self.pre_colors[updated_index] = new_color;

        // Recompute scores of indexes before updated_index
        for i in 0..updated_index {
            let (prev_score, prev_index) = self.pre_scores[i];
            let score = HyAB(&new_color, &self.pre_colors[i]);
            if score < prev_score {
                self.pre_scores[i] = (score, updated_index);
                self.scores.update((i, updated_index), score)
            } else if prev_index == updated_index {
                // Have to recompute score for this element
                self.update_pair_score(i);
            } // else, no need to change it
        }

        // Recompute score of updated_index
        if updated_index < self.pre_scores.len() {
            self.update_pair_score(updated_index);
        }
    }

    fn test_improvement(&self, updated_index: usize, updated_color: &sRGB) -> bool {
        let new_color = self.color_lut.get(updated_color);
        let (old_score, (i, j)) = self.scores.get_min_score();
        let color = if updated_index == i {
            self.pre_colors[j]
        } else {
            self.pre_colors[i]
        };
        return HyAB(&new_color, &color) > old_score;
    }
}
