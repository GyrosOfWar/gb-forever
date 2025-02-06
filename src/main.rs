use gb_forever::{
    config::load_config,
    db::{Database, VideoId},
    downloader::{BackgroundDownloader, DownloadOrchestrator},
    ia::InternetArchive,
    Result,
};

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
    let downloader =
        DownloadOrchestrator::new(database.clone(), ia.clone(), config.video_path.clone());

    tokio::fs::create_dir_all(&config.video_path).await?;

    match database.current_video().await? {
        Some(_) => todo!(),
        None => {
            // no videos downloaded yet, get the first five videos
            let items = database.peek_next_videos(5).await?;
            downloader
                .download_videos(
                    items
                        .iter()
                        .map(|item| VideoId::DatabaseId(item.video_id))
                        .collect(),
                )
                .await?;
        }
    }

    let download_sender = BackgroundDownloader::start_new(downloader);

    // TODO start background job to fetch new videos in the background
    // TODO start background job to clean up old videos
    // TODO run ffmpeg
    // TODO update the playlist file when required

    Ok(())
}
