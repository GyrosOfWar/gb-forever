use camino::Utf8Path;
use reqwest::Url;
use tokio::{
    fs::{self, File},
    io::AsyncWriteExt,
};
use tracing::info;
use tracing_subscriber::EnvFilter;

pub type Result<T> = color_eyre::Result<T>;

pub async fn write_concat_text_file(path: &Utf8Path, sources: &[&str]) -> Result<()> {
    let temp_file = path.with_file_name(format!(
        "{}_temp.{}",
        path.file_stem().unwrap(),
        path.extension().unwrap()
    ));
    let mut file = File::create(&temp_file).await?;
    for source in sources {
        let string = format!("file '{}'\n", source);
        file.write_all(string.as_bytes()).await?;
    }

    fs::rename(temp_file, path).await?;

    Ok(())
}

pub async fn fetch_giant_bomb_videos() -> Result<()> {
    let query = "collection:giant-bomb-archive";
    let fields = [
        "title",
        "identifier",
        "date",
        "size",
        "files",
        "external-identifier",
        "collection",
        "format",
        "topics",
    ];
    let fields = fields.join(",");
    let client = reqwest::Client::new();

    let mut url = Url::parse("https://archive.org/services/search/v1/scrape")?;
    url.query_pairs_mut()
        .append_pair("q", query)
        .append_pair("fields", &fields)
        .append_pair("count", "10000");

    let response = client.get(url).send().await?;
    let data: serde_json::Value = response.json().await?;
    info!("{}", serde_json::to_string_pretty(&data)?);

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    fetch_giant_bomb_videos().await?;

    Ok(())
}
