use camino::Utf8PathBuf;
use tokio::process::Command;
use tracing::error;

use crate::Result;

fn stream_inner(concat_file: String, stream_key: String, cwd: Utf8PathBuf) -> Result<()> {
    let destination = format!("rtmp://live-ber.twitch.tv/app/{stream_key}");

    let process = Command::new("ffmpeg")
        .current_dir(cwd)
        .args(vec![
            "-re",
            "-f",
            "concat",
            "-",
            &concat_file,
            "-preset",
            "veryfast",
            "-b:v",
            "6000k",
            "-maxrate",
            "6000k",
            "-bufsize",
            "6000k",
            "-pix_fmt",
            "yuv420p",
            "-g",
            "50",
            "-c:a",
            "aac",
            "-b:a",
            "160k",
            "-ac",
            "2",
            "-ar",
            "44100",
            "-f",
            "flv",
            &destination,
        ])
        .spawn()?;

    Ok(())
}

pub fn start_stream(concat_file: String, stream_key: String, cwd: Utf8PathBuf) {
    tokio::spawn(async move {
        if let Err(e) = stream_inner(concat_file, stream_key, cwd) {
            error!("error starting stream: {}", e);
        }
    });
}
