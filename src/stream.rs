use camino::Utf8PathBuf;

// pub async fn write_concat_text_file(path: &Utf8Path, sources: &[&str]) -> Result<()> {
//     let temp_file = path.with_file_name(format!(
//         "{}_temp.{}",
//         path.file_stem().unwrap(),
//         path.extension().unwrap()
//     ));
//     let mut file = File::create(&temp_file).await?;
//     for source in sources {
//         let string = format!("file '{}'\n", source);
//         file.write_all(string.as_bytes()).await?;
//     }

//     fs::rename(temp_file, path).await?;

//     Ok(())
// }

pub struct ConcatFile {
    path: Utf8PathBuf,
}

impl ConcatFile {}
