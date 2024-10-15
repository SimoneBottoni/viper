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
    pub const fn new(id: u64, key_pair: KeyPair, dataset: Dataset, secret: BigInt) -> Self {
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

    #[test]
    fn test_single_client() {
        let mkhs = Mkhs::setup(1, 2);
        let client = init_client(&mkhs, 1);

        let commitments = client.compute_commitments();
        let flatten_dataset = client
            .dataset
            .dataset
            .iter()
            .flatten()
            .map(|e| e.clone())
            .collect::<Vec<BigInt>>();
        let check = Client::verify_commitment(&commitments, &flatten_dataset, &client.secret);
        assert!(check.is_ok());

        let signature = client.compute_signature(&mkhs, &client.dataset.fr());
        let pk = HashMap::from([(client.id, client.key_pair.pk)]);
        let check = Client::verify_signature(&mkhs, &pk, &client.dataset.fr(), &signature);
        assert!(check.is_ok());
    }

    #[test]
    fn test_multi_client() {
        let mkhs = Mkhs::setup(2, 2);

        let c1 = init_client(&mkhs, 1);
        let c2 = init_client(&mkhs, 2);

        let commitments1 = c1.compute_commitments();
        let signature1 = c1.compute_signature(&mkhs, &c1.dataset.fr());

        let commitments2 = c2.compute_commitments();
        let signature2 = c2.compute_signature(&mkhs, &c2.dataset.fr());

        let agg_commitment: Vec<Commitment> = commitments1
            .par_iter()
            .zip(commitments2)
            .map(|(c1, c2)| c1 + &c2)
            .collect();
        let agg_signature: Vec<Signature> = signature1
            .par_iter()
            .zip(signature2)
            .map(|(s1, s2)| mkhs.eval(&[s1.clone(), s2]))
            .collect();

        let agg_dataset = Dataset::new(&[
            vec![BigInt::from(2), BigInt::from(4)],
            vec![BigInt::from(6), BigInt::from(8)],
        ]);
        let agg_dataset_flatten: Vec<BigInt> = agg_dataset
            .dataset
            .iter()
            .flatten()
            .map(|el| el.clone())
            .collect();

        let check = Client::verify_commitment(
            &agg_commitment,
            &agg_dataset_flatten,
            &(c1.secret + c2.secret),
        );
        assert!(check.is_ok());

        let pk = HashMap::from([(c1.id, c1.key_pair.pk), (c2.id, c2.key_pair.pk)]);
        let check = Client::verify_signature(&mkhs, &pk, &agg_dataset.fr(), &agg_signature);
        assert!(check.is_ok());
    }

    fn init_client(mkhs: &Mkhs, id: u64) -> Client {
        let client_id: u64 = id;
        let secret = BigInt::from(11);

        let dataset: Vec<Vec<BigInt>> = vec![
            vec![BigInt::from(1), BigInt::from(2)],
            vec![BigInt::from(3), BigInt::from(4)],
        ];
        let dataset = Dataset::new(&dataset);

        let key_pair = mkhs.generate_keys(client_id);

        Client::new(client_id, key_pair.clone(), dataset.clone(), secret.clone())
    }
}
