use std::f32::INFINITY;

use itertools::enumerate;

pub fn get_score<T, F: Fn(&T, &T) -> f32>(i: usize, pre_colors: &Vec<T>, dist: F) -> (usize, f32) {
    let c = &pre_colors[i];
    let mut score = (i, INFINITY);
    for j in (i + 1)..pre_colors.len() {
        let dist = dist(c, &pre_colors[j]);
        if dist < score.1 {
            score = (j, dist);
        }
    }
    return score;
}

pub fn get_scores<T, F: Fn(&T, &T) -> f32>(pre_colors: &Vec<T>, dist: &F) -> Vec<(usize, f32)> {
    let mut scores = Vec::with_capacity(pre_colors.len() - 1);
    for i in 0..(pre_colors.len() - 1) {
        scores.push(get_score(i, pre_colors, dist));
    }
    return scores;
}

pub fn get_score_constrained<T, F: Fn(&T, &T) -> f32>(i: usize,  pre_colors: &Vec<T>, constraint: f32, dist: F) -> (usize, f32) {
    let c = &pre_colors[i];
    let mut score = (i, constraint);
    for j in (i + 1)..pre_colors.len() {
        let dist = dist(c, &pre_colors[j]);
        if dist < score.1 {
            score = (j, dist);
        }
    }
    return score;
}

pub fn get_scores_constrained<T, F: Fn(&T, &T) -> f32>(pre_colors: &Vec<T>, pre_constraints: &Vec<f32>, dist: &F) -> Vec<(usize, f32)> {
    let mut scores = Vec::with_capacity(pre_colors.len());
    for i in 0..pre_colors.len() {
        scores.push(get_score_constrained(i, pre_colors, pre_constraints[i], dist));
    }
    return scores;
}

pub fn get_min_score(scores: &Vec<(usize, f32)>) -> (usize, usize, f32) {
    let mut output = (0, 0, INFINITY);
    for (i, (j, val)) in enumerate(scores) {
        if val < &output.2 {
            output = (i, *j, *val);
        }
    }
    output
}

pub fn update_scores<T, F: Fn(&T, &T) -> f32>(
    scores: &mut Vec<(usize, f32)>,
    updated_index: usize,
    pre_colors: &Vec<T>,
    dist: &F,
) {
    let c_updated = &pre_colors[updated_index];

    // Recompute scores of indexes before updated_index
    for i in 0..updated_index {
        let (prev_index, prev_score) = scores[i];
        let score = dist(c_updated, &pre_colors[i]);
        if score < prev_score {
            scores[i] = (updated_index, score);
        } else if prev_index == updated_index {
            // Have to recompute score for this element
            scores[i] = get_score(i, pre_colors, dist);
        } // else, no need to change it
    }

    // Recompute score of updated_index
    if updated_index < scores.len() {
        scores[updated_index] = get_score(updated_index, pre_colors, dist)
    }
}

pub fn update_scores_constrained<T, F: Fn(&T, &T) -> f32>(
    scores: &mut Vec<(usize, f32)>,
    updated_index: usize,
    pre_colors: &Vec<T>,
    pre_constraints: &Vec<f32>,
    dist: &F,
) {
    let c_updated = &pre_colors[updated_index];

    // Recompute scores of indexes before updated_index
    for i in 0..updated_index {
        let (prev_index, prev_score) = scores[i];
        let score = dist(c_updated, &pre_colors[i]);
        if score < prev_score {
            scores[i] = (updated_index, score);
        } else if prev_index == updated_index {
            // Have to recompute score for this element
            scores[i] = get_score_constrained(i, pre_colors, pre_constraints[i], dist);
        } // else, no need to change it
    }

    // Recompute score of updated_index
    if updated_index < scores.len() {
        scores[updated_index] = get_score_constrained(updated_index, pre_colors, pre_constraints[updated_index], dist)
    }
}