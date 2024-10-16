use ark_bls12_381::Fr;
use criterion::measurement::WallTime;
use criterion::{criterion_group, BenchmarkGroup, BenchmarkId, Criterion};
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

struct Parameters {
    cols: usize,
    decimals: Vec<u32>,
    rows: Vec<usize>,
    clients: Vec<usize>,
}

impl Parameters {
    fn build() -> Self {
        let cols = 10;

        let decimals = vec![4, 6, 8];
        let rows = vec![
            100, 500, 750, 1000, 5000, 7500, 10000, 25000, 50000, 75000, 100000,
        ];
        let clients = vec![2, 5, 10];

        Self {
            cols,
            decimals,
            rows,
            clients,
        }
    }
}

struct Setup {
    mkhs: Mkhs,
    aggregated_secret: BigInt,
    clients: Vec<Client>,
    pks: HashMap<u64, PK>,
}

impl Setup {
    fn build(n_client: usize, n_row: usize, n_col: usize, decimals: u32) -> Self {
        let mkhs = Mkhs::setup(n_client, n_col);
        let secrets: Vec<BigInt> = (0..n_client)
            .map(|_| BigInt::from(rand::thread_rng().gen_range(1..=100)))
            .collect();
        let aggregated_secret: BigInt = secrets.iter().sum();

        let clients: Vec<Client> = (1..=n_client)
            .into_par_iter()
            .map(|id| {
                let dataset = Dataset::build(n_col, n_row, decimals);
                let key_pair = mkhs.generate_keys(id as u64);
                Client::new(id as u64, key_pair, dataset, secrets[id - 1].clone())
            })
            .collect();

        let pks: HashMap<u64, PK> = HashMap::from_par_iter(
            clients
                .par_iter()
                .map(|client| (client.id, client.key_pair.pk.clone())),
        );

        Self {
            mkhs,
            aggregated_secret,
            clients,
            pks,
        }
    }
}

fn bench_client_computation(
    rows: usize,
    cols: usize,
    decimals: u32,
    group: &mut BenchmarkGroup<WallTime>,
) {
    let n_client = 1;
    let Setup {
        mkhs,
        clients,
        ..
    } = Setup::build(n_client, rows, cols, decimals);

    // Computing commitments
    group.bench_function(
        BenchmarkId::new(
            "computing_commitments",
            format!("decimals: {} - rows: {}", decimals, rows),
        ),
        |b| {
            b.iter(|| {
                clients[0].compute_commitments();
            })
        },
    );

    // Computing signatures
    let bench_dataset = clients[0].dataset.fr();
    group.bench_function(
        BenchmarkId::new(
            "computing_signatures",
            format!("decimals: {} - rows: {}", decimals, rows),
        ),
        |b| {
            b.iter(|| {
                clients[0].compute_signature(&mkhs, &bench_dataset);
            })
        },
    );
}

fn bench_client_verification(
    rows: usize,
    cols: usize,
    decimals: u32,
    group: &mut BenchmarkGroup<WallTime>,
) {
    let n_client = 1;
    let Setup {
        mkhs,
        aggregated_secret,
        clients,
        pks,
    } = Setup::build(n_client, rows, cols, decimals);

    // Computing commitments
    let commitments: Vec<Vec<Commitment>> = clients
        .par_iter()
        .map(|client| client.compute_commitments())
        .collect();

    // Computing signatures
    let signatures: Vec<Vec<Signature>> = clients
        .par_iter()
        .map(|client| {
            let fr_dataset = client.dataset.fr();
            client.compute_signature(&mkhs, &fr_dataset)
        })
        .collect();

    // Aggregation
    let aggregated_dataset = clients[0]
        .dataset
        .dataset
        .iter()
        .flatten()
        .map(|e| e.clone())
        .collect::<Vec<BigInt>>();

    let aggregated_commitments = commitments[0].clone();
    let aggregated_signatures = signatures[0].clone();

    // Verifying commitments
    group.bench_function(
        BenchmarkId::new(
            "verifying_commitments",
            format!("decimals: {} - rows: {}", decimals, rows),
        ),
        |b| {
            b.iter(|| {
                let _ = Client::verify_commitment(
                    &aggregated_commitments,
                    &aggregated_dataset,
                    &aggregated_secret,
                );
            })
        },
    );

    // Verifying signatures
    let aggregated_data: Vec<Fr> = aggregated_dataset
        .par_iter()
        .map(|el| Fr::from(el.to_u64().unwrap()))
        .collect();
    let aggregated_data: Vec<Vec<Fr>> = aggregated_data.chunks(cols).map(|s| s.into()).collect();
    group.bench_function(
        BenchmarkId::new(
            "verifying_signatures",
            format!("decimals: {} - rows: {}", decimals, rows),
        ),
        |b| {
            b.iter(|| {
                let _ =
                    Client::verify_signature(&mkhs, &pks, &aggregated_data, &aggregated_signatures);
            })
        },
    );
}

fn bench_aggregator(
    rows: usize,
    cols: usize,
    clients: usize,
    decimals: u32,
    group: &mut BenchmarkGroup<WallTime>,
) {
    // Setup
    let Setup { mkhs, clients, .. } = Setup::build(clients, rows, cols, decimals);

    let commitments: Vec<Vec<Commitment>> = clients
        .par_iter()
        .map(|client| client.compute_commitments())
        .collect();

    // Computing signatures
    let signatures: Vec<Vec<Signature>> = clients
        .par_iter()
        .map(|client| {
            let fr_dataset = client.dataset.fr();
            client.compute_signature(&mkhs, &fr_dataset)
        })
        .collect();

    // Aggregating commitments
    group.bench_function(
        BenchmarkId::new(
            "aggregating_commitments",
            format!("clients: {} - rows: {}", clients.len(), rows),
        ),
        |b| {
            b.iter(|| {
                Aggregator::aggregate_commitments(&commitments);
            })
        },
    );

    // Aggregating signatures
    group.bench_function(
        BenchmarkId::new(
            "aggregating_signatures",
            format!("clients: {} - rows: {}", clients.len(), rows),
        ),
        |b| {
            b.iter(|| {
                Aggregator::aggregate_signatures(&mkhs, &signatures);
            })
        },
    );
}

fn bench_aggregator_open(
    rows: usize,
    cols: usize,
    decimals: u32,
    group: &mut BenchmarkGroup<WallTime>,
) {
    let n_client = 1;
    let Setup {
        aggregated_secret,
        clients,
        ..
    } = Setup::build(n_client, rows, cols, decimals);

    // Computing commitments
    let commitments: Vec<Vec<Commitment>> = clients
        .par_iter()
        .map(|client| client.compute_commitments())
        .collect();

    let aggregated_commitments = commitments[0].clone();

    // Opening commitments
    group.bench_function(
        BenchmarkId::new(
            "opening_commitments",
            format!("decimals: {} - rows: {}", decimals, rows),
        ),
        |b| {
            b.iter(|| {
                let _ = Aggregator::open_commitments(&aggregated_commitments, &aggregated_secret);
            })
        },
    );
}

pub fn criterion_benchmark_client_computation(c: &mut Criterion) {
    let mut group = c.benchmark_group("client_c");

    let Parameters {
        cols,
        decimals,
        rows,
        ..
    } = Parameters::build();

    for decimals in decimals.clone() {
        for n_row in rows.clone() {
            bench_client_computation(n_row, cols, decimals, &mut group)
        }
    }
    group.finish();
}

pub fn criterion_benchmark_client_verification(c: &mut Criterion) {
    let mut group = c.benchmark_group("client_v");

    let Parameters {
        cols,
        decimals,
        rows,
        ..
    } = Parameters::build();

    for decimals in decimals.clone() {
        for n_row in rows.clone() {
            bench_client_computation(n_row, cols, decimals, &mut group)
        }
    }
    group.finish();
}

pub fn criterion_benchmark_aggregator(c: &mut Criterion) {
    let mut group = c.benchmark_group("aggregator");
    let Parameters {
        cols,
        decimals,
        rows,
        clients,
    } = Parameters::build();

    for n_clients in clients {
        for n_row in rows.clone() {
            bench_aggregator(n_row, cols, n_clients, decimals[0], &mut group)
        }
    }
    group.finish();
}

pub fn criterion_benchmark_aggregator_open(c: &mut Criterion) {
    let mut group = c.benchmark_group("aggregator_open");
    let Parameters {
        cols,
        decimals,
        rows,
        ..
    } = Parameters::build();

    for decimals in decimals.clone() {
        for n_row in rows.clone() {
            bench_aggregator_open(n_row, cols, decimals, &mut group)
        }
    }
    group.finish();
}

criterion_group! {
    name = client_c;
    config = Criterion::default().sample_size(10);
    targets = criterion_benchmark_client_computation
}

criterion_group! {
    name = client_v;
    config = Criterion::default().sample_size(10);
    targets = criterion_benchmark_client_verification
}

criterion_group! {
    name = aggregator;
    config = Criterion::default().sample_size(10);
    targets = criterion_benchmark_aggregator
}

criterion_group! {
    name = aggregator_open;
    config = Criterion::default().sample_size(10);
    targets = criterion_benchmark_aggregator_open
}