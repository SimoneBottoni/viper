use rand::prelude::IteratorRandom;
use rand::{thread_rng, Rng};
use rayon::prelude::*;

pub fn init_dataset(n_col: usize, n_row: usize) -> Vec<Vec<u64>> {
    let values: Vec<u64> = (0..n_row * n_col)
        .map(|_| thread_rng().gen_range(0..100))
        .collect();
    (0..n_row).map(|_| sample_values(&values, n_col)).collect()
}

fn sample_values(values: &[u64], n_col: usize) -> Vec<u64> {
    values
        .iter()
        .choose_multiple(&mut thread_rng(), n_col)
        .par_iter()
        .map(|el| **el)
        .collect()
}
