use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::distributions::Alphanumeric;
use rand::rngs::ThreadRng;
use rand::Rng;
use readb::Database;
use std::cmp::min;
use std::fs;

enum Operation {
    Read(String),
    Write((String, String)),
}

fn benchmark_write(c: &mut Criterion) {
    let n = 100_000;
    let mut rand = rand::thread_rng();

    fn random_32_byte_string(rand: &mut ThreadRng) -> String {
        rand.sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect()
    }

    let data: Vec<(String, String)> = (0..n)
        .map(|_| {
            (
                random_32_byte_string(&mut rand),
                random_32_byte_string(&mut rand),
            )
        })
        .collect();

    // now fill that data into both databases withouth measuring
    let tempdir = tempfile::tempdir().unwrap();
    let readb = tempdir.path().join("readb");
    fs::create_dir(&readb).unwrap();

    let readb_tx = tempdir.path().join("readb_tx");
    fs::create_dir(&readb_tx).unwrap();

    let sledb = tempdir.path().join("sledb");
    fs::create_dir(&sledb).unwrap();

    let sled_instance = sled::Config::new()
        .path(&sledb)
        .temporary(true)
        .open()
        .unwrap();

    let mut readb_instance = readb::DefaultDatabase::new(readb::DatabaseSettings {
        path: Some(readb),
        cache_size: None,
        index_type: readb::IndexType::HashMap,
        ..Default::default()
    });

    let mut readb_tx_instance = readb::DefaultDatabase::new(readb::DatabaseSettings {
        path: Some(readb_tx),
        ..Default::default()
    });

    for (key, value) in data.iter() {
        readb_instance.put(key.as_str(), value.as_bytes()).unwrap();
        sled_instance
            .insert(key.as_bytes(), value.as_bytes())
            .unwrap();
    }

    readb_instance.persist().unwrap();

    let ops = vec![1000, 10_000, 100_000, 1_000_000];
    for op in ops {
        println!("Initializing benchmark for {} operations", op);

        let tasks = (0..op)
            .map(|i| {
                let idx = rand.gen_range(0..data.len());
                let key = data[idx].0.clone();
                if rand.gen_bool(0.99) {
                    // with a 1% chance use a key that is not in the database
                    if rand.gen_bool(0.01) {
                        let key = random_32_byte_string(&mut rand);
                        Operation::Read(key)

                        // With a 3% chance use a key that was already used
                    } else if i > 1 && rand.gen_bool(0.03) {
                        let key = data[rand.gen_range(0..(min(i, n)))].0.clone();
                        Operation::Read(key)
                    } else {
                        Operation::Read(key)
                    }
                } else {
                    let value = random_32_byte_string(&mut rand);
                    Operation::Write((key, value))
                }
            })
            .collect::<Vec<_>>();

        println!("Running benchmark for {} operations", op);

        c.bench_function(format!("readb_write_{}", op).as_str(), |b| {
            b.iter(|| {
                for task in tasks.iter() {
                    match task {
                        Operation::Read(key) => {
                            let _ = readb_instance.get(black_box(key)).unwrap();
                        }
                        Operation::Write((key, value)) => {
                            let _ = readb_instance
                                .put(black_box(key), black_box(value.as_bytes()))
                                .unwrap();
                        }
                    }
                }
            })
        });

        c.bench_function(format!("readb_write_tx_{}", op).as_str(), |b| {
            b.iter(|| {
                let mut tx = readb_tx_instance.tx().unwrap();
                for task in tasks.iter() {
                    match task {
                        Operation::Read(key) => {
                            tx.commit().unwrap();

                            tx = readb_tx_instance.tx().unwrap();
                            let _ = readb_instance.get(black_box(key)).unwrap();
                        }
                        Operation::Write((key, value)) => {
                            tx.put(black_box(key), black_box(value.as_bytes())).unwrap();
                        }
                    }
                }
            })
        });

        c.bench_function(format!("sled_write_{}", op).as_str(), |b| {
            b.iter(|| {
                for task in tasks.iter() {
                    match task {
                        Operation::Read(key) => {
                            let _ = sled_instance.get(black_box(key.as_bytes())).unwrap();
                        }
                        Operation::Write((key, value)) => {
                            let _ = sled_instance
                                .insert(black_box(key.as_bytes()), black_box(value.as_bytes()))
                                .unwrap();
                        }
                    }
                }
            })
        });

        println!("Finished benchmark for {} operations", op);
    }
}

criterion_group!(benches, benchmark_write);
criterion_main!(benches);
