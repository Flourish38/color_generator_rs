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

pub trait ScoreMetric {
    fn get_min_score(&self) -> (usize, usize, f32);

    fn update(&mut self, i: usize, color: sRGB);
}

pub struct ConstrainedDistance<'a> {
    colors: &'a SrgbLut<Oklab>,
    pre_colors: Vec<Oklab>,
    constraints: &'a SrgbLut<f32>,
    pre_constraints: Vec<f32>,
    scores: Vec<(usize, f32)>,
}

impl<'a> ConstrainedDistance<'a>{
    pub fn new(colors: &Vec<sRGB>, lut: &'a SrgbLut<Oklab>, constraints: &'a SrgbLut<f32>) -> ConstrainedDistance<'a> {
        let pre_colors = colors.iter().map(|c| lut.get(c)).collect_vec();
        let pre_constraints = colors.iter().map(|c| constraints.get(c)).collect_vec();
        let scores = get_scores_constrained(&pre_colors, &pre_constraints, &HyAB);
        ConstrainedDistance {
            colors: lut,
            pre_colors: pre_colors,
            constraints: constraints,
            pre_constraints: pre_constraints,
            scores: scores,
        }
    }
}

impl<'a> ScoreMetric for ConstrainedDistance<'a>{
    fn get_min_score(&self) -> (usize, usize, f32) {
        get_min_score(&self.scores)
    }

    fn update(&mut self, i: usize, color: sRGB) {
        self.pre_colors[i] = self.colors.get(&color);
        self.pre_constraints[i] = self.constraints.get(&color);
        update_scores_constrained(&mut self.scores, i, &self.pre_colors, &self.pre_constraints, &HyAB);
    }
}