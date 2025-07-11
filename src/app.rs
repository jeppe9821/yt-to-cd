use std::path::{Path, PathBuf};
use std::thread::spawn;

use crate::core::ytdownload;

use iced::{Length};
use iced::widget::{Column, column, row, text_input, button, text, scrollable};
use iced::widget::scrollable::{Id};
use iced::Task;
use std::fs;

use std::process::Command as ProcessCommand;

use crate::binary_inc;

pub struct ApplicationState {
    pub logs: Vec<String>,
    yt_url: String,
    directory_path: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    DirectoryPathChanged(String),
    YoutubeUrlChanged(String),
    DownloadYt,
    Completed,
    DownloadFailed(String),
    CopyLogsPressed,
    ClearLogsPressed,
}

const LOGS_SCROLL_ID: &str = "logs_scroll";

pub trait App {
    fn new(_flags: ()) -> (ApplicationState, Task<Message>);
    fn install_ytdlp(&mut self) -> PathBuf;
    fn download(&mut self) -> Result<String,String>;
    fn update(&mut self, message: Message) -> Task<Message>;
    fn view(&self) -> Column<Message>;
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

impl App for ApplicationState {
    fn new(_flags: ()) -> (Self, Task<Message>) {
        
        let music_dir: String = dirs::home_dir()
            .unwrap_or(PathBuf::new())
            .join("Music")
            .join("PLAYLIST_NAME")
            .to_string_lossy()
            .to_string();

        (
            Self {
                logs: Vec::new(),
                yt_url: String::new(),
                directory_path: music_dir
            },
            Task::none(),
        )
    }

    fn install_ytdlp(&mut self) -> PathBuf {
        let ytdlp_path = binary_inc::save_ytdlp_to_file()
            .unwrap();

        if let Ok(output) = ProcessCommand::new(&ytdlp_path).arg("--version").output() {
            if output.status.success() {
                let prnt: String = String::from_utf8(output.stdout)
                    .unwrap();

                self.logs.push(format!("yt-dlp is running on version {prnt}"));
            } else {
                self.logs.push("yt-dlp could not be installed".to_owned());
                self.logs.push(format!("Exit code: {:?}", output.status.code()));
                self.logs.push(format!("stdout: {}", String::from_utf8_lossy(&output.stdout)));
                self.logs.push(format!("stderr: {}", String::from_utf8_lossy(&output.stderr)));
            }
        }

        ytdlp_path
    }

    fn download(&mut self) -> Result<String,String> {
        self.logs.push("Installing yt-dlp".to_owned());

        let ytdlp_path: PathBuf = self.install_ytdlp();

        let download_dir: PathBuf = Path::new(&self.directory_path).to_path_buf();
        let output_template: PathBuf = download_dir.join("%(title)s.%(ext)s");

        if Path::exists(download_dir.as_path()) == false {
            let _ = fs::create_dir(&download_dir);
        }

        let mut command = ProcessCommand::new(ytdlp_path);
        command
            .arg("--extract-audio")
            .arg("--format").arg("bestaudio/best")
            .arg("--ignore-errors")
            .arg("--no-playlist")
            .arg("--continue")
            .arg("--no-abort-on-error")
            .arg("-o").arg(output_template.to_string_lossy().to_string())
            .arg(&self.yt_url)
            .current_dir(download_dir);
        
        command.spawn()
            .expect("Failed to start");

        Ok("Result goes here".to_string())
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::DirectoryPathChanged(str) => {
                self.directory_path = str.clone();
                Task::none()
            }
            Message::YoutubeUrlChanged(str) => {
                self.yt_url = str.clone();
                Task::none()
            }
            Message::DownloadYt => {
                let download_result: Result<String, String> = self.download();

                let result = match download_result {
                    Ok(_) => Message::Completed,
                    Err(e) => Message::DownloadFailed(e)
                };

                Task::done(result)
            },
            Message::Completed => {
                println!("The files has been downloaded correctly");
                Task::none()
            },
            Message::DownloadFailed(error) => {
                println!("The download just failed! {error}");
                Task::none()
            },
            Message::CopyLogsPressed => {
                Task::none()
            },
            Message::ClearLogsPressed => {
                Task::none()
            }
        }
    }

    fn view(&self) -> Column<Message> {
        let title = text("YouTube Playlist Downloader").size(30);
    
        let save_directory = text_input("Save Directory", &self.directory_path)
            .on_input(Message::DirectoryPathChanged)
            .padding(10)
            .size(20);

        let input = text_input("Youtube URL", &self.yt_url)
            .on_input(Message::YoutubeUrlChanged)
            .padding(10)
            .size(20);
    
        let download_button = button("Download")
            .on_press(Message::DownloadYt)
            .padding(10)
            .style(red_button_style);
    
    
        let action_button_row = row![
            download_button,
        ]
        .spacing(10);
    
        let copy_logs_button = button("Copy Logs")
        .style(red_button_style)
        .on_press(Message::CopyLogsPressed)
        .padding(10);

        let clear_logs_button = button("Clear Logs")
        .style(red_button_style)
        .on_press(Message::ClearLogsPressed)
        .padding(10);
    
        let log_button_row = row![
            copy_logs_button,
            clear_logs_button
        ]
        .spacing(10);
    
        let logs_text = self.logs.join("\n");
        
        let logs_display = text(logs_text)
            .size(14)
            .font(iced::Font::MONOSPACE);
    
        let scroll = scrollable(
            column![
                logs_display,
            ]
        )
        .height(Length::Fill)
        .id(Id::new(LOGS_SCROLL_ID));
    
        column![
            title,
            save_directory,
            input,
            action_button_row,
            text("Logs:").size(20),
            scroll,
            log_button_row
        ]
        .padding(20)
        .spacing(15)
        .into()
    }
} 


fn red_button_style(_theme: &iced::Theme, _status: iced::widget::button::Status) -> iced::widget::button::Style {
    iced::widget::button::Style {
        background: Some(iced::Background::Color(iced::Color::from_rgb(0.6, 0.2, 0.2))),
        text_color: iced::Color::WHITE,
        ..Default::default()
    }
}