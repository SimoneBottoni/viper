use crate::primitives::commitment::Commitment;
use crate::primitives::mkhs::{KeyPair, Label, Mkhs, Signature, PK};
use ark_bls12_381::Fr;
use num_bigint::BigInt;
use rayon::prelude::*;
use std::collections::HashMap;

pub struct Client {
    pub id: u64,
}

impl Client {
    pub fn compute_commitments(data: &[Vec<BigInt>], random: &BigInt) -> Vec<Commitment> {
        data.par_iter()
            .flatten()
            .map(|el| Commitment::commit(el, random))
            .collect()
    }

    pub fn verify_commitment(
        commitments: &[Vec<Commitment>],
        aggregated_data: &[Vec<BigInt>],
        random: &BigInt,
    ) -> anyhow::Result<()> {
        commitments.par_iter().enumerate().try_for_each(|(i, row)| {
            row.par_iter()
                .enumerate()
                .try_for_each(|(j, el)| el.open(&aggregated_data[i][j], random))
        })
    }

    pub fn compute_signature(
        mkhs: &Mkhs,
        key_pair: &KeyPair,
        messages: &[Vec<Fr>],
    ) -> Vec<Signature> {
        messages
            .par_iter()
            .map(|row| mkhs.sign(&key_pair.sk, row))
            .collect()
    }

    pub fn verify_signature(
        mkhs: &Mkhs,
        pks: &HashMap<u64, PK>,
        labels: &[Label],
        aggregated_data: &[Vec<Fr>],
        aggregated_signatures: &[Signature],
    ) -> anyhow::Result<()> {
        aggregated_signatures
            .par_iter()
            .enumerate()
            .try_for_each(|(i, signature)| mkhs.verify(labels, pks, &aggregated_data[i], signature))
    }
}
