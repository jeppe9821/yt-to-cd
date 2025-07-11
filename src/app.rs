use std::path::{Path, PathBuf};

use iced::{Length};
use iced::widget::{Column, column, row, text_input, button, text, scrollable};
use iced::widget::scrollable::{Id};
use iced::Task;
use std::fs;

use tokio::io::{BufReader, AsyncBufReadExt};
use tokio::process::Command as ProcessCommand;
use std::process::{Stdio};

use std::sync::{Arc, RwLock};

use crate::binary_inc;

use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

pub type SharedLogs = Arc<RwLock<Vec<String>>>;

#[derive(Debug, Clone)]
pub enum Message {
    DirectoryPathChanged(String),
    YoutubeUrlChanged(String),
    DownloadYt,
    Completed,
    DownloadFailed(String),
    CopyLogsPressed,
    ClearLogsPressed,
    LogUpdated(String),
}

const LOGS_SCROLL_ID: &str = "logs_scroll";

#[derive(Clone)]
pub struct ApplicationState {
    yt_url: String,
    directory_path: String,
    state: SharedLogs,
    log_sender: Option<UnboundedSender<Message>>,
}

pub trait App {
    fn new(_flags: ()) -> (ApplicationState, Task<Message>);
    async fn install_ffmpeg() -> PathBuf;
    async fn install_ytdlp() -> PathBuf;
    async fn download(state: SharedLogs, directory_path: String, yt_url: String, log_sender: UnboundedSender<Message>) -> Result<String,String>;
    fn update(&mut self, message: Message) -> Task<Message>;
    fn view(&self) -> Column<Message>;
}

impl App for ApplicationState {
    fn new(_flags: ()) -> (Self, Task<Message>) {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let shared_logs: SharedLogs = Arc::new(RwLock::new(Vec::new()));
        let logs_clone = shared_logs.clone();

        // Create task to consume messages and push to logs
        let log_forward_task = Task::perform(async move {
            tokio::spawn(async move {
                while let Some(msg) = rx.recv().await {
                    if let Message::LogUpdated(log) = msg {
                        logs_clone.write().unwrap().push(log);
                    }
                }
            });

            Message::LogUpdated(String::new()) // dummy result
        }, |_| Message::LogUpdated(String::new()));

        
        let music_dir: String = dirs::home_dir()
            .unwrap_or(PathBuf::new())
            .join("Music")
            .join("PLAYLIST_NAME")
            .to_string_lossy()
            .to_string();

        (
            Self {
                yt_url: String::new(),
                directory_path: music_dir,
                state: shared_logs,
                log_sender: Some(tx),
            },
            log_forward_task
        )
    }

    async fn install_ffmpeg() -> PathBuf {
        let ffmpeg_path = binary_inc::save_ffmpeg_to_file().unwrap();
        let mut command = ProcessCommand::new(&ffmpeg_path);
        command.arg("-version").stderr(Stdio::piped());
        let mut child = command.spawn().expect("Failed to spawn process");
        let stderr = child.stderr.take().expect("No stderr");
        let mut stderr_reader = BufReader::new(stderr).lines();

        tokio::spawn(async move {
            let status = child.wait().await.expect("Process error");
            println!("Process status: {}", status);
        });

        while let Some(line) = stderr_reader.next_line().await.unwrap_or(None) {
            println!("Stderr: {}", line);
        }

        ffmpeg_path
    }

    async fn install_ytdlp() -> PathBuf {
        let ytdlp_path = binary_inc::save_ytdlp_to_file().unwrap();
        let mut command = ProcessCommand::new(&ytdlp_path);
        command.arg("--version").stderr(Stdio::piped());
        let mut child = command.spawn().expect("Failed to spawn process");
        let stderr = child.stderr.take().expect("No stderr");
        let mut stderr_reader = BufReader::new(stderr).lines();

        tokio::spawn(async move {
            let status = child.wait().await.expect("Process error");
            println!("Process status: {}", status);
        });

        while let Some(line) = stderr_reader.next_line().await.unwrap_or(None) {
            println!("Stderr: {}", line);
        }

        ytdlp_path
    }

    async fn download(state: SharedLogs, directory_path: String, yt_url: String, log_sender: UnboundedSender<Message>) -> Result<String,String> {
        {
            let mut logs = state.write().unwrap();
            //logs.push("Installing yt-dlp...".into());
            let _ = log_sender.send(Message::LogUpdated("Installing yt-dlp...".into()));
        }

        let ytdlp_path = ApplicationState::install_ytdlp().await;

        {
            let msg = format!("yt-dlp installed at {}", ytdlp_path.display());
            //state.write().unwrap().push(msg.clone());
            let _ = log_sender.send(Message::LogUpdated(msg));
        }

        {
            //state.write().unwrap().push("Installing ffmpeg...".into());
            let _ = log_sender.send(Message::LogUpdated("Installing ffmpeg...".into()));
        }

        let ffmpeg_path = ApplicationState::install_ffmpeg().await;

        {
            let msg = format!("ffmpeg installed at {}", ffmpeg_path.display());
            //state.write().unwrap().push(msg.clone());
            let _ = log_sender.send(Message::LogUpdated(msg));
        }

        let download_dir = Path::new(&directory_path).to_path_buf();
        let output_template = download_dir.join("%(title)s.%(ext)s");

        if !download_dir.exists() {
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
            .arg(yt_url)
            .stdout(Stdio::piped())
            .current_dir(&download_dir);

        let mut process = command.spawn().expect("Failed to start process");
        let stdout = process.stdout.take().expect("No stdout");
        let mut stdout_reader = BufReader::new(stdout).lines();

        while let Some(line) = stdout_reader.next_line().await.unwrap_or(None) {
            /*{
                state.write().unwrap().push(line.clone());
            }*/
            let _ = log_sender.send(Message::LogUpdated(line.clone()));

            let dis = line.clone();

            println!("{dis}");
        }

        Ok("Download finished".into())
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::DirectoryPathChanged(str) => {
                self.directory_path = str;
                Task::none()
            }
            Message::YoutubeUrlChanged(str) => {
                self.yt_url = str;
                Task::none()
            }
            Message::DownloadYt => {
                let shared = self.state.clone();
                let directory_path = self.directory_path.clone();
                let yt_url = self.yt_url.clone();
                let sender = self.log_sender.clone().unwrap();

                Task::perform(async move {
                    ApplicationState::download(shared, directory_path, yt_url, sender).await
                }, |result| match result {
                    Ok(_) => Message::Completed,
                    Err(e) => Message::DownloadFailed(e),
                })
            },
            Message::Completed => {
                println!("Download completed");
                Task::none()
            },
            Message::DownloadFailed(error) => {
                println!("Download failed: {error}");
                Task::none()
            },
            Message::CopyLogsPressed => Task::none(),
            Message::ClearLogsPressed => Task::none(),
            Message::LogUpdated(_) => Task::none()
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

        let action_button_row = row![download_button].spacing(10);

        let copy_logs_button = button("Copy Logs")
            .style(red_button_style)
            .on_press(Message::CopyLogsPressed)
            .padding(10);

        let clear_logs_button = button("Clear Logs")
            .style(red_button_style)
            .on_press(Message::ClearLogsPressed)
            .padding(10);

        let log_button_row = row![copy_logs_button, clear_logs_button].spacing(10);

        let logs_text = self.state.read().unwrap().join("\n");
        let logs_display = text(logs_text).size(14).font(iced::Font::MONOSPACE);

        let scroll = scrollable(column![logs_display])
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
