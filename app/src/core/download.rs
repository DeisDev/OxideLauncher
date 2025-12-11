//! Download management

#![allow(dead_code)] // Download types will be used as features are completed

use std::path::PathBuf;
use tokio::sync::mpsc;
use futures::StreamExt;
use crate::core::error::{OxideError, Result};

/// Download progress event
#[derive(Debug, Clone)]
pub enum DownloadProgress {
    Started { url: String, total_size: Option<u64> },
    Progress { url: String, downloaded: u64, total: Option<u64> },
    Completed { url: String },
    Failed { url: String, error: String },
}

/// Download a file to a path
pub async fn download_file(
    url: &str,
    dest: &PathBuf,
    progress_tx: Option<mpsc::Sender<DownloadProgress>>,
) -> Result<()> {
    // Ensure parent directory exists
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let client = reqwest::Client::new();
    let response = client.get(url).send().await?;
    
    if !response.status().is_success() {
        return Err(OxideError::Download(format!(
            "HTTP error {}: {}",
            response.status(),
            url
        )));
    }

    let total_size = response.content_length();
    
    if let Some(tx) = &progress_tx {
        let _ = tx.send(DownloadProgress::Started {
            url: url.to_string(),
            total_size,
        }).await;
    }

    let mut file = tokio::fs::File::create(dest).await?;
    let mut downloaded: u64 = 0;
    let mut stream = response.bytes_stream();
    
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        tokio::io::AsyncWriteExt::write_all(&mut file, &chunk).await?;
        
        downloaded += chunk.len() as u64;
        
        if let Some(tx) = &progress_tx {
            let _ = tx.send(DownloadProgress::Progress {
                url: url.to_string(),
                downloaded,
                total: total_size,
            }).await;
        }
    }

    if let Some(tx) = &progress_tx {
        let _ = tx.send(DownloadProgress::Completed {
            url: url.to_string(),
        }).await;
    }

    Ok(())
}

/// Download a file and verify its SHA1 hash
pub async fn download_file_verified(
    url: &str,
    dest: &PathBuf,
    expected_sha1: &str,
    progress_tx: Option<mpsc::Sender<DownloadProgress>>,
) -> Result<()> {
    download_file(url, dest, progress_tx).await?;
    
    // Verify hash
    if !expected_sha1.is_empty() {
        let actual_hash = compute_sha1(dest)?;
        if actual_hash != expected_sha1 {
            // Delete the file if hash doesn't match
            let _ = std::fs::remove_file(dest);
            return Err(OxideError::Download(format!(
                "SHA1 mismatch for {}: expected {}, got {}",
                url, expected_sha1, actual_hash
            )));
        }
    }
    
    Ok(())
}

/// Compute SHA1 hash of a file
pub fn compute_sha1(path: &PathBuf) -> Result<String> {
    use sha1::{Sha1, Digest};
    
    let mut file = std::fs::File::open(path)?;
    let mut hasher = Sha1::new();
    std::io::copy(&mut file, &mut hasher)?;
    
    let hash = hasher.finalize();
    Ok(format!("{:x}", hash))
}

/// Download multiple files concurrently
pub async fn download_files(
    downloads: Vec<DownloadTask>,
    max_concurrent: usize,
    progress_tx: Option<mpsc::Sender<DownloadProgress>>,
) -> Vec<Result<()>> {
    use futures::stream::FuturesUnordered;
    
    let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(max_concurrent));
    let mut futures = FuturesUnordered::new();
    
    for task in downloads {
        let sem = semaphore.clone();
        let tx = progress_tx.clone();
        
        futures.push(async move {
            let _permit = sem.acquire().await.unwrap();
            
            if task.sha1.is_some() {
                download_file_verified(&task.url, &task.dest, task.sha1.as_deref().unwrap_or(""), tx).await
            } else {
                download_file(&task.url, &task.dest, tx).await
            }
        });
    }
    
    let mut results = Vec::new();
    while let Some(result) = futures.next().await {
        results.push(result);
    }
    
    results
}

/// A download task
#[derive(Debug, Clone)]
pub struct DownloadTask {
    pub url: String,
    pub dest: PathBuf,
    pub sha1: Option<String>,
    pub size: Option<u64>,
}

impl DownloadTask {
    pub fn new(url: impl Into<String>, dest: PathBuf) -> Self {
        Self {
            url: url.into(),
            dest,
            sha1: None,
            size: None,
        }
    }

    pub fn with_sha1(mut self, sha1: impl Into<String>) -> Self {
        self.sha1 = Some(sha1.into());
        self
    }

    pub fn with_size(mut self, size: u64) -> Self {
        self.size = Some(size);
        self
    }
}
