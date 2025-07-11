use std::path::PathBuf;
use std::path::Path;
use std::fs;
use std::io::Error;

const FFMPEG_BINARY: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/ffmpeg.exe"));
const YTDLP_BINARY: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/yt-dlp.exe"));

fn save_to_file(file: &str, binary: &[u8]) -> Result<PathBuf, Error> {
    let bin_directory = Path::new("\\bin");
    let exists: bool = Path::exists(bin_directory);

    if exists == false {
        let create_dir_result: Result<(), Error> = fs::create_dir(bin_directory);
        
        let display = bin_directory.display();

        match create_dir_result {
            Ok(_) => println!("Created directory successfully at {display}"),
            Err(e) => println!("Could not create directory at {display}")
        }
    }

    let file_path: PathBuf = bin_directory.join(file);
    std::fs::write(&file_path, binary)?;

    Ok(file_path)
}

pub fn save_ytdlp_to_file() -> Result<PathBuf, Error> {
    save_to_file("yt-dlp.exe", YTDLP_BINARY)
}

pub fn save_ffmpeg_to_file() -> Result<PathBuf, Error> {
    save_to_file("ffmpeg.exe", FFMPEG_BINARY)
}