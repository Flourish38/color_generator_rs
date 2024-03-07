use crate::color::*;
use crate::score::*;
use itertools::Itertools;

use crate::color::{sRGB, as_index};
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
        Self::new(|c| backgrounds.iter()
            .map(|bg| f(bg, &c))
            .min_by(|x, y| x.partial_cmp(y).unwrap())
            .unwrap()
        )
    }
}

pub trait ScoreMetric {
    fn get_min_score(&self) -> (usize, usize, f32);

    fn update(&mut self, i: usize, color: sRGB);
}

pub struct ConstrainedDistance<'a, 'b> {
    color_lut: &'a SrgbLut<Oklab>,
    pre_colors: Vec<Oklab>,
    constraint_lut: &'b SrgbLut<f32>,
    pre_constraints: Vec<f32>,
    scores: Vec<(usize, f32)>,
}

impl<'a, 'b> ConstrainedDistance<'a, 'b>{
    pub fn new(colors: &Vec<sRGB>, color_lut: &'a SrgbLut<Oklab>, constraint_lut: &'b SrgbLut<f32>) -> ConstrainedDistance<'a, 'b> {
        let pre_colors = colors.iter().map(|c| color_lut.get(c)).collect_vec();
        let pre_constraints = colors.iter().map(|c| constraint_lut.get(c)).collect_vec();
        let scores = get_scores_constrained(&pre_colors, &pre_constraints, &HyAB);
        ConstrainedDistance {
            color_lut: color_lut,
            pre_colors: pre_colors,
            constraint_lut: constraint_lut,
            pre_constraints: pre_constraints,
            scores: scores,
        }
    }
}

impl<'a, 'b> ScoreMetric for ConstrainedDistance<'a, 'b>{
    fn get_min_score(&self) -> (usize, usize, f32) {
        get_min_score(&self.scores)
    }

    fn update(&mut self, i: usize, color: sRGB) {
        self.pre_colors[i] = self.color_lut.get(&color);
        self.pre_constraints[i] = self.constraint_lut.get(&color);
        update_scores_constrained(&mut self.scores, i, &self.pre_colors, &self.pre_constraints, &HyAB);
    }
}