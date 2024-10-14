use ark_bls12_381::Fr;
use num_bigint::BigInt;
use num_traits::ToPrimitive;
use rand::Rng;
use rayon::prelude::*;
use std::collections::HashMap;
use viper::primitives::commitment::Commitment;
use viper::primitives::mkhs::{Mkhs, Signature, PK};
use viper::system::aggregator::Aggregator;
use viper::system::client::Client;
use viper::util::dataset::Dataset;

fn main() {
    let n_col = 3;
    let n_row = 10;

    let n_client = 2;
    let mkhs = Mkhs::setup(n_client, n_col);

    let secrets: Vec<BigInt> = (0..n_client)
        .map(|_| BigInt::from(rand::thread_rng().gen_range(1..=100)))
        .collect();

    // Training phase
    println!("TRAINING...");
    let clients: Vec<Client> = (1..=n_client)
        .into_par_iter()
        .map(|id| {
            let dataset = Dataset::build(n_col, n_row);
            let key_pair = mkhs.generate_keys(id as u64);
            Client::new(id as u64, key_pair, dataset, secrets[id - 1].clone())
        })
        .collect();

    let pks: HashMap<u64, PK> = HashMap::from_par_iter(
        clients
            .par_iter()
            .map(|client| (client.id, client.key_pair.pk.clone())),
    );

    // Commitments and Signatures
    println!("COMPUTING COMMITMENTS...");
    let commitments: Vec<Vec<Commitment>> = clients
        .par_iter()
        .map(|client| client.compute_commitments())
        .collect();
    println!("COMPUTING SIGNATURES...");
    let signatures: Vec<Vec<Signature>> = clients
        .par_iter()
        .map(|client| {
            let fr_dataset = client.dataset.fr();
            client.compute_signature(&mkhs, &fr_dataset)
        })
        .collect();

    // Aggregator
    println!("AGGREGATING COMMITMENTS...");
    let aggregated_commitments = Aggregator::aggregate_commitments(&commitments);
    println!("AGGREGATING SIGNATURES...");
    let aggregated_signatures = Aggregator::aggregate_signatures(&mkhs, &signatures);

    // Clients secret
    let aggregated_secret = secrets.iter().sum();

    // Aggregator Open
    println!("OPENING COMMITMENTS...");
    let aggregated_dataset =
        Aggregator::open_commitments(&aggregated_commitments, &aggregated_secret).unwrap();

    // Clients' verification
    println!("VERIFYING COMMITMENTS...");
    let commitment_check = Client::verify_commitment(
        &aggregated_commitments,
        &aggregated_dataset,
        &aggregated_secret,
    );

    println!("Commitment check: {:?}", commitment_check);

    let aggregated_data: Vec<Fr> = aggregated_dataset
        .par_iter()
        .map(|el| Fr::from(el.to_u64().unwrap()))
        .collect();
    let aggregated_data: Vec<Vec<Fr>> = aggregated_data.chunks(n_col).map(|s| s.into()).collect();

    println!("VERIFYING SIGNATURES...");
    let signature_check =
        Client::verify_signature(&mkhs, &pks, &aggregated_data, &aggregated_signatures);

    println!("Signature check: {:?}", signature_check);
}
