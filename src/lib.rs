pub mod config;
pub mod db;
pub mod downloader;
pub mod ffmpeg;
pub mod ia;
pub mod stream;

pub type Result<T> = color_eyre::Result<T>;
