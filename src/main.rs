use camino::Utf8Path;
use gb_forever::{config::load_config, db::Database, ia::InternetArchive, Result};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let _ = dotenvy::dotenv();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let config = load_config()?;
    let database = Database::connect(&config.database_url).await?;
    let ia = InternetArchive::default();

    let video_path = Utf8Path::new("videos");
    tokio::fs::create_dir_all(&video_path).await?;

    // TODO fetch current video from database
    // TODO start background job to fetch new videos in the background
    // TODO start background job to clean up old videos
    // TODO run ffmpeg
    // TODO update the playlist file when required

    Ok(())
}
