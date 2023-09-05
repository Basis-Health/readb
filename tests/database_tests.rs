#[cfg(feature = "write")]
mod tests {
    use rand::Rng;

    use readb::{Database, DatabaseSettings, DefaultDatabase, IndexType};
    #[cfg(feature = "garbage-collection")]
    use walkdir::WalkDir;

    #[test]
    fn test_simple_read_after_creation() {
        let dir = tempfile::tempdir().unwrap();
        {
            let mut db = DefaultDatabase::new(DatabaseSettings {
                path: Some(dir.path().to_path_buf()),
                cache_size: None,
                index_type: IndexType::HashMap,
                ..Default::default()
            });

            db.put("key", "value".as_bytes()).unwrap();
            db.put("another_key", "another_value".as_bytes()).unwrap();
            db.persist().unwrap();
        }

        let mut db = DefaultDatabase::new(DatabaseSettings {
            path: Some(dir.path().to_path_buf()),
            cache_size: None,
            index_type: IndexType::HashMap,
            ..Default::default()
        });

        assert_eq!(db.get("key").unwrap().unwrap(), "value".as_bytes());
        assert!(db.get("another_key").unwrap().is_some());
        assert!(db.get("non_existent_key").unwrap().is_none());
    }

    #[test]
    fn test_linking_data() {
        let dir = tempfile::tempdir().unwrap();
        {
            let mut db = DefaultDatabase::new(DatabaseSettings {
                path: Some(dir.path().to_path_buf()),
                cache_size: None,
                index_type: IndexType::HashMap,
                ..Default::default()
            });

            db.put("key", "value".as_bytes()).unwrap();
            db.put("another_key", "another_value".as_bytes()).unwrap();
            db.persist().unwrap();
        }

        let mut db = DefaultDatabase::new(DatabaseSettings {
            path: Some(dir.path().to_path_buf()),
            cache_size: None,
            index_type: IndexType::HashMap,
            ..Default::default()
        });

        // Easy test
        db.link("key", "lined_key").unwrap();
        assert_eq!(db.get("lined_key").unwrap().unwrap(), "value".as_bytes());

        // Overriding test
        db.link("another_key", "linked_key").unwrap();
        assert_eq!(
            db.get("linked_key").unwrap().unwrap(),
            "another_value".as_bytes()
        );

        // Override existing key
        db.link("linked_key", "key").unwrap();
        assert_eq!(db.get("key").unwrap().unwrap(), "another_value".as_bytes());

        // Now link to a non-existent key, check that it was unsuccessful
        assert!(db.link("non_existent_key", "key").is_err());
        assert_eq!(db.get("key").unwrap().unwrap(), "another_value".as_bytes());
    }

    #[test]
    fn test_binary_data() {
        let some_dummy_string = "hello world";
        let encoded = bincode::serialize(&some_dummy_string).unwrap();

        let some_other_dummy_string = "hello world 2";
        let encoded2 = bincode::serialize(&some_other_dummy_string).unwrap();

        let dir = tempfile::tempdir().unwrap();
        {
            let mut db = DefaultDatabase::new(DatabaseSettings {
                path: Some(dir.path().to_path_buf()),
                cache_size: None,
                index_type: IndexType::HashMap,
                ..Default::default()
            });

            db.put("key", &encoded).unwrap();
            db.put("another_key", &encoded2).unwrap();
            db.persist().unwrap();
        }

        let mut db = DefaultDatabase::new(DatabaseSettings {
            path: Some(dir.path().to_path_buf()),
            cache_size: None,
            index_type: IndexType::HashMap,
            ..Default::default()
        });

        assert_eq!(db.get("key").unwrap().unwrap(), encoded);
        assert_eq!(db.get("another_key").unwrap().unwrap(), encoded2);
    }

    #[test]
    fn force_fragmentation_test() {
        let dir = tempfile::tempdir().unwrap();
        {
            let mut db = DefaultDatabase::new(DatabaseSettings {
                path: Some(dir.path().to_path_buf()),
                cache_size: None,
                index_type: IndexType::HashMap,
                ..Default::default()
            });

            for i in 0..10000 {
                db.put(
                    format!("key{}", i).as_str(),
                    format!("value{}", i).as_bytes(),
                )
                .unwrap();
            }

            db.persist().unwrap();
        }

        let mut db = DefaultDatabase::new(DatabaseSettings {
            path: Some(dir.path().to_path_buf()),
            cache_size: None,
            index_type: IndexType::HashMap,
            ..Default::default()
        });

        assert_eq!(db.get("key0").unwrap().unwrap(), "value0".as_bytes());
        assert_eq!(db.get("key4242").unwrap().unwrap(), "value4242".as_bytes());
        assert_eq!(db.get("key9998").unwrap().unwrap(), "value9998".as_bytes());
        assert_eq!(db.get("key9999").unwrap().unwrap(), "value9999".as_bytes());
    }

    #[test]
    fn test_multithreaded_read() {
        let dir = tempfile::tempdir().unwrap();
        {
            let mut db = DefaultDatabase::new(DatabaseSettings {
                path: Some(dir.path().to_path_buf()),
                cache_size: None,
                index_type: IndexType::HashMap,
                ..Default::default()
            });

            for i in 0..10000 {
                db.put(
                    format!("key{}", i).as_str(),
                    format!("value{}", i).as_bytes(),
                )
                .unwrap();
            }
            db.persist().unwrap();
        }

        let num_threads = 10;
        let barrier = std::sync::Arc::new(std::sync::Barrier::new(num_threads));

        let join_handles = (0..num_threads)
            .map(|_| {
                let barrier = barrier.clone();
                let dir = dir.path().to_path_buf();
                std::thread::spawn(move || {
                    let mut db = DefaultDatabase::new(DatabaseSettings {
                        path: Some(dir),
                        cache_size: None,
                        index_type: IndexType::HashMap,
                        ..Default::default()
                    });
                    barrier.wait();

                    for i in 0..10000 {
                        let key = format!("key{}", i);
                        let value = format!("value{}", i);
                        assert_eq!(db.get(key.as_str()).unwrap().unwrap(), value.as_bytes());
                    }
                })
            })
            .collect::<Vec<_>>();

        for handle in join_handles {
            handle.join().unwrap();
        }
    }

    #[test]
    fn tests_around_buffering() {
        let temp_dir = tempfile::tempdir().unwrap();
        let mut db = DefaultDatabase::new(DatabaseSettings {
            path: Some(temp_dir.path().to_path_buf()),
            cache_size: None,
            index_type: IndexType::HashMap,
            ..Default::default()
        });

        // Case 1: Load all data into buffer, so store < 4096 bytes
        println!("Case 1");
        for i in 0..10 {
            let key = format!("key{}", i);
            let value = format!("value{}", i); // 6-7 bytes
            db.put(key.as_str(), value.as_bytes()).unwrap();
        }

        for i in 0..10 {
            let key = format!("key{}", i);
            let value = format!("value{}", i); // 6-7 bytes
            assert_eq!(db.get(key.as_str()).unwrap().unwrap(), value.as_bytes());
        }

        println!("Case 1 done");

        // clear buffer
        db.persist().unwrap();

        // Case 2: Store 2 objects with 4000 bytes each, then retrieve them both
        println!("Case 2");
        let mut big_value = Vec::new();
        for _ in 0..4000 {
            big_value.push(1);
        }

        db.put("key1", &big_value).unwrap();
        db.put("key2", &big_value).unwrap();

        assert_eq!(db.get("key1").unwrap().unwrap(), big_value.as_slice());
        assert_eq!(db.get("key2").unwrap().unwrap(), big_value.as_slice());

        println!("Case 2 done");
        db.persist().unwrap();

        // Case 3: Store object larger than 4096 bytes, then retrieve it
        println!("Case 3");
        let mut big_value = Vec::new();
        for _ in 0..5000 {
            big_value.push(0);
        }

        db.put("key3", &big_value).unwrap();
        assert_eq!(db.get("key3").unwrap().unwrap(), big_value.as_slice());

        println!("Case 3 done");
    }

    #[test]
    fn lots_of_write_and_reads() {
        let temp_dir = tempfile::tempdir().unwrap();
        let mut db = DefaultDatabase::new(DatabaseSettings {
            path: Some(temp_dir.path().to_path_buf()),
            cache_size: None,
            index_type: IndexType::HashMap,
            ..Default::default()
        });

        let mut rng = rand::thread_rng();
        let mut keys: Vec<String> = Vec::new();
        let mut values: Vec<String> = Vec::new();

        let mut c = 0;
        for i in 0..100_000 {
            if i > 10 && rng.gen_bool(0.4) {
                let index = rng.gen_range(0..c);
                let data = db.get(keys[index].as_str()).unwrap().unwrap();
                assert_eq!(data, values[index].as_bytes());
            } else {
                let key = format!("key{}", i);
                let value = format!("value{}", i);
                db.put(key.as_str(), value.as_bytes()).unwrap();
                c += 1;
                keys.push(key);
                values.push(value);
            }
        }
    }

    #[test]
    #[cfg(feature = "garbage-collection")]
    fn test_garbage_collect() {
        let temp_dir = tempfile::tempdir().unwrap();
        {
            let mut db = DefaultDatabase::new(DatabaseSettings {
                path: Some(temp_dir.path().to_path_buf()),
                cache_size: None,
                index_type: IndexType::HashMap,
                ..Default::default()
            });

            db.put("key1", "value1".as_bytes()).unwrap();
            db.put("key2", "value2".as_bytes()).unwrap();
            db.put("key3", "value3".as_bytes()).unwrap();

            db.delete("key2").unwrap();

            db.persist().unwrap();
        }

        // Now we want to see the file size of the database / folder
        let mut total_size = 0;
        for entry in WalkDir::new(temp_dir.path()) {
            let entry = entry.unwrap();
            total_size += entry.metadata().unwrap().len();
        }

        assert!(total_size > 0);

        {
            let mut db = DefaultDatabase::new(DatabaseSettings {
                path: Some(temp_dir.path().to_path_buf()),
                cache_size: None,
                index_type: IndexType::HashMap,
                ..Default::default()
            });

            db.gc().unwrap();
            db.persist().unwrap();
        }

        // Now we want to see the file size of the database / folder
        let mut total_size_after_gc = 0;
        for entry in WalkDir::new(temp_dir.path()) {
            let entry = entry.unwrap();
            total_size_after_gc += entry.metadata().unwrap().len();
        }

        println!("Size difference: {} -> {}", total_size, total_size_after_gc);
        assert!(total_size_after_gc < total_size);
    }

    #[test]
    fn test_create_dir() {
        let temp_dir = tempfile::tempdir().unwrap();
        let _ = DefaultDatabase::new(DatabaseSettings {
            path: Some(temp_dir.path().join("./some_random_temp_dir")),
            create_path: true,
            ..Default::default()
        });
    }
}
