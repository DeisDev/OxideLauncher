//! News fetching command for retrieving launcher news from the website.
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

use serde::{Deserialize, Serialize};
use tracing::debug;

const NEWS_API_URL: &str = "https://oxidelauncher.org/news/index.json";

/// A single news article
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsArticle {
    pub id: String,
    pub slug: String,
    pub title: String,
    pub summary: String,
    pub date: String,
    pub category: String,
    pub author: String,
}

/// Response from the news API
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct NewsApiResponse {
    articles: Vec<NewsArticle>,
    last_updated: String,
}

#[tauri::command]
pub async fn get_news() -> Result<Vec<NewsArticle>, String> {
    debug!("Fetching news from {}", NEWS_API_URL);
    
    let client = reqwest::Client::builder()
        .user_agent(format!("OxideLauncher/{}", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;
    
    let response = client
        .get(NEWS_API_URL)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch news: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!("News API returned error: {}", response.status()));
    }
    
    let data: NewsApiResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse news response: {}", e))?;
    
    debug!("Fetched {} news articles", data.articles.len());
    Ok(data.articles)
}
