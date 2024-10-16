use criterion::measurement::WallTime;
use criterion::{criterion_group, criterion_main, BenchmarkGroup, BenchmarkId, Criterion};
use rayon::prelude::*;
use viper::primitives::commitment::Commitment;
use viper::primitives::mkhs::Signature;
use viper::system::aggregator::Aggregator;

mod utils;

fn aggregator(
    rows: usize,
    cols: usize,
    clients: usize,
    decimals: u32,
    group: &mut BenchmarkGroup<WallTime>,
) {
    // Setup
    let utils::Setup { mkhs, clients, .. } = utils::Setup::build(clients, rows, cols, decimals);

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

pub fn bench_aggregator(c: &mut Criterion) {
    let mut group = c.benchmark_group("aggregator");
    group.sample_size(10);

    let utils::Parameters {
        cols,
        decimals,
        rows,
        clients,
    } = utils::Parameters::build();

    for n_clients in clients {
        for n_row in rows.clone() {
            aggregator(n_row, cols, n_clients, decimals[0], &mut group)
        }
    }
    group.finish();
}

criterion_group!(benches, bench_aggregator);
criterion_main!(benches);
