use ark_bls12_381::Fr;
use criterion::measurement::WallTime;
use criterion::{criterion_group, criterion_main, BenchmarkGroup, BenchmarkId, Criterion};
use num_bigint::BigInt;
use num_traits::ToPrimitive;
use rayon::prelude::*;
use viper::primitives::commitment::Commitment;
use viper::primitives::mkhs::Signature;
use viper::system::client::Client;

mod utils;

fn client_verification(
    rows: usize,
    cols: usize,
    decimals: u32,
    group: &mut BenchmarkGroup<WallTime>,
) {
    let n_client = 1;
    let utils::Setup {
        mkhs,
        aggregated_secret,
        clients,
        pks,
    } = utils::Setup::build(n_client, rows, cols, decimals);

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

pub fn bench_client_v(c: &mut Criterion) {
    let mut group = c.benchmark_group("client_v");
    group.sample_size(10);

    let utils::Parameters {
        cols,
        decimals,
        rows,
        ..
    } = utils::Parameters::build();

    for decimals in decimals.clone() {
        for n_row in rows.clone() {
            client_verification(n_row, cols, decimals, &mut group)
        }
    }
    group.finish();
}

criterion_group!(benches, bench_client_v);
criterion_main!(benches);
