use camino::Utf8PathBuf;
use color_eyre::Result;
use tokio::{
    fs::{self, File},
    io::AsyncWriteExt,
};

pub struct ConcatFile {
    path: Utf8PathBuf,
    entries: Vec<Utf8PathBuf>,
}

impl ConcatFile {
    fn file_content(&self) -> String {
        let entries: Vec<_> = self.entries.iter().map(|e| format!("file '{e}'")).collect();
        entries.join("\n")
    }

    pub async fn append_video(&mut self, path: Utf8PathBuf) -> Result<()> {
        self.entries.push(path);

        let temp_file_path = self.path.with_file_name(format!(
            "{}_temp.{}",
            self.path.file_stem().unwrap(),
            self.path.extension().unwrap()
        ));
        let mut temp_file = File::create(&temp_file_path).await?;
        let string = self.file_content();
        temp_file.write_all(string.as_bytes()).await?;

        drop(temp_file);

        fs::rename(&temp_file_path, &self.path).await?;

        Ok(())
    }
}
