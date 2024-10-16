use num_bigint::BigInt;
use rand::Rng;
use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator};
use rayon::prelude::*;
use std::collections::HashMap;
use viper::primitives::mkhs::{Mkhs, PK};
use viper::system::client::Client;
use viper::util::dataset::Dataset;

pub struct Parameters {
    pub cols: usize,
    pub decimals: Vec<u32>,
    pub rows: Vec<usize>,
    pub clients: Vec<usize>,
}

impl Parameters {
    pub fn build() -> Self {
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

pub struct Setup {
    pub mkhs: Mkhs,
    pub aggregated_secret: BigInt,
    pub clients: Vec<Client>,
    pub pks: HashMap<u64, PK>,
}

impl Setup {
    pub fn build(n_client: usize, n_row: usize, n_col: usize, decimals: u32) -> Self {
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
