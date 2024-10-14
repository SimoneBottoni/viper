use criterion::{black_box, criterion_group, criterion_main, Criterion};
use num_bigint::BigInt;
use rand::Rng;
use rayon::iter::*;
use rayon::prelude::FromParallelIterator;
use std::collections::HashMap;
use viper::primitives::mkhs::{Mkhs, PK};
use viper::system::client::Client;
use viper::util::dataset::Dataset;

fn bench(n_row: usize, n_col: usize, n_client: usize, c: &mut Criterion) {
    let mkhs = Mkhs::setup(n_client, n_col);
    let secrets: Vec<BigInt> = (0..n_client)
        .map(|_| BigInt::from(rand::thread_rng().gen_range(1..=100)))
        .collect();

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

    c.bench_function("Computing Commitments.", |b| {
        b.iter(|| {
            clients[0].compute_commitments();
        })
    });

    let bench_dataset = clients[0].dataset.fr();
    c.bench_function("Computing Signature.", |b| {
        b.iter(|| {
            clients[0].compute_signature(&mkhs, &bench_dataset);
        })
    });
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let n_row = 10000;
    let n_col = 10;
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
