use crate::remote::compression::CompressionType;
use anyhow::Result;
use std::io::Write;
use std::path::PathBuf;

const LOCAL_PREFIX: &str = ".rdb";
const EXTENSIONS: [&str; 3] = ["type", "index", "data"];

pub async fn clone_from(
    address: &str,
    path: &str,
    compression: Option<CompressionType>,
) -> Result<()> {
    let local_path = PathBuf::from(path);
    for e in EXTENSIONS {
        let local_extension = format!("./{}.{}", LOCAL_PREFIX, e);
        let local_file = local_path.join(local_extension);
        let remote_file = format!("{}/{}", address, e);

        clone_from_remote(&remote_file, &local_file, &compression).await?;
    }

    Ok(())
}

async fn clone_from_remote(
    address: &str,
    path: &PathBuf,
    compression: &Option<CompressionType>,
) -> Result<()> {
    let mut response = reqwest::get(address).await?;
    let mut file = std::fs::File::create(path)?;
    while let Some(chunk) = response.chunk().await? {
        file.write_all(&chunk)?;
    }

    if let Some(compression_type) = compression {
        crate::remote::compression::decompress_file(&mut file, compression_type)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tempfile::tempdir;
    use tokio::sync::OnceCell;
    use warp::Filter;

    static SERVER_ADDR: &str = "http://127.0.0.1:8080";
    static SERVER_STARTED: OnceCell<()> = OnceCell::const_new();

    const SERVER_FILE_PATH: &str = "/test/some";

    async fn start_mock_server() {
        SERVER_STARTED
            .get_or_init(|| async {
                let route = warp::any()
                    .map(|| {
                        let mut response =
                            warp::http::Response::new(warp::hyper::Body::from("Hello, World!"));
                        response.headers_mut().insert(
                            "content-type",
                            warp::http::HeaderValue::from_static("text/plain"),
                        );
                        response
                    })
                    .with(warp::log("mock_server"));

                let _ = tokio::spawn(warp::serve(route).run(([127, 0, 0, 1], 8080)));
            })
            .await;
    }

    #[tokio::test]
    async fn test_clone_from() {
        start_mock_server().await;
        tokio::time::sleep(Duration::from_millis(100)).await; // Give the server some time to start

        // Create a temporary directory using the tempfile crate
        let temp = tempdir().unwrap();
        let path = temp.path().to_str().unwrap();

        let compression = Some(CompressionType::Uncompressed);

        let s_path = format!("{}{}", SERVER_ADDR, SERVER_FILE_PATH);
        let result = clone_from(s_path.as_str(), path, compression).await;

        // The temporary directory and its contents will be automatically deleted when `temp` is dropped.
        // You can perform more assertions here if needed, like checking the content of the cloned files.

        assert!(result.is_ok());
    }
}
