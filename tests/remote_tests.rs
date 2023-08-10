#[cfg(all(feature = "remote-cloning", feature = "write"))]
mod tests {
    use readb::{clone_from, DatabaseSettings, DefaultDatabase, IndexType};
    use std::fs;
    use std::path::PathBuf;
    use tokio::sync::OnceCell;
    use warp::Filter;

    const RANDOM_STRINGS_WITH_KEYS: [(&str, &str); 6] = [
        ("hi", "hello"),
        ("there", "there"),
        ("how", "are"),
        ("you", "doing"),
        ("today", "today"),
        ("?", "?"),
    ];

    static SERVER_STARTED: OnceCell<()> = OnceCell::const_new();

    fn create_database(location: &PathBuf) {
        let mut db = DefaultDatabase::new(DatabaseSettings {
            path: Some(location.to_path_buf()),
            cache_size: None,
            index_type: IndexType::HashMap,
        })
        .unwrap();

        for (_, (key, val)) in RANDOM_STRINGS_WITH_KEYS.iter().enumerate() {
            db.put(key, val.as_bytes()).unwrap();
        }

        db.persist().unwrap();
    }

    async fn start_mock_server(dir: PathBuf) {
        let d = dir.as_os_str().to_str().unwrap().to_string();
        SERVER_STARTED
            .get_or_init(|| async move {
                let content = warp::path("content")
                    .and(warp::path::param::<String>())
                    .map(move |filetype: String| {
                        println!("Reading file: {}", filetype);
                        let file_path = format!("{}/.rdb.{}", d.as_str(), filetype);
                        let content = std::fs::read_to_string(file_path).unwrap();
                        warp::http::Response::builder()
                            .header("content-type", "text/plain")
                            .body(content)
                    })
                    .with(warp::log("mock_server"));

                let _ = tokio::spawn(warp::serve(content).run(([127, 0, 0, 1], 3030)));
            })
            .await;
    }

    #[tokio::test]
    async fn test_simple_read_after_creation() {
        let dir = tempfile::tempdir().unwrap();
        let mock_dir = dir.path().join("./mock");
        fs::create_dir(&mock_dir).unwrap();

        println!("Creating mock database");
        create_database(&mock_dir);

        println!("Starting mock server");
        start_mock_server(mock_dir).await;

        println!("Mock server started");

        let database_dir = dir.path().join("./database");
        fs::create_dir(&database_dir).unwrap();

        println!(
            "Cloning from http://localhost:3030/content to {}",
            database_dir.as_os_str().to_str().unwrap()
        );
        clone_from(
            "http://localhost:3030/content",
            &database_dir.as_os_str().to_str().unwrap(),
            None,
        )
        .await
        .unwrap();

        let mut db = DefaultDatabase::new(DatabaseSettings {
            path: Some(database_dir.to_path_buf()),
            cache_size: None,
            index_type: IndexType::HashMap,
        })
        .unwrap();

        for (key, value) in RANDOM_STRINGS_WITH_KEYS.iter() {
            assert_eq!(db.get(key).unwrap().unwrap(), value.as_bytes());
        }
    }
}
