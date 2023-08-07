use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::seq::SliceRandom;
use rand::SeedableRng;
use readb::{new_index_table, DatabaseSettings, DefaultDatabase, IndexTable, IndexType};
use redb::{Database, Error, ReadableTable, TableDefinition};
use sled;
use std::cmp::max;
use std::fs::write;

fn benchmark_retrieval_from_db(c: &mut Criterion) {
    // Helper function to create data for our benchmark.
    fn generate_data(n: usize) -> Vec<(String, String)> {
        (0..n)
            .map(|i| (format!("key_{}", i), format!("value_{}", i)))
            .collect()
    }

    for &n in &[10, 100, 1000, 10_000, 100_000] {
        // Adjust this list for desired `n` values.
        let data_copy = generate_data(n);
        let mut group = c.benchmark_group(format!("Retrieve {} items", n));

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("index.bin");

        // Insertion phase (not measured)
        {
            let mut index_table: Box<dyn IndexTable> =
                new_index_table(path.clone(), IndexType::HashMap).unwrap();
            for (i, (key, _)) in data_copy.iter().enumerate() {
                index_table.insert(key.clone(), i).unwrap();
            }
            index_table.persist().unwrap();
        }

        {
            // Create the data file, by storing the value in each line (not measured)
            let content = data_copy
                .iter()
                .map(|(_, value)| value.clone())
                .collect::<Vec<_>>()
                .join("\n");
            write(path, content).unwrap();
        }

        // Benchmark for your database system
        group.bench_function("rdb_retrieve", |b| {
            // shuffle data with seed 42
            let mut data = data_copy.clone();
            let mut rng = rand::rngs::StdRng::seed_from_u64(42);
            data.shuffle(&mut rng);

            // Measure retrieval for your DB
            b.iter(|| {
                let mut db = DefaultDatabase::new(DatabaseSettings {
                    path: Some(dir.path().to_path_buf()),
                    cache_size: None,
                    index_type: IndexType::HashMap,
                })
                .unwrap();

                for (key, _) in data.iter() {
                    let _ = db.get(black_box(key)).unwrap();
                }
            });
        });

        group.bench_function("rdb_retrieve_10_percent", |b| {
            // shuffle data with seed 42
            let mut data = data_copy.clone();
            let mut rng = rand::rngs::StdRng::seed_from_u64(42);
            data.shuffle(&mut rng);
            // take 10% of the data
            let data = data.into_iter().take(n / 10).collect::<Vec<_>>();

            // Measure retrieval for your DB
            b.iter(|| {
                let mut db = DefaultDatabase::new(DatabaseSettings {
                    path: Some(dir.path().to_path_buf()),
                    cache_size: None,
                    index_type: IndexType::HashMap,
                })
                .unwrap();

                for (key, _) in data.iter() {
                    let _ = db.get(black_box(key)).unwrap();
                }
            });
        });

        group.bench_function("rdb_retrieve_20_percent_with_repetitions", |b| {
            // shuffle data with seed 42
            let mut data = Vec::new();
            let mut rng = rand::rngs::StdRng::seed_from_u64(42);

            let mut indexes = (0..n).collect::<Vec<_>>();
            indexes.shuffle(&mut rng);
            // consider only 3.5% of the indexes
            let length = max((n as f64 * 0.035) as usize, 1);
            let indexes = indexes.into_iter().take(length).collect::<Vec<_>>();

            let twenty_percent = (n as f64 * 0.2) as usize;
            for _ in 0..twenty_percent {
                // take a random index
                let index = indexes.choose(&mut rng).unwrap();
                data.push(data_copy[*index].clone());
            }

            // Measure retrieval for your DB
            b.iter(|| {
                let mut db = DefaultDatabase::new(DatabaseSettings {
                    path: Some(dir.path().to_path_buf()),
                    cache_size: None,
                    index_type: IndexType::HashMap,
                })
                .unwrap();

                for (key, _) in data.iter() {
                    let _ = db.get(black_box(key)).unwrap();
                }
            });
        });

        let _config = sled::Config::new().temporary(true);
        let db = sled::open(format!("{}/sled/my_db", dir.path().to_str().unwrap())).unwrap();

        // Insertion phase (not measured)
        for (key, value) in &data_copy {
            let value = value.as_bytes();
            db.insert(key, value).unwrap();
        }

        // Benchmark for sled
        group.bench_function("sled_retrieve", |b| {
            // shuffle data with seed 42
            let mut data = data_copy.clone();
            let mut rng = rand::rngs::StdRng::seed_from_u64(42);
            data.shuffle(&mut rng);

            // Measure retrieval for sled
            b.iter(|| {
                for (key, _) in &data {
                    let _ = db.get(black_box(key)).unwrap();
                }
            });
        });

        // Benchmark for sled
        group.bench_function("sled_retrieve_10_percent", |b| {
            // shuffle data with seed 42
            let mut data = data_copy.clone();
            let mut rng = rand::rngs::StdRng::seed_from_u64(42);
            data.shuffle(&mut rng);
            // take 10% of the data
            let data = data.into_iter().take(n / 10).collect::<Vec<_>>();

            // Measure retrieval for sled
            b.iter(|| {
                for (key, _) in &data {
                    let _ = db.get(black_box(key)).unwrap();
                }
            });
        });

        // Benchmark for sled
        group.bench_function("sled_retrieve_20_percent_with_repetitions", |b| {
            // shuffle data with seed 42
            let mut data = Vec::new();
            let mut rng = rand::rngs::StdRng::seed_from_u64(42);

            let mut indexes = (0..n).collect::<Vec<_>>();
            indexes.shuffle(&mut rng);
            // consider only 3.5% of the indexes
            let length = max((n as f64 * 0.035) as usize, 1);
            let indexes = indexes.into_iter().take(length).collect::<Vec<_>>();

            let twenty_percent = (n as f64 * 0.2) as usize;
            for _ in 0..twenty_percent {
                // take a random index
                let index = indexes.choose(&mut rng).unwrap();
                data.push(data_copy[*index].clone());
            }

            // Measure retrieval for sled
            b.iter(|| {
                for (key, _) in &data {
                    let _ = db.get(black_box(key)).unwrap();
                }
            });
        });

        let redb = Database::create(dir.path().join("./redb.rdb")).unwrap();
        const TABLE: TableDefinition<&str, &str> = TableDefinition::new("my_data");

        let write_txn = redb.begin_write().unwrap();
        {
            let mut table = write_txn.open_table(TABLE).unwrap();
            for (key, value) in &data_copy {
                table.insert(key.as_str(), value.as_str()).unwrap();
            }
        }
        write_txn.commit().unwrap();

        // Benchmark for redb
        group.bench_function("redb_retrieve", |b| {
            // shuffle data with seed 42
            let mut data = data_copy.clone();
            let mut rng = rand::rngs::StdRng::seed_from_u64(42);
            data.shuffle(&mut rng);

            // Measure retrieval for redb
            b.iter(|| {
                let read_txn = redb.begin_read().unwrap();
                let table = read_txn.open_table(TABLE).unwrap();
                for (key, _) in &data {
                    let _ = table.get(black_box(key.as_str())).unwrap();
                }
            });
        });

        // Benchmark for redb
        group.bench_function("redb_retrieve_10_percent", |b| {
            // shuffle data with seed 42
            let mut data = data_copy.clone();
            let mut rng = rand::rngs::StdRng::seed_from_u64(42);
            data.shuffle(&mut rng);
            // take 10% of the data
            let data = data.into_iter().take(n / 10).collect::<Vec<_>>();

            // Measure retrieval for redb
            b.iter(|| {
                let read_txn = redb.begin_read().unwrap();
                let table = read_txn.open_table(TABLE).unwrap();
                for (key, _) in &data {
                    let _ = table.get(black_box(key.as_str())).unwrap();
                }
            });
        });

        // Benchmark for redb
        group.bench_function("redb_retrieve_20_percent_with_repetitions", |b| {
            // shuffle data with seed 42
            let mut data = Vec::new();
            let mut rng = rand::rngs::StdRng::seed_from_u64(42);

            let mut indexes = (0..n).collect::<Vec<_>>();
            indexes.shuffle(&mut rng);
            // consider only 3.5% of the indexes
            let length = max((n as f64 * 0.035) as usize, 1);
            let indexes = indexes.into_iter().take(length).collect::<Vec<_>>();

            let twenty_percent = (n as f64 * 0.2) as usize;
            for _ in 0..twenty_percent {
                // take a random index
                let index = indexes.choose(&mut rng).unwrap();
                data.push(data_copy[*index].clone());
            }

            // Measure retrieval for redb
            b.iter(|| {
                let read_txn = redb.begin_read().unwrap();
                let table = read_txn.open_table(TABLE).unwrap();
                for (key, _) in &data {
                    let _ = table.get(black_box(key.as_str())).unwrap();
                }
            });
        });

        group.finish();
    }
}

criterion_group!(benches, benchmark_retrieval_from_db);
criterion_main!(benches);
