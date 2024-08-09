use std::f64::consts::TAU;

use color_lib::*;
use itertools::iproduct;
use num_integer::Roots;
use rand::{distributions, prelude::Distribution, thread_rng};

#[derive(Debug)]
pub struct PairMatrix<T> {
    data: Vec<T>,
}

impl<T: std::fmt::Display> std::fmt::Display for PairMatrix<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let n = PairMatrix::<T>::inverse_triangle(self.data.len());
        let mut out_str = "\t".to_string()
            + (0..n)
                .map(|a| usize::to_string(&a))
                .collect::<Vec<_>>()
                .join("\t")
                .as_str();
        for i in 0..n {
            out_str += format!("\n{}\t", i).as_str();
            for j in 0..i {
                out_str += format!("{:.2}\t", self[(j, i)]).as_str();
            }
        }

        write!(f, "{}", out_str.as_str())
    }
}

impl<T: Default> PairMatrix<T> {
    fn new(n: usize) -> Self {
        Self {
            data: std::iter::repeat_with(Default::default)
                .take(PairMatrix::<T>::triangle_number(n))
                .collect(),
        }
    }
}

impl<T> PairMatrix<T> {
    fn new_populated<T2>(base: &Vec<T2>, f: impl Fn(&T2, &T2) -> T) -> Self {
        let n = base.len();
        let mut data = Vec::with_capacity(PairMatrix::<T>::triangle_number(n));
        for j in 1..n {
            for i in 0..j {
                data.push(f(&base[i], &base[j]));
            }
        }
        Self { data: data }
    }

    // don't tell anyone, but this actually computes the triangle number of n-1.
    fn triangle_number(n: usize) -> usize {
        (n * (n - 1)) / 2
    }

    fn inverse_triangle(tri: usize) -> usize {
        (1 + (1 + 8 * tri).sqrt()) / 2
    }

    fn ordered_pair_to_index((min, max): (usize, usize)) -> usize {
        PairMatrix::<T>::triangle_number(max) + min
    }
}

impl<T: Copy> PairMatrix<T> {
    fn unordered_index(&self, (a, b): (usize, usize)) -> T {
        if a < b {
            self[(a, b)]
        } else {
            self[(b, a)]
        }
    }
}

impl<T> std::ops::Index<(usize, usize)> for PairMatrix<T> {
    type Output = T;

    fn index(&self, index: (usize, usize)) -> &Self::Output {
        &self.data[PairMatrix::<T>::ordered_pair_to_index(index)]
    }
}

impl<T> std::ops::IndexMut<(usize, usize)> for PairMatrix<T> {
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        &mut self.data[PairMatrix::<T>::ordered_pair_to_index(index)]
    }
}

fn adjacency_matrix(ring_sizes: &Vec<usize>, angles: &Vec<f64>) -> PairMatrix<f32> {
    let rings = ring_sizes.len();
    let n = ring_sizes.iter().sum();
    let mut output = PairMatrix::<f32>::new(n);

    let mut prev_ring_sizes = 0;
    for ring in 0..rings {
        if ring_sizes[ring] > 1 {
            output[(prev_ring_sizes, prev_ring_sizes + ring_sizes[ring] - 1)] = 1.0;
            for i in prev_ring_sizes..prev_ring_sizes + ring_sizes[ring] - 1 {
                output[(i, i + 1)] = 1.0;
            }
        }
        if ring < rings - 1 {
            let large_angle = TAU / (ring_sizes[ring] as f64);
            let small_angle = TAU / (ring_sizes[ring + 1] as f64);
            for i in 0..ring_sizes[ring] {
                let start_angle = angles[ring] + (i as f64) * large_angle;
                for j in 0..ring_sizes[ring + 1] {
                    let start_relative =
                        (angles[ring + 1] + (j as f64) * small_angle - start_angle).rem_euclid(TAU);
                    let end_relative = (start_relative + small_angle).rem_euclid(TAU);
                    output[(i + prev_ring_sizes, j + prev_ring_sizes + ring_sizes[ring])] =
                        match (start_relative < large_angle, end_relative < large_angle) {
                            (true, true) => 1.0,
                            (true, false) => (large_angle - start_relative) / small_angle,
                            (false, true) => end_relative / small_angle,
                            (false, false) => 0.0,
                        } as f32;
                }
            }
        }
        prev_ring_sizes += ring_sizes[ring];
    }

    return output;
}

fn color_pair_matrix(colors: &Vec<sRGB>) -> PairMatrix<f32> {
    PairMatrix::<f32>::new_populated(colors, |c1, c2| HyAB(&(*c1).into(), &(*c2).into()))
}

fn compute_score(
    permutation: &Vec<usize>,

    adjacency_matrix: &PairMatrix<f32>,
    color_pair_matrix: &PairMatrix<f32>,
) -> f32 {
    let mut output = 0.0;
    for i in 0..permutation.len() - 1 {
        for j in i + 1..permutation.len() {
            output += adjacency_matrix[(i, j)]
                * color_pair_matrix.unordered_index((permutation[i], permutation[j]));
        }
    }
    return output;
}

fn get_c(x: &sRGB) -> f32 {
    Oklch::from(*x).C
}

fn get_h(x: &sRGB) -> f32 {
    Oklch::from(*x).h
}

fn sort_colors_naive(colors: &mut Vec<sRGB>, ring_sizes: &Vec<usize>) {
    colors.sort_unstable_by(|c1, c2| get_c(c1).partial_cmp(&get_c(c2)).unwrap());
    let mut start_index = 0;
    for ring in ring_sizes {
        colors[start_index..start_index + ring]
            .sort_unstable_by(|c1, c2| get_h(c1).partial_cmp(&get_h(c2)).unwrap());
        start_index += ring;
    }
}

fn sort_colors_simple(colors: &Vec<sRGB>, ring_sizes: &Vec<usize>) -> Vec<sRGB> {
    let mut output = colors.clone();
    sort_colors_naive(&mut output, ring_sizes);
    output
}

pub fn sort_colors(colors: &Vec<sRGB>, ring_sizes: &Vec<usize>, angles: &Vec<f64>) -> Vec<sRGB> {
    let n = colors.len();
    // start it kinda close to the destination
    let sorting_colors = sort_colors_simple(colors, ring_sizes);
    println!("a");
    let adj = adjacency_matrix(ring_sizes, angles);
    println!("b");
    let cpm = color_pair_matrix(&sorting_colors);
    println!("c");
    let mut permutation: Vec<usize> = (0..n).collect();
    let mut best = (compute_score(&permutation, &adj, &cpm), permutation.clone());

    let _pairs: Vec<_> = iproduct!((0..n), (0..n)).filter(|(a, b)| a < b).collect();
    let dist = distributions::Slice::new(_pairs.as_slice()).unwrap();
    println!("d");
    for it in 0..10000000 {
        let (i, j) = *dist.sample(&mut thread_rng());
        permutation.swap(i, j);
        let score = compute_score(&permutation, &adj, &cpm);
        if score < best.0 {
            println!("New best!\t{}", score);
            best = (score, permutation.clone());
        } else if it % 100 == 99 {
            permutation = best.1.clone();
        }
    }

    let mut output = sorting_colors.clone();
    for i in 0..n {
        output[i] = sorting_colors[best.1[i]];
    }
    return output;
}
