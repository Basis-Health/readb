#[cfg(feature = "remote-cloning")]
mod tests {
    use readb::{
        clone_from, new_index_table, DatabaseSettings, DefaultDatabase, IndexTable, IndexType,
    };
    use std::io::Write;
    use std::path::PathBuf;
    use tokio::sync::OnceCell;
    use warp::Filter;

    static SERVER_STARTED: OnceCell<()> = OnceCell::const_new();

    fn create_mock_files_at_location(dir: PathBuf) {
        let path = dir.join("./.rdb.data");
        let index_path = dir.join("./.rdb.index");

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
    }

    async fn start_mock_server(dir: PathBuf) {
        let d = dir.as_os_str().to_str().unwrap().to_string();
        SERVER_STARTED
            .get_or_init(|| async move {
                let content = warp::path("content")
                    .and(warp::path::param::<String>())
                    .map(move |filetype: String| {
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
    async fn remote_clone_test() {
        let dir = tempfile::tempdir().unwrap();
        let data_dir = dir.path().to_path_buf().join("data");
        // Create dir
        std::fs::create_dir_all(data_dir.clone()).unwrap();

        create_mock_files_at_location(data_dir.clone());
        start_mock_server(data_dir.clone()).await;

        let database_dir = dir
            .path()
            .to_path_buf()
            .join("database")
            .as_os_str()
            .to_str()
            .unwrap()
            .to_string();
        std::fs::create_dir_all(database_dir.clone()).unwrap();

        clone_from("http://localhost:3030/content", database_dir.as_str(), None)
            .await
            .unwrap();

        // Now let's create the database
        let mut db = DefaultDatabase::new(DatabaseSettings {
            path: Some(PathBuf::from(database_dir)),
            cache_size: None,
            index_type: IndexType::HashMap,
        })
            .unwrap();

        let existent_value = db.get("hi").unwrap();
        assert_eq!(existent_value.unwrap(), "hello".to_string());

        let non_existent_value = db.get("non-existent").unwrap();
        assert!(non_existent_value.is_none());
    }
}
