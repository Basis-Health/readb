#[cfg(feature = "index-write")]
mod tests {
    use rand::{Rng, SeedableRng};
    use readb::{new_index_table, DatabaseSettings, DefaultDatabase, IndexTable, IndexType};
    use std::io::Write;

    #[test]
    fn create_index_table_and_then_retrieve_from_db() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("./.rdb.data");
        let index_path = dir.path().join("./.rdb.index");

        {
            let index_type_path = index_path.with_extension("type");
            // write HashMap to type file
            let mut file = std::fs::File::create(index_type_path).unwrap();
            file.write_all(b"HashMap\n").unwrap();
        }

        let random_strings_with_keys = vec![
            ("hi", "hello"),
            ("there", "there"),
            ("how", "are"),
            ("you", "doing"),
            ("today", "today"),
            ("?", "?"),
        ];

        {
            let mut index_table: Box<dyn IndexTable> =
                new_index_table(index_path, IndexType::HashMap).unwrap();

            for (i, (key, _)) in random_strings_with_keys.iter().enumerate() {
                index_table.insert(key.to_string(), i).unwrap();
            }
            index_table.persist().unwrap();
        }

        {
            // Create the data file, by storing the value in each line
            let mut file = std::fs::File::create(path).unwrap();
            for (_, value) in random_strings_with_keys.iter() {
                file.write_all(value.as_bytes()).unwrap();
                file.write_all(b"\n").unwrap();
            }
        }

        // Now let's create the database
        let mut db = DefaultDatabase::new(DatabaseSettings {
            path: Some(dir.path().to_path_buf()),
            cache_size: None,
            index_type: IndexType::HashMap,
        })
            .unwrap();

        // And let's retrieve some data
        for (key, expected_value) in random_strings_with_keys.iter() {
            let value = db.get(key).unwrap();
            assert!(value.is_some());
            assert_eq!(value.unwrap(), *expected_value);
        }

        let non_existent_value = db.get("non-existent").unwrap();
        assert!(non_existent_value.is_none());

        // Create link
        db.link("you", "new-link").unwrap();
        let value = db.get("new-link").unwrap();
        assert!(value.is_some());
        assert_eq!(value.unwrap(), "doing");
    }

    fn n_threads_accessing_at_the_same_time_test(n: usize, thread_count: usize) {
        // seed is 16807
        let mut rng = rand::rngs::StdRng::seed_from_u64(16807);

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("./.rdb.data");
        let index_path = dir.path().join("./.rdb.index");

        {
            let index_type_path = index_path.with_extension("type");
            // write HashMap to type file
            let mut file = std::fs::File::create(index_type_path).unwrap();
            file.write_all(b"HashMap\n").unwrap();
        }

        let random_key_values = (0..n)
            .map(|i| (format!("key{}", i), format!("value{}", i)))
            .collect::<Vec<_>>();

        {
            let mut index_table: Box<dyn IndexTable> =
                new_index_table(index_path.clone(), IndexType::HashMap).unwrap();

            for (i, (key, _)) in random_key_values.iter().enumerate() {
                index_table.insert(key.to_string(), i).unwrap();
            }
            index_table.persist().unwrap();
        }

        {
            // Create the data file, by storing the value in each line
            let mut file = std::fs::File::create(path.clone()).unwrap();
            for (_, value) in random_key_values.iter() {
                file.write_all(value.as_bytes()).unwrap();
                file.write_all(b"\n").unwrap();
            }
        }

        // all threads should access approximately 10% of the keys
        let mut keys_to_access_by_thread: Vec<Vec<String>> = Vec::new();
        for _ in 0..thread_count {
            let mut keys_to_access = Vec::new();
            for _ in 0..n / 10 {
                let index = rng.gen_range(0..n);
                keys_to_access.push(random_key_values[index].0.clone());
            }
            keys_to_access_by_thread.push(keys_to_access);
        }

        // Create the 16 threads
        // each thread should take its keys from the keys_to_access_by_thread vector
        // and then access the database
        // there should be a barrier so that each thread constructed the database before accessing it
        let mut threads = Vec::new();
        let barrier = std::sync::Arc::new(std::sync::Barrier::new(thread_count));

        for keys_to_access in keys_to_access_by_thread {
            let dir = dir.path().to_path_buf();
            let c = barrier.clone();
            threads.push(std::thread::spawn(move || {
                let mut db = DefaultDatabase::new(DatabaseSettings {
                    path: Some(dir),
                    cache_size: None,
                    index_type: IndexType::HashMap,
                })
                    .unwrap();

                // barrier
                c.wait();

                for key in keys_to_access {
                    let value = db.get(&key).unwrap();
                    assert!(value.is_some());
                    assert_eq!(
                        value.unwrap(),
                        format!("value{}", key[3..].parse::<usize>().unwrap())
                    );
                }
            }));
        }

        for thread in threads {
            thread.join().unwrap();
        }
    }

    #[test]
    fn two_threads_accessing_at_the_same_time_test() {
        let n = 1000;
        let threads = 2;

        n_threads_accessing_at_the_same_time_test(n, threads);
    }

    #[test]
    fn sixteen_threads_accessing_at_the_same_time_test() {
        let n = 10000;
        let threads = 16;

        n_threads_accessing_at_the_same_time_test(n, threads);
    }
}