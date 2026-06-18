use anyhow::{anyhow, Result};
use axum::extract::Multipart;
use std::path::Path;
use tokio::fs;
use uuid::Uuid;

const MAX_SIZE: usize = 2 * 1024 * 1024;
const ALLOWED_TYPES: &[&str] = &["image/jpeg", "image/png", "image/webp"];

pub async fn save_photo(mut multipart: Multipart, storage_path: &str) -> Result<String> {
    while let Some(field) = multipart.next_field().await? {
        let name = field.name().unwrap_or("").to_string();
        if name != "photo" {
            continue;
        }
        let content_type = field.content_type().unwrap_or("").to_string();
        if !ALLOWED_TYPES.contains(&content_type.as_str()) {
            return Err(anyhow!("Only JPEG, PNG, WebP images are allowed"));
        }
        let ext = match content_type.as_str() {
            "image/jpeg" => "jpg",
            "image/png" => "png",
            "image/webp" => "webp",
            _ => "jpg",
        };
        let data = field.bytes().await?;
        if data.len() > MAX_SIZE {
            return Err(anyhow!("File size must not exceed 2MB"));
        }
        let filename = format!("{}.{}", Uuid::new_v4(), ext);
        fs::create_dir_all(storage_path).await?;
        let path = Path::new(storage_path).join(&filename);
        fs::write(path, &data).await?;
        return Ok(filename);
    }
    Err(anyhow!("No photo field in request"))
}

pub async fn delete_photo(storage_path: &str, filename: &str) {
    let path = Path::new(storage_path).join(filename);
    let _ = fs::remove_file(path).await;
}

pub fn build_photo_url(app_url: &str, filename: &str) -> String {
    format!("{}/storage/photos/{}", app_url, filename)
}
