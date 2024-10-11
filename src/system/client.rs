use crate::primitives::commitment::Commitment;
use crate::primitives::mkhs::{KeyPair, Mkhs, Signature, PK};
use crate::util::dataset::Dataset;
use ark_bls12_381::Fr;
use num_bigint::BigInt;
use rayon::prelude::*;
use std::collections::HashMap;

pub struct Client {
    pub id: u64,
    pub key_pair: KeyPair,
    pub dataset: Dataset,
    pub secret: BigInt,
}

impl Client {
    pub fn new(id: u64, key_pair: KeyPair, dataset: Dataset, secret: BigInt) -> Self {
        Self {
            id,
            key_pair,
            dataset,
            secret,
        }
    }

    pub fn compute_commitments(&self) -> Vec<Commitment> {
        self.dataset
            .dataset
            .par_iter()
            .flatten()
            .map(|el| Commitment::commit(el, &self.secret))
            .collect()
    }

    pub fn verify_commitment(
        commitments: &[Commitment],
        aggregated_data: &[BigInt],
        random: &BigInt,
    ) -> anyhow::Result<()> {
        commitments
            .par_iter()
            .enumerate()
            .try_for_each(|(i, el)| el.open(&aggregated_data[i], random))
    }

    pub fn compute_signature(&self, mkhs: &Mkhs, messages: &[Vec<Fr>]) -> Vec<Signature> {
        messages
            .par_iter()
            .map(|row| mkhs.sign(&self.key_pair.sk, row))
            .collect()
    }

    pub fn verify_signature(
        mkhs: &Mkhs,
        pks: &HashMap<u64, PK>,
        aggregated_data: &[Vec<Fr>],
        aggregated_signatures: &[Signature],
    ) -> anyhow::Result<()> {
        aggregated_signatures
            .par_iter()
            .enumerate()
            .try_for_each(|(i, signature)| mkhs.verify(pks, &aggregated_data[i], signature))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
