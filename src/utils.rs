use tokio::fs;

pub async fn ensure_upload_dir_exists() {
    if let Err(err) = fs::create_dir_all("./uploads").await {
        eprintln!("Failed to create uploads directory: {}", err);
    }
}
