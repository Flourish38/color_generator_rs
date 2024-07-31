use crate::metric::{Constraint, PairDistance, ScoreMetric};
use crate::update::{update_color, update_color_pair};
use color_lib::sRGB;

#[derive(Clone, Copy)]
enum Metric {
    Pair(usize, (usize, usize)),
    Const(usize, usize),
}

pub struct Optimizer<'a> {
    colors: Vec<sRGB>,
    min_score_metric: Metric,
    pair_metrics: Vec<(f32, PairDistance<'a>)>,
    constraints: Vec<(f32, Constraint<'a>)>,
    best_colors: (f32, Vec<sRGB>),
}

impl<'a> Optimizer<'a> {
    pub fn new(
        pair_metrics: Vec<(f32, PairDistance<'a>)>,
        constraints: Vec<(f32, Constraint<'a>)>,
        colors: Vec<sRGB>,
    ) -> Self {
        let best_colors = colors.clone();
        let min_score = pair_metrics
            .iter()
            .enumerate()
            .map(|(i, (w, pm))| {
                let (s, pair_index) = pm.get_min_score();
                (s / w, Metric::Pair(i, pair_index))
            })
            .chain(constraints.iter().enumerate().map(|(i, (w, c))| {
                let (s, j) = c.get_min_score();
                (s / w, Metric::Const(i, j))
            }))
            .min_by(|(s1, _), (s2, _)| s1.partial_cmp(s2).unwrap())
            .unwrap();
        Self {
            colors: colors,
            min_score_metric: min_score.1,
            pair_metrics: pair_metrics,
            constraints: constraints,
            best_colors: (min_score.0, best_colors),
        }
    }

    pub fn update(&mut self) {
        let m = self.min_score_metric;
        let (index, c) = match m {
            Metric::Pair(i, pair) => {
                let pair_metric = &self.pair_metrics[i].1;
                let (mut index, mut c) = update_color_pair(&self.colors, pair);
                if !pair_metric.test_improvement(index, &c) {
                    (index, c) = update_color_pair(&self.colors, pair)
                }
                (index, c)
            }
            Metric::Const(i, index) => {
                let constraint = &self.constraints[i].1;
                let mut c = update_color(&self.colors, index);
                if !constraint.test_improvement(index, &c) {
                    c = update_color(&self.colors, index)
                }
                (index, c)
            }
        };
        self.colors[index] = c;
        let mut min_score = (f32::INFINITY, Metric::Const(0, 0));
        for (i, (w, pair_metric)) in self.pair_metrics.iter_mut().enumerate() {
            pair_metric.update(index, &c);
            let (s, pair_index) = pair_metric.get_min_score();
            let score = s / *w;
            if score < min_score.0 {
                min_score = (score, Metric::Pair(i, pair_index));
            }
        }
        for (i, (w, constraint)) in self.constraints.iter_mut().enumerate() {
            constraint.update(index, &c);
            let (s, j) = constraint.get_min_score();
            let score = s / *w;
            if score < min_score.0 {
                min_score = (score, Metric::Const(i, j));
            }
        }
        self.min_score_metric = min_score.1;
        if min_score.0 > self.best_colors.0 {
            self.best_colors = (min_score.0, self.colors.clone());
        }
    }

    pub fn restore_best(&mut self) {
        self.colors = self.best_colors.1.clone();
        let mut min_score = (f32::INFINITY, Metric::Const(0, 0));
        for (i, (w, pair_metric)) in self.pair_metrics.iter_mut().enumerate() {
            for index in 0..self.colors.len() {
                pair_metric.update(index, &self.colors[index]);
            }
            let (s, pair_index) = pair_metric.get_min_score();
            let score = s / *w;
            if score < min_score.0 {
                min_score = (score, Metric::Pair(i, pair_index));
            }
        }
        for (i, (w, constraint)) in self.constraints.iter_mut().enumerate() {
            for index in 0..self.colors.len() {
                constraint.update(index, &self.colors[index]);
            }
            let (s, j) = constraint.get_min_score();
            let score = s / *w;
            if score < min_score.0 {
                min_score = (score, Metric::Const(i, j));
            }
        }
        self.min_score_metric = min_score.1;
        assert_eq!(min_score.0, self.best_colors.0);
    }

    pub fn get_best_score(&self) -> f32 {
        self.best_colors.0
    }

    pub fn get_best(self) -> (f32, Vec<sRGB>) {
        self.best_colors
    }
}
