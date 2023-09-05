mod tests {
    use readb::{Database, DatabaseSettings, DefaultDatabase, IndexType};

    // TODO: Remove test and unwrap with 1.0
    #[allow(deprecated)]
    #[test]
    fn test_unwrap_creation() {
        let dir = tempfile::tempdir().unwrap();

        let _ = DefaultDatabase::new(DatabaseSettings {
            path: Some(dir.path().to_path_buf()),
            cache_size: None,
            index_type: IndexType::HashMap,
            ..Default::default()
        })
        .unwrap();
    }
}
