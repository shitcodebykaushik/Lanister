mod handlers;

use actix_web::{web, App, HttpServer};
use tokio::sync::RwLock;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::fs;

const UPLOAD_DIR: &str = "./uploads";

type FileState = Arc<RwLock<HashSet<String>>>;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    ensure_upload_dir_exists().await;

    let available_files: FileState = Arc::new(RwLock::new(HashSet::new()));

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(available_files.clone()))
            .service(handlers::upload_file)
            .service(handlers::download_file)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

async fn ensure_upload_dir_exists() {
    if let Err(err) = fs::create_dir_all(UPLOAD_DIR).await {
        eprintln!("Failed to create uploads directory: {}", err);
    }
}
