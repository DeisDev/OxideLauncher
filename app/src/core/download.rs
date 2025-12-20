//! File download utilities and progress tracking.
//!
//! Oxide Launcher — A Rust-based Minecraft launcher
//! Copyright (C) 2025 Oxide Launcher contributors
//!
//! This file is part of Oxide Launcher.
//!
//! Oxide Launcher is free software: you can redistribute it and/or modify
//! it under the terms of the GNU General Public License as published by
//! the Free Software Foundation, either version 3 of the License, or
//! (at your option) any later version.
//!
//! Oxide Launcher is distributed in the hope that it will be useful,
//! but WITHOUT ANY WARRANTY; without even the implied warranty of
//! MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
//! GNU General Public License for more details.
//!
//! You should have received a copy of the GNU General Public License
//! along with this program. If not, see <https://www.gnu.org/licenses/>.

#![allow(dead_code)] // Download types will be used as features are completed

use std::path::PathBuf;
use std::time::Duration;
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
    Retrying { url: String, attempt: u32, max_retries: u32, error: String },
}

/// Download options
#[derive(Debug, Clone)]
pub struct DownloadOptions {
    pub timeout_seconds: u64,
    pub retries: u32,
}

impl Default for DownloadOptions {
    fn default() -> Self {
        Self {
            timeout_seconds: 30,
            retries: 2,
        }
    }
}

/// Download a file to a path with retry support
pub async fn download_file(
    url: &str,
    dest: &PathBuf,
    progress_tx: Option<mpsc::Sender<DownloadProgress>>,
) -> Result<()> {
    download_file_with_options(url, dest, progress_tx, DownloadOptions::default()).await
}

/// Download a file to a path with custom options and retry support
pub async fn download_file_with_options(
    url: &str,
    dest: &PathBuf,
    progress_tx: Option<mpsc::Sender<DownloadProgress>>,
    options: DownloadOptions,
) -> Result<()> {
    let mut last_error = None;
    
    for attempt in 0..=options.retries {
        match download_file_inner(url, dest, progress_tx.clone(), options.timeout_seconds).await {
            Ok(()) => return Ok(()),
            Err(e) => {
                last_error = Some(e.to_string());
                
                if attempt < options.retries {
                    // Notify about retry
                    if let Some(tx) = &progress_tx {
                        let _ = tx.send(DownloadProgress::Retrying {
                            url: url.to_string(),
                            attempt: attempt + 1,
                            max_retries: options.retries,
                            error: last_error.clone().unwrap_or_default(),
                        }).await;
                    }
                    
                    // Exponential backoff: 1s, 2s, 4s...
                    let delay = Duration::from_secs(1 << attempt);
                    tokio::time::sleep(delay).await;
                }
            }
        }
    }
    
    // All retries exhausted
    if let Some(tx) = &progress_tx {
        let _ = tx.send(DownloadProgress::Failed {
            url: url.to_string(),
            error: last_error.clone().unwrap_or_else(|| "Unknown error".to_string()),
        }).await;
    }
    
    Err(OxideError::Download(format!(
        "Failed to download {} after {} retries: {}",
        url,
        options.retries,
        last_error.unwrap_or_else(|| "Unknown error".to_string())
    )))
}

/// Internal download function (single attempt)
async fn download_file_inner(
    url: &str,
    dest: &PathBuf,
    progress_tx: Option<mpsc::Sender<DownloadProgress>>,
    timeout_seconds: u64,
) -> Result<()> {
    // Ensure parent directory exists
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(timeout_seconds))
        .build()?;
    
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
    download_file_verified_with_options(url, dest, expected_sha1, progress_tx, DownloadOptions::default()).await
}

/// Download a file and verify its SHA1 hash with custom options
pub async fn download_file_verified_with_options(
    url: &str,
    dest: &PathBuf,
    expected_sha1: &str,
    progress_tx: Option<mpsc::Sender<DownloadProgress>>,
    options: DownloadOptions,
) -> Result<()> {
    download_file_with_options(url, dest, progress_tx.clone(), options).await?;
    
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

/// Download multiple files concurrently with default options
pub async fn download_files(
    downloads: Vec<DownloadTask>,
    max_concurrent: usize,
    progress_tx: Option<mpsc::Sender<DownloadProgress>>,
) -> Vec<Result<()>> {
    download_files_with_options(downloads, max_concurrent, progress_tx, DownloadOptions::default()).await
}

/// Download multiple files concurrently with custom options
pub async fn download_files_with_options(
    downloads: Vec<DownloadTask>,
    max_concurrent: usize,
    progress_tx: Option<mpsc::Sender<DownloadProgress>>,
    options: DownloadOptions,
) -> Vec<Result<()>> {
    use futures::stream::FuturesUnordered;
    
    let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(max_concurrent));
    let options = std::sync::Arc::new(options);
    let mut futures = FuturesUnordered::new();
    
    for task in downloads {
        let sem = semaphore.clone();
        let tx = progress_tx.clone();
        let opts = options.clone();
        
        futures.push(async move {
            let _permit = sem.acquire().await.unwrap();
            
            if task.sha1.is_some() {
                download_file_verified_with_options(
                    &task.url, 
                    &task.dest, 
                    task.sha1.as_deref().unwrap_or(""), 
                    tx,
                    (*opts).clone()
                ).await
            } else {
                download_file_with_options(&task.url, &task.dest, tx, (*opts).clone()).await
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
