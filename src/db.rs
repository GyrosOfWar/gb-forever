use std::{fmt, str::FromStr, time::Instant};

use color_eyre::eyre::{bail, Context};
use tracing::info;

use crate::{ia::MetadataItem, Result};

#[derive(Debug)]
pub struct GbVideo {
    pub id: i64,
    pub date: Option<String>,
    pub description: Option<String>,
    pub title: String,
    pub item_size: Option<i64>,
    pub identifier: String,
    pub external_identifier: Option<String>,
    pub collections: Option<Vec<String>>,
    pub creator: Option<String>,
}

#[derive(Debug)]
pub struct PlaylistEntry {
    pub id: i64,
    pub video_id: i64,
    pub status: String,
    pub file_path: Option<String>,
    pub last_progress: Option<i32>,
}

#[derive(Debug, Clone, Copy)]
pub enum PlaylistEntryStatus {
    /// The video has not been downloaded or played
    Unplayed,
    /// The video is currently being downloaded
    Pending,
    /// The video has been downloaded but not yet played
    Downloaded,
    /// The video is currently being played
    Active,
    /// The video has been played
    Finished,
}

impl FromStr for PlaylistEntryStatus {
    type Err = color_eyre::Report;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "active" => Ok(Self::Active),
            "downloaded" => Ok(Self::Downloaded),
            "pending" => Ok(Self::Pending),
            "unplayed" => Ok(Self::Unplayed),
            "finished" => Ok(Self::Finished),
            _ => bail!("invalid playlist entry status: {s}"),
        }
    }
}

impl fmt::Display for PlaylistEntryStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Active => write!(f, "active"),
            Self::Downloaded => write!(f, "downloaded"),
            Self::Pending => write!(f, "pending"),
            Self::Unplayed => write!(f, "unplayed"),
            Self::Finished => write!(f, "finished"),
        }
    }
}

#[derive(Clone)]
pub struct Database {
    pool: sqlx::PgPool,
}

impl Database {
    pub async fn connect(url: &str) -> Result<Self> {
        let pool = sqlx::PgPool::connect(url).await?;
        sqlx::migrate!("./migrations").run(&pool).await?;
        Ok(Self { pool })
    }

    pub async fn insert_items(&self, items: &[MetadataItem]) -> Result<()> {
        if items.is_empty() {
            return Ok(());
        }
        let start = Instant::now();
        let mut tx = self
            .pool
            .begin()
            .await
            .wrap_err("failed to start transaction")?;
        for item in items {
            sqlx::query!(
                r#"
                INSERT INTO gb_videos(
                    "date",
                    "description",
                    title,
                    item_size,
                    identifier,
                    external_identifier,
                    collections,
                    creator
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                ON CONFLICT (identifier) DO NOTHING
                "#,
                item.date,
                item.description,
                item.title,
                item.item_size.map(|v| v as i64),
                item.identifier,
                item.external_identifier,
                item.collections.as_ref().map(|c| c.as_slice()),
                item.creator.as_deref()
            )
            .execute(&mut *tx)
            .await
            .wrap_err("failed to insert item")?;
        }
        tx.commit().await.wrap_err("failed to commit transaction")?;

        let elapsed = start.elapsed();
        info!("inserted {} items in {:?}", items.len(), elapsed);
        Ok(())
    }

    pub async fn random_video(&self) -> Result<GbVideo> {
        sqlx::query_as!(GbVideo, "SELECT * FROM gb_videos ORDER BY random() LIMIT 1")
            .fetch_one(&self.pool)
            .await
            .wrap_err("failed to fetch random video from database")
    }

    pub async fn create_random_playlist(&self) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        sqlx::query!("DELETE FROM playlist_entry")
            .execute(&mut *tx)
            .await?;

        sqlx::query!("DELETE FROM active_playlist_entry")
            .execute(&mut *tx)
            .await?;

        sqlx::query!(
            "INSERT INTO playlist_entry (video_id, status) 
            SELECT id, 'unplayed' FROM gb_videos ORDER BY random()"
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query!("INSERT INTO active_playlist_entry (id, entry_index) VALUES (1, 1)")
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;

        Ok(())
    }

    pub async fn set_video_downloaded(
        &self,
        identifier: &str,
        file_path: impl Into<String>,
    ) -> Result<()> {
        todo!()
    }

    // TOOD add method to update current position in the video

    // TODO add method to get the current video

    pub async fn current_video(&self) -> Result<Option<PlaylistEntry>> {
        let index = sqlx::query_scalar!("SELECT entry_index FROM active_playlist_entry")
            .fetch_one(&self.pool)
            .await?;
        let entry = sqlx::query_as!(
            PlaylistEntry,
            "SELECT * FROM playlist_entry WHERE id = $1",
            index
        )
        .fetch_one(&self.pool)
        .await?;
        let status: PlaylistEntryStatus = entry.status.parse()?;
        if let PlaylistEntryStatus::Active = status {
            Ok(Some(entry))
        } else {
            Ok(None)
        }
    }

    pub async fn peek_next_videos(&self, count: usize) -> Result<Vec<PlaylistEntry>> {
        sqlx::query_as!(
            PlaylistEntry,
            "SELECT * FROM playlist_entry WHERE status = 'unplayed' ORDER BY id LIMIT $1",
            count as i64
        )
        .fetch_all(&self.pool)
        .await
        .wrap_err("failed to fetch next videos from database")
    }

    pub async fn move_to_next_video(&self) -> Result<Option<PlaylistEntry>> {
        let entry = self.current_video().await?;
        if let Some(entry) = entry {
            let next_index = entry.id + 1;
            todo!()
        } else {
            Ok(None)
        }
    }

    pub async fn fetch_video(&self, id: i64) -> Result<GbVideo> {
        sqlx::query_as!(GbVideo, "SELECT * FROM gb_videos WHERE id = $1", id)
            .fetch_one(&self.pool)
            .await
            .wrap_err("failed to fetch video from database")
    }

    // TODO add method to move to the next video (unless it's not downloaded yet)
}

#[cfg(test)]
mod tests {}
