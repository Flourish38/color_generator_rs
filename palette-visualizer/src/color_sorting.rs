struct PairMatrix<T> {
    data: Vec<T>
}


impl<T> PairMatrix<T> {
    fn new(n: usize) -> Self {
        Self { data: Vec::with_capacity(PairMatrix::<T>::triangle_number(n)) }
    }

    fn new_populated<T2>(base: Vec<T2>, f: impl Fn(&T2, &T2) -> T) -> Self {
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
    fn triangle_number(n:usize) -> usize {
        (n*(n-1))/2
    }

    fn ordered_pair_to_index((min, max): (usize, usize)) -> usize {
        PairMatrix::<T>::triangle_number(max) + min
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