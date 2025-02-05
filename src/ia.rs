use crate::Result;
use async_stream::try_stream;
use futures::Stream;
use reqwest::Url;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct MetadataItem {
    #[serde(rename = "addeddate")]
    pub added_date: String,
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
            .await?
            .json()
            .await
            .map_err(From::from)
    }

    pub fn search_all<'a>(
        &'a self,
        query: &'a str,
    ) -> impl Stream<Item = Result<MetadataItem>> + 'a {
        try_stream! {
            let mut cursor = None;
            loop {
                let response = self.search(query, 10_000, cursor).await?;
                for item in response.items {
                    yield item;
                }
                cursor = response.cursor;
            }
        }
    }
}
