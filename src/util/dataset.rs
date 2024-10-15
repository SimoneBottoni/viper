use ark_bls12_381::Fr;
use num_bigint::BigInt;
use num_traits::ToPrimitive;
use rand::prelude::IteratorRandom;
use rand::{thread_rng, Rng};
use rayon::prelude::*;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Dataset {
    pub dataset: Vec<Vec<BigInt>>,
}

impl Dataset {
    pub fn new(data: &[Vec<BigInt>]) -> Self {
        Self {
            dataset: data.to_vec(),
        }
    }

    pub fn build(n_col: usize, n_row: usize) -> Self {
        let values: Vec<u64> = (0..n_row * n_col)
            .map(|_| thread_rng().gen_range(0..100))
            .collect();
        let res: Vec<Vec<BigInt>> = (0..n_row).map(|_| sample_values(&values, n_col)).collect();
        Self::new(&res)
    }

    pub fn fr(&self) -> Vec<Vec<Fr>> {
        self.dataset
            .par_iter()
            .map(|row| {
                row.par_iter()
                    .map(|el| Fr::from(el.to_u64().unwrap()))
                    .collect()
            })
            .collect()
    }
}

fn sample_values(values: &[u64], n_col: usize) -> Vec<BigInt> {
    values
        .iter()
        .choose_multiple(&mut thread_rng(), n_col)
        .par_iter()
        .map(|el| BigInt::from(**el))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_bls12_381::Fr;
    use ark_ff::{BigInteger, PrimeField};
    use num_traits::ToPrimitive;

    #[test]
    fn test_sample_values() {
        let a = BigInt::from(100);
        let b = BigInt::from(200);

        let a_fr = Fr::from(a.to_u64().unwrap());
        let b_fr = Fr::from(b.to_u64().unwrap());

        let res = a_fr + b_fr;
        let temp = res.into_bigint().to_bytes_le();
        let temp = BigInt::from_signed_bytes_le(temp.as_slice());
        println!("{}", temp);
    }
}
