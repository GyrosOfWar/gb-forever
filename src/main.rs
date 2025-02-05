use camino::Utf8Path;
use futures::{pin_mut, TryStreamExt};
use gb_forever::ia::InternetArchive;
use gb_forever::Result;
use reqwest::Url;
use tokio::{
    fs::{self, File},
    io::AsyncWriteExt,
};
use tracing::info;
use tracing_subscriber::EnvFilter;

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

pub async fn get_item_details(item: &str) -> Result<()> {
    let client = reqwest::Client::new();

    let mut url = Url::parse("https://archive.org/metadata/")?;
    url.set_path(item);

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

    let archive = InternetArchive::default();
    let stream = archive.search_all("collection:giant-bomb-archive");

    pin_mut!(stream);

    while let Some(item) = stream.try_next().await? {
        info!("got item: {:#?}", item);
    }

    Ok(())
}
