use crate::primitives::commitment::Commitment;
use crate::primitives::ec::Point;
use crate::primitives::mkhs::{Mkhs, Signature};
use crate::primitives::phollard_rho::pollards_rho;
use num_bigint::BigInt;
use rayon::prelude::*;
use std::ops::Neg;

pub struct Aggregator;

impl Aggregator {
    pub fn aggregate_commitments(commitments: &[Vec<Commitment>]) -> Vec<Commitment> {
        let commitments_t: Vec<Vec<Commitment>> = transpose_dataset(commitments);
        commitments_t
            .par_iter()
            .map(|col| col.par_iter().map(|el| el.clone()).sum::<Commitment>())
            .collect()
    }

    pub fn aggregate_signatures(mkhs: &Mkhs, signatures: &[Vec<Signature>]) -> Vec<Signature> {
        let signatures_t: Vec<Vec<Signature>> = transpose_dataset(signatures);
        signatures_t.par_iter().map(|col| mkhs.eval(col)).collect()
    }

    pub fn open_commitments(
        commitments: &[Commitment],
        secret: &BigInt,
    ) -> anyhow::Result<Vec<BigInt>> {
        let commitments: Vec<Commitment> = commitments
            .par_iter()
            .map(|el| Commitment::new(&el.c + &(Point::default() * secret).neg()))
            .collect();
        commitments
            .par_iter()
            .map(|el| pollards_rho(&Point::default(), &el.c))
            .collect()
    }
}

fn transpose_dataset<T>(dataset: &[Vec<T>]) -> Vec<Vec<T>>
where
    T: Send + Sync + Clone,
{
    (0..dataset[0].len())
        .map(|col| dataset.par_iter().map(|row| row[col].clone()).collect())
        .collect()
}
