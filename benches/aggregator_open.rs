use criterion::measurement::WallTime;
use criterion::{criterion_group, criterion_main, BenchmarkGroup, BenchmarkId, Criterion};
use num_traits::ToPrimitive;
use rayon::prelude::*;
use viper::primitives::commitment::Commitment;
use viper::system::aggregator::Aggregator;

mod utils;

fn aggregator_open(rows: usize, cols: usize, decimals: u32, group: &mut BenchmarkGroup<WallTime>) {
    let n_client = 1;
    let utils::Setup {
        aggregated_secret,
        clients,
        ..
    } = utils::Setup::build(n_client, rows, cols, decimals);

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

pub fn bench_aggregator_open(c: &mut Criterion) {
    let mut group = c.benchmark_group("aggregator_open");
    group.sample_size(10);

    let utils::Parameters {
        cols,
        decimals,
        rows,
        ..
    } = utils::Parameters::build();

    for decimals in decimals.clone() {
        for n_row in rows.clone() {
            aggregator_open(n_row, cols, decimals, &mut group)
        }
    }
    group.finish();
}

criterion_group!(benches, bench_aggregator_open);
criterion_main!(benches);
