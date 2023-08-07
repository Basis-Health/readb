#[cfg(feature = "write")]
mod tests {
    use readb::{DatabaseSettings, DefaultDatabase};
    use readb::IndexType::HashMap;

    #[test]
    fn test_write_and_read() {
        let tempdir = tempfile::tempdir().unwrap();
        {
            let mut database = DefaultDatabase::new(DatabaseSettings {
                path: Some(tempdir.path().to_path_buf()),
                cache_size: None,
                index_type: HashMap,
            }).unwrap();

            database.put("key", "value").unwrap();
            database.put("another_key", "another_value").unwrap();
            database.persist().unwrap();
        }

        {
            let mut database = DefaultDatabase::new(DatabaseSettings {
                path: Some(tempdir.path().to_path_buf()),
                cache_size: None,
                index_type: HashMap,
            }).unwrap();

            assert_eq!(database.get("key").unwrap().unwrap(), "value");
            assert_eq!(database.get("another_key").unwrap().unwrap(), "another_value");
        }
    }
}