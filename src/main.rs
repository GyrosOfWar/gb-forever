use std::env;

use camino::Utf8Path;
use gb_forever::{db::Database, ia::InternetArchive, Result};
use tracing::info;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let _ = dotenvy::dotenv();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let url = env::var("DATABASE_URL")?;
    let database = Database::connect(&url).await?;
    let ia = InternetArchive::default();

    for _ in 0..10 {
        let video = database.random_video().await?;
        ia.download_video(&video.identifier, Utf8Path::new("videos"))
            .await?;
    }

    Ok(())
}
