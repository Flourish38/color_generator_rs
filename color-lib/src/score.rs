use std::{f32::INFINITY, fmt::Debug};

use crate::color::{HyAB, Oklab};

fn parent(i: usize) -> usize {
    (i - 1) / 2
}

fn left(i: usize) -> usize {
    2 * i + 1
}

fn right(i: usize) -> usize {
    2 * i + 2
}

pub trait ScoreIndex: Debug + Copy {
    fn get(&self) -> usize;
}

impl ScoreIndex for usize {
    fn get(&self) -> usize {
        *self
    }
}

impl ScoreIndex for (usize, usize) {
    fn get(&self) -> usize {
        self.0
    }
}

type ScoreHeap<T> = Vec<(f32, T)>;
pub struct Scores<T>
where
    T: ScoreIndex,
{
    index: Vec<usize>,
    heap: ScoreHeap<T>,
}

impl<T: ScoreIndex> Scores<T> {
    fn percolate_up(&mut self, mut i: usize) {
        loop {
            if i == 0 {
                break;
            }
            let p = parent(i);
            if self.heap[p].0 <= self.heap[i].0 {
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
                break;
            }
            self.swap_heap(i, min_index);
            i = min_index;
        }
    }

    fn swap_heap(&mut self, i: usize, j: usize) {
        self.heap.swap(i, j);
        self.index.swap(self.heap[i].1.get(), self.heap[j].1.get());
    }

    pub fn get_min_score(&self) -> (f32, T) {
        self.heap[0]
    }

    pub fn update(&mut self, i: T, val: f32) {
        let index = self.index[i.get()];
        let old_val = self.heap[index].0;
        self.heap[index] = (val, i);
        if val < old_val {
            self.percolate_up(index);
        } else {
            self.percolate_down(index);
        }
    }
}

impl Scores<usize> {
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
}

impl Scores<(usize, usize)> {
    pub fn new_pairs(data: &Vec<(f32, usize)>) -> Self {
        let len = data.len();
        assert!(len > 0);
        let mut scores = Self {
            index: (0..len).collect(),
            heap: Vec::with_capacity(len),
        };
        for i in 0..len {
            scores.heap.push((data[i].0, (i, data[i].1)));
            scores.percolate_up(i);
        }
        scores
    }
}

pub fn get_pair_score(i: usize, pre_colors: &Vec<Oklab>) -> (f32, usize) {
    let c = &pre_colors[i];
    let mut score = (INFINITY, i);
    for j in (i + 1)..pre_colors.len() {
        let dist = HyAB(c, &pre_colors[j]);
        if dist < score.0 {
            score = (dist, j);
        }
    }
    return score;
}

pub fn get_pair_scores(pre_colors: &Vec<Oklab>) -> Vec<(f32, usize)> {
    let mut scores = Vec::with_capacity(pre_colors.len() - 1);
    for i in 0..(pre_colors.len() - 1) {
        scores.push(get_pair_score(i, pre_colors));
    }
    return scores;
}

#[cfg(test)]
mod tests {
    use std::iter::repeat_with;

    use itertools::Itertools;
    use rand::{random, thread_rng, Rng};

    use super::*;

    fn verify_invariants<T: ScoreIndex + Copy>(
        scores: &Scores<T>,
        data: &Vec<f32>,
    ) -> Result<(), String> {
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
            if entry.0 != data[i] || entry.1.get() != i {
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
                    "Heap property not satisfied! {:?} {} {} {} {} {} {}",
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
