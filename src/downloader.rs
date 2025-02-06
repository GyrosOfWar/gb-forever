use crate::{db::Database, ia::InternetArchive, Result};
use camino::Utf8PathBuf;
use color_eyre::Report;
use tokio::sync::mpsc;
use tracing::error;

#[derive(Debug)]
pub enum VideoId {
    IaIdentifier(String),
    DatabaseId(i64),
}

#[derive(Clone)]
pub struct DownloadOrchestrator {
    database: Database,
    ia: InternetArchive,
    video_folder: Utf8PathBuf,
}

impl DownloadOrchestrator {
    pub fn new(
        database: Database,
        ia: InternetArchive,
        video_folder: Utf8PathBuf,
    ) -> DownloadOrchestrator {
        Self {
            database,
            ia,
            video_folder,
        }
    }

    async fn resolve_id(&self, id: VideoId) -> Result<String> {
        match id {
            VideoId::IaIdentifier(id) => Ok(id),
            VideoId::DatabaseId(id) => {
                let video = self.database.fetch_video(id).await?;
                Ok(video.identifier)
            }
        }
    }

    pub async fn download_single_video(&self, id: VideoId) -> Result<()> {
        let identifier = self.resolve_id(id).await?;
        let file_path = self
            .ia
            .download_video(&identifier, &self.video_folder)
            .await?;
        self.database
            .set_video_downloaded(&identifier, file_path)
            .await?;

        Ok(())
    }

    pub async fn download_videos(&self, ids: Vec<VideoId>) -> Result<()> {
        let futures = ids.into_iter().map(|id| self.download_single_video(id));
        futures::future::try_join_all(futures).await?;
        Ok(())
    }
}

pub struct BackgroundDownloader {
    downloader: DownloadOrchestrator,
    rx: tokio::sync::mpsc::Receiver<Vec<VideoId>>,
    video_folder: Utf8PathBuf,
}

impl BackgroundDownloader {
    pub fn start_new(
        downloader: DownloadOrchestrator,
        video_folder: Utf8PathBuf,
    ) -> mpsc::Sender<Vec<VideoId>> {
        let (mut this, tx) = Self::new(downloader, video_folder);
        tokio::spawn(async move {
            if let Err(e) = this.run().await {
                tracing::error!("background downloader failed: {e}");
            }
        });
        tx
    }

    fn new(
        downloader: DownloadOrchestrator,
        video_folder: Utf8PathBuf,
    ) -> (Self, mpsc::Sender<Vec<VideoId>>) {
        let (tx, rx) = mpsc::channel(32);

        (
            Self {
                downloader,
                rx,
                video_folder,
            },
            tx,
        )
    }

    async fn run(&mut self) -> Result<()> {
        loop {
            if let Some(ids) = self.rx.recv().await {
                if let Err(e) = self.downloader.download_videos(ids).await {
                    error!("failed to download video: {e}");
                }
            } else {
                break Ok(());
            }
        }
    }
}
