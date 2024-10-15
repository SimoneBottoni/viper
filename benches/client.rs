use ark_bls12_381::Fr;
use criterion::{criterion_group, criterion_main, Criterion};
use num_bigint::BigInt;
use num_traits::ToPrimitive;
use rand::Rng;
use rayon::iter::*;
use rayon::prelude::FromParallelIterator;
use std::collections::HashMap;
use viper::primitives::commitment::Commitment;
use viper::primitives::mkhs::{Mkhs, Signature, PK};
use viper::system::aggregator::Aggregator;
use viper::system::client::Client;
use viper::util::dataset::Dataset;

fn bench(n_row: usize, n_col: usize, n_client: usize, c: &mut Criterion) {
    // Setup
    let mkhs = Mkhs::setup(n_client, n_col);
    let secrets: Vec<BigInt> = (0..n_client)
        .map(|_| BigInt::from(rand::thread_rng().gen_range(1..=100)))
        .collect();
    let aggregated_secret: BigInt = secrets.iter().sum();

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

    // Computing commitments
    c.bench_function("Computing commitments.", |b| {
        b.iter(|| {
            clients[0].compute_commitments();
        })
    });
    let commitments: Vec<Vec<Commitment>> = clients
        .par_iter()
        .map(|client| client.compute_commitments())
        .collect();

    // Computing signatures
    let bench_dataset = clients[0].dataset.fr();
    c.bench_function("Computing signatures.", |b| {
        b.iter(|| {
            clients[0].compute_signature(&mkhs, &bench_dataset);
        })
    });
    let signatures: Vec<Vec<Signature>> = clients
        .par_iter()
        .map(|client| {
            let fr_dataset = client.dataset.fr();
            client.compute_signature(&mkhs, &fr_dataset)
        })
        .collect();

    // Aggregating commitments
    c.bench_function("Aggregating commitments.", |b| {
        b.iter(|| {
            Aggregator::aggregate_commitments(&commitments);
        })
    });
    let aggregated_commitments = Aggregator::aggregate_commitments(&commitments);

    // Aggregating signatures
    c.bench_function("Aggregating signatures.", |b| {
        b.iter(|| {
            Aggregator::aggregate_signatures(&mkhs, &signatures);
        })
    });
    let aggregated_signatures = Aggregator::aggregate_signatures(&mkhs, &signatures);

    // Opening commitments
    c.bench_function("Opening commitments.", |b| {
        b.iter(|| {
            let _ = Aggregator::open_commitments(&aggregated_commitments, &aggregated_secret);
        })
    });
    let aggregated_dataset =
        Aggregator::open_commitments(&aggregated_commitments, &aggregated_secret).unwrap();

    // Verifying commitments
    c.bench_function("Verifying commitments.", |b| {
        b.iter(|| {
            let _ = Client::verify_commitment(
                &aggregated_commitments,
                &aggregated_dataset,
                &aggregated_secret,
            );
        })
    });
    let commitment_check = Client::verify_commitment(
        &aggregated_commitments,
        &aggregated_dataset,
        &aggregated_secret,
    );
    assert!(commitment_check.is_ok());

    // Verifying signatures
    let aggregated_data: Vec<Fr> = aggregated_dataset
        .par_iter()
        .map(|el| Fr::from(el.to_u64().unwrap()))
        .collect();
    let aggregated_data: Vec<Vec<Fr>> = aggregated_data.chunks(n_col).map(|s| s.into()).collect();
    c.bench_function("Verifying signatures.", |b| {
        b.iter(|| {
            let _ = Client::verify_signature(&mkhs, &pks, &aggregated_data, &aggregated_signatures);
        })
    });
    let signature_check =
        Client::verify_signature(&mkhs, &pks, &aggregated_data, &aggregated_signatures);
    assert!(signature_check.is_ok());
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let n_row = 3;
    let n_col = 2;
    let n_client = 1;

    bench(n_row, n_col, n_client, c)
}

criterion_group! {
    name = benches;
    // This can be any expression that returns a `Criterion` object.
    config = Criterion::default().sample_size(10);
    targets = criterion_benchmark
}
criterion_main!(benches);
