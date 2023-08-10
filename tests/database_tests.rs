#[cfg(feature = "write")]
mod tests {
    use readb::{DatabaseSettings, DefaultDatabase, IndexType};

    #[test]
    fn test_simple_read_after_creation() {
        let dir = tempfile::tempdir().unwrap();
        {
            let mut db = DefaultDatabase::new(DatabaseSettings {
                path: Some(dir.path().to_path_buf()),
                cache_size: None,
                index_type: IndexType::HashMap,
            })
            .unwrap();

            db.put("key", "value".as_bytes()).unwrap();
            db.put("another_key", "another_value".as_bytes()).unwrap();
            db.persist().unwrap();
        }

        let mut db = DefaultDatabase::new(DatabaseSettings {
            path: Some(dir.path().to_path_buf()),
            cache_size: None,
            index_type: IndexType::HashMap,
        })
        .unwrap();

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
            })
            .unwrap();

            db.put("key", "value".as_bytes()).unwrap();
            db.put("another_key", "another_value".as_bytes()).unwrap();
            db.persist().unwrap();
        }

        let mut db = DefaultDatabase::new(DatabaseSettings {
            path: Some(dir.path().to_path_buf()),
            cache_size: None,
            index_type: IndexType::HashMap,
        })
        .unwrap();

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
            })
            .unwrap();

            db.put("key", &encoded).unwrap();
            db.put("another_key", &encoded2).unwrap();
            db.persist().unwrap();
        }

        let mut db = DefaultDatabase::new(DatabaseSettings {
            path: Some(dir.path().to_path_buf()),
            cache_size: None,
            index_type: IndexType::HashMap,
        })
        .unwrap();

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
            })
            .unwrap();

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
        })
        .unwrap();

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
            })
            .unwrap();

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
                    })
                    .unwrap();
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
}
