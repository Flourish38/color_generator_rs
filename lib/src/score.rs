use std::f32::INFINITY;

use itertools::enumerate;

fn parent(i: usize) -> usize {
    (i - 1) / 2
}

fn left(i: usize) -> usize {
    2 * i + 1
}

fn right(i: usize) -> usize {
    2 * i + 2
}

type ScoreHeap = Vec<(f32, usize)>;
pub struct Scores {
    index: Vec<usize>,
    heap: ScoreHeap,
}

impl Scores {
    fn percolate_up(&mut self, mut i: usize) {
        loop {
            if i == 0 {
                // println!("up=0");
                break;
            }
            let p = parent(i);
            if self.heap[p].0 <= self.heap[i].0 {
                // println!("up_sorted");
                break;
            }
            self.swap_heap(i, p);
            i = p;
        }
    }

    fn percolate_down(&mut self, mut i: usize) {
        let i_val = self.heap[i].0;
        loop {
            let l = left(i);
            let len = self.heap.len();
            if l >= len {
                // println!("down_end");
                break;
            }
            let r = right(i);
            let l_val = self.heap[l].0;
            let (min_index, min_val) = if r < len && self.heap[r].0 < l_val {
                (r, self.heap[r].0)
            } else {
                (l, l_val)
            };
            if i_val <= min_val {
                // println!("down_sorted {i} {i_val} {min_val}");
                break;
            }
            self.swap_heap(i, min_index);
            i = min_index;
        }
    }

    fn swap_heap(&mut self, i: usize, j: usize) {
        self.heap.swap(i, j);
        self.index.swap(self.heap[i].1, self.heap[j].1);
    }

    pub fn new(data: &Vec<f32>) -> Self {
        let len = data.len();
        assert!(len > 0);
        let mut scores = Self {
            index: (0..len).collect(),
            heap: Vec::with_capacity(len),
        };
        for i in 0..len {
            scores.heap.push((data[i], i));
            scores.percolate_up(i);
        }
        scores
    }

    pub fn get_min_score(&self) -> (f32, usize) {
        self.heap[0]
    }

    pub fn update(&mut self, i: usize, val: f32) {
        let index = self.index[i];
        // println!("{i}, {index}");
        let old_val = self.heap[index].0;
        self.heap[index] = (val, i);
        if val < old_val {
            self.percolate_up(index);
        } else {
            self.percolate_down(index);
        }
    }
}

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

pub fn get_score_constrained<T, F: Fn(&T, &T) -> f32>(
    i: usize,
    pre_colors: &Vec<T>,
    constraint: f32,
    dist: F,
) -> (usize, f32) {
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

pub fn get_scores_constrained<T, F: Fn(&T, &T) -> f32>(
    pre_colors: &Vec<T>,
    pre_constraints: &Vec<f32>,
    dist: &F,
) -> Vec<(usize, f32)> {
    let mut scores = Vec::with_capacity(pre_colors.len());
    for i in 0..pre_colors.len() {
        scores.push(get_score_constrained(
            i,
            pre_colors,
            pre_constraints[i],
            dist,
        ));
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
        scores[updated_index] = get_score_constrained(
            updated_index,
            pre_colors,
            pre_constraints[updated_index],
            dist,
        )
    }
}

#[cfg(test)]
mod tests {
    use std::iter::repeat_with;

    use itertools::Itertools;
    use rand::{random, thread_rng, Rng};

    use super::*;

    fn verify_invariants(scores: &Scores, data: &Vec<f32>) -> Result<(), String> {
        let len = data.len();
        if len != scores.heap.len() || len != scores.index.len() {
            return Err(format!(
                "Length mismatch! {} {} {}",
                len,
                scores.heap.len(),
                scores.index.len()
            ));
        }
        if &scores.get_min_score().0
            != data
                .iter()
                .min_by(|x, y| x.partial_cmp(y).unwrap())
                .unwrap()
        {
            return Err(format!(
                "Mininmum mismatch! {} {}",
                scores.get_min_score().0,
                data.iter()
                    .min_by(|x, y| x.partial_cmp(y).unwrap())
                    .unwrap()
            ));
        }
        for i in 0..len {
            let entry = scores.heap[scores.index[i]];
            if entry != (data[i], i) {
                return Err(format!(
                    "Heap has wrong entry! ({}, {}) {:?} {}",
                    data[i], i, entry, scores.index[i]
                ));
            }
        }
        for i in 0..len {
            let val = scores.heap[i].0;
            let l = left(i);
            let r = right(i);

            if (l < len && scores.heap[l].0 < val) || (r < len && scores.heap[r].0 < val) {
                return Err(format!(
                    "Heap property not satisfied! {} {} {} {} {} {} {}",
                    scores.heap[i].1, i, val, l, scores.heap[l].0, r, scores.heap[r].0
                ));
            }
        }
        return Ok(());
    }

    fn heap_test(n: usize) {
        let mut data = repeat_with(random::<f32>).take(n).collect_vec();
        let mut scores = Scores::new(&data);
        match verify_invariants(&scores, &data) {
            Err(e) => panic!("{e}"),
            _ => (),
        }
        for iter in 0..1000 {
            let i = thread_rng().gen_range(0..n);
            let val = random();
            data[i] = val;
            scores.update(i, val);
            match verify_invariants(&scores, &data) {
                Err(e) => panic!("{iter} {i} {val} {e}"),
                _ => (),
            }
        }
    }

    #[test]
    fn test_heap() {
        for p in 0..5 {
            let n = 10_usize.pow(p);
            heap_test(n);
            heap_test(n + 1);
        }
        for n in 1..1000 {
            let data = repeat_with(random::<f32>).take(n).collect_vec();
            let scores = Scores::new(&data);
            match verify_invariants(&scores, &data) {
                Err(e) => panic!("{n} {e}"),
                _ => (),
            }
        }
    }
}
