mod core;

mod app;
use std::path::Path;

use app::ApplicationState;
use app::App;
mod binary_inc;

use iced::window;

use std::process::Command as ProcessCommand;

const APPLICATION_TITLE: &str = "Youtube to CD Downloader";

fn install_ytdlp() -> () {
    let ytdlp_path = binary_inc::save_ytdlp_to_file()
        .unwrap();

    if let Ok(output) = ProcessCommand::new(ytdlp_path).arg("--version").output() {
        if output.status.success() {
            let prnt: String = String::from_utf8(output.stdout)
                .unwrap();

            println!("yt-dlp is running on version {prnt}");
        }
    }
}

fn install_ffmpeg() -> () {
    let ffmpeg_path = binary_inc::save_ffmpeg_to_file()
        .unwrap();

    if let Ok(output) = ProcessCommand::new(ffmpeg_path).arg("-version").output() {
        if output.status.success() {
            let prnt: String = String::from_utf8(output.stdout)
                .unwrap();

            println!("ffmpeg is running on version {prnt}");
        }
    }
}

fn get_window_settings() -> window::Settings {
    let path: &Path = Path::new("assets/cd.png");
    let icon = iced::window::icon::from_file(path)
        .unwrap();

    let window_settings: window::Settings = window::Settings {
        icon: Some(icon),
        ..Default::default()
    };

    window_settings
}

pub fn main() -> iced::Result {
    install_ytdlp();
    install_ffmpeg();

    let window_settings: window::Settings = get_window_settings();
    
    iced::application(APPLICATION_TITLE, 
        ApplicationState::update, 
        ApplicationState::view)
        .window(window_settings)
        .run_with(|| ApplicationState::new(()))
}
