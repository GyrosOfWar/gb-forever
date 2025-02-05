use camino::Utf8Path;
use futures::{pin_mut, TryStreamExt};
use gb_forever::ia::InternetArchive;
use gb_forever::Result;
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

        // let details = archive.get_item_details(&item.identifier).await?;
        // info!("got details: {:#?}", details);
    }

    Ok(())
}
