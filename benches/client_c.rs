use criterion::measurement::WallTime;
use criterion::{criterion_group, criterion_main, BenchmarkGroup, BenchmarkId, Criterion};
use num_traits::ToPrimitive;
use rayon::prelude::*;

mod utils;

fn client_computation(
    rows: usize,
    cols: usize,
    decimals: u32,
    group: &mut BenchmarkGroup<WallTime>,
) {
    let n_client = 1;
    let utils::Setup { mkhs, clients, .. } = utils::Setup::build(n_client, rows, cols, decimals);

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

pub fn bench_client_c(c: &mut Criterion) {
    let mut group = c.benchmark_group("client_c");
    group.sample_size(10);

    let utils::Parameters {
        cols,
        decimals,
        rows,
        ..
    } = utils::Parameters::build();

    for decimals in decimals.clone() {
        for n_row in rows.clone() {
            client_computation(n_row, cols, decimals, &mut group)
        }
    }
    group.finish();
}

criterion_group!(benches, bench_client_c);
criterion_main!(benches);
