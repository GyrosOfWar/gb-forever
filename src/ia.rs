use crate::Result;
use async_stream::try_stream;
use camino::Utf8Path;
use color_eyre::eyre::{Context, OptionExt};
use futures::Stream;
use reqwest::Url;
use serde::Deserialize;
use tracing::info;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtendedMetadata {
    pub created: i64,
    pub files: Vec<File>,
    #[serde(rename = "files_count")]
    pub files_count: i64,
    #[serde(rename = "item_last_updated")]
    pub item_last_updated: i64,
    #[serde(rename = "item_size")]
    pub item_size: i64,
    pub server: String,
    pub uniq: i64,
    #[serde(rename = "dir")]
    pub directory: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct File {
    pub crc32: Option<String>,
    pub format: String,
    pub height: Option<String>,
    pub length: Option<String>,
    pub md5: String,
    pub mtime: Option<String>,
    pub name: String,
    pub sha1: Option<String>,
    pub size: Option<String>,
    pub source: String,
    pub width: Option<String>,
    pub original: Option<String>,
    pub btih: Option<String>,
    pub summation: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct MetadataItem {
    pub collections: Option<Vec<String>>,
    pub creator: Option<String>,
    pub date: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "external-identifier")]
    pub external_identifier: Option<String>,
    pub identifier: String,
    pub item_size: Option<u64>,
    pub title: String,
}

#[derive(Debug, Deserialize)]
pub struct MetadataResponse {
    pub count: u64,
    pub cursor: Option<String>,
    pub items: Vec<MetadataItem>,
    pub total: u64,
}

#[derive(Default)]
pub struct InternetArchive {
    client: reqwest::Client,
}

impl InternetArchive {
    pub async fn search(
        &self,
        query: &str,
        count: usize,
        cursor: Option<String>,
    ) -> Result<MetadataResponse> {
        info!("searching for: {}", query);
        let mut url = Url::parse("https://archive.org/services/search/v1/scrape")?;
        url.query_pairs_mut()
            .append_pair("q", query)
            .append_pair("fields", "*")
            .append_pair("count", &count.to_string());
        if let Some(cursor) = cursor {
            url.query_pairs_mut().append_pair("cursor", &cursor);
        }

        self.client
            .get(url)
            .send()
            .await
            .wrap_err("failed to fetch search results")?
            .json()
            .await
            .wrap_err("failed to decode response")
    }

    pub fn search_all<'a>(
        &'a self,
        query: &'a str,
    ) -> impl Stream<Item = Result<MetadataItem>> + 'a {
        try_stream! {
            let mut cursor = None;
            loop {
                info!("fetching items with cursor: {:?}", cursor);
                let response = self.search(query, 10_000, cursor).await?;
                for item in response.items {
                    yield item;
                }
                cursor = response.cursor;

                if cursor.is_none() {
                    break;
                }
            }
        }
    }

    pub async fn get_item_details(&self, identifier: &str) -> Result<ExtendedMetadata> {
        let mut url = Url::parse("https://archive.org")?;
        url.set_path(&format!("/metadata/{}", identifier));
        info!("Making request to {url}");
        self.client
            .get(url)
            .send()
            .await?
            .json()
            .await
            .map_err(From::from)
    }

    pub async fn download_video(&self, identifier: &str, folder: &Utf8Path) -> Result<()> {
        let details = self.get_item_details(identifier).await?;
        let video_file = details
            .files
            .iter()
            .find(|f| f.format == "MPEG4")
            .ok_or_eyre("no video file found")?;

        let url = format!(
            "https://{}{}/{}",
            details.server, details.directory, video_file.name
        );
        info!("Downloading from URL {url}");

        Ok(())
    }
}
