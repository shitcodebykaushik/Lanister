use actix_web::{web, HttpResponse, Responder, http::header};
use tokio::fs::{self, File};
use tokio::io::AsyncWriteExt;
use chrono::Utc;
use bytes::BytesMut;
use futures_util::TryStreamExt;
use std::collections::HashSet;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;

const UPLOAD_DIR: &str = "./uploads";

type FileState = Arc<RwLock<HashSet<String>>>;

#[actix_web::route("/files", method = "PUT")]
pub async fn upload_file(
    mut payload: web::Payload,
    available_files: web::Data<FileState>,
) -> impl Responder {
    let mut field_data = BytesMut::new();

    while let Ok(Some(chunk)) = payload.try_next().await {
        println!("Received chunk of size: {}", chunk.len());
        field_data.extend_from_slice(&chunk);
    }

    let timestamp = Utc::now().timestamp();
    let filename = format!("{}/{}_uploaded_file", UPLOAD_DIR, timestamp);

    match File::create(&filename).await {
        Ok(mut file) => {
            if let Err(err) = file.write_all(&field_data).await {
                return HttpResponse::InternalServerError().body(format!("Error writing file: {}", err));
            }
        }
        Err(err) => {
            return HttpResponse::InternalServerError().body(format!("Error creating file: {}", err));
        }
    }

    let mut files = available_files.write().await;
    files.insert(filename.clone());
    println!("File added to available set: {}", filename);
    println!("Available files after upload: {:?}", *files); 

    HttpResponse::Ok().body(format!("File uploaded as: {}", filename))
}

#[actix_web::route("/files/{filename}", method = "GET")]
pub async fn download_file(
    path: web::Path<String>,
    available_files: web::Data<FileState>,
) -> impl Responder {
    let filename = path.into_inner();
    let filepath = format!("{}/{}", UPLOAD_DIR, filename);

    println!("Requested file: {}", filepath);

    {
        let files = available_files.read().await;
        println!("Available files before download: {:?}", *files); // Debug log
        if !files.contains(&filepath) {
            println!("File not available: {}", filepath);
            return HttpResponse::NotFound().body("File not available or already downloaded and deleted");
        }
    }

    if Path::new(&filepath).exists() {
        match fs::read(&filepath).await {
            Ok(file_content) => {
                {
                    let mut files = available_files.write().await;
                    files.remove(&filepath);
                    println!("File removed from available set: {}", filepath);
                    println!("Available files after removal: {:?}", *files); 
                }

                if let Err(err) = fs::remove_file(&filepath).await {
                    println!("Error deleting file: {}", err);
                    return HttpResponse::InternalServerError().body("Failed to delete file after download");
                }

                println!("File served and deleted: {}", filepath);

                HttpResponse::Ok()
                    .insert_header((header::CONTENT_TYPE, "application/octet-stream"))
                    .insert_header((
                        header::CONTENT_DISPOSITION,
                        format!("attachment; filename=\"{}\"", filename),
                    ))
                    .body(file_content)
            }
            Err(err) => {
                eprintln!("Error reading file: {}", err);
                HttpResponse::InternalServerError().body(format!("Error reading file: {}", err))
            }
        }
    } else {
        println!("File not found: {}", filepath);
        HttpResponse::NotFound().body("File not found")
    }
}
