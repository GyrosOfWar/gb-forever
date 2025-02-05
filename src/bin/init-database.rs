use futures::{pin_mut, TryStreamExt};
use gb_forever::{db::Database, ia::InternetArchive, Result};
use std::env;
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

    let stream = ia
        .search_all("collection:giant-bomb-archive")
        .try_chunks(10_000);
    pin_mut!(stream);

    while let Some(chunk) = stream.try_next().await? {
        info!("got chunk of {} items", chunk.len());
        database.insert_items(&chunk).await?;
    }

    database.create_random_playlist().await?;

    Ok(())
}
