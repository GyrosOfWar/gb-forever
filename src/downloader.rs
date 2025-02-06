use crate::{
    db::{Database, VideoId},
    ia::InternetArchive,
    Result,
};
use camino::Utf8PathBuf;
use tokio::sync::mpsc;
use tracing::error;

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

    async fn resolve_id(&self, id: &VideoId) -> Result<(String, i64)> {
        let video = self.database.fetch_video(id).await?;

        Ok((video.identifier.clone(), video.id))
    }

    pub async fn download_single_video(&self, id: VideoId) -> Result<()> {
        let (identifier, video_id) = self.resolve_id(&id).await?;
        self.database.set_video_pending(video_id).await?;

        let file_path = self
            .ia
            .download_video(&identifier, &self.video_folder)
            .await?;
        self.database
            .set_video_downloaded(video_id, file_path)
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
}

impl BackgroundDownloader {
    pub fn start_new(downloader: DownloadOrchestrator) -> mpsc::Sender<Vec<VideoId>> {
        let (mut this, tx) = Self::new(downloader);
        tokio::spawn(async move {
            if let Err(e) = this.run().await {
                tracing::error!("background downloader failed: {e}");
            }
        });
        tx
    }

    fn new(downloader: DownloadOrchestrator) -> (Self, mpsc::Sender<Vec<VideoId>>) {
        let (tx, rx) = mpsc::channel(32);

        (Self { downloader, rx }, tx)
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
