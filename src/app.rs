use crate::core::ytdownload;

use iced::{Element, Length};
use iced::widget::{Column, column, row, text_input, button, text, scrollable};
use iced::widget::scrollable::{AbsoluteOffset, Id};
use iced::Task;

pub struct ApplicationState {
    logs: Vec<String>,
    yt_url: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    YoutubeUrlChanged(String),
    DownloadYt,
    Completed,
    DownloadFailed(String)
}

const LOGS_SCROLL_ID: &str = "logs_scroll";

pub trait App {
    fn new(_flags: ()) -> (ApplicationState, Task<Message>);
    fn update(&mut self, message: Message) -> Task<Message>;
    fn view(&self) -> Column<Message>;
}

impl App for ApplicationState {
    fn new(_flags: ()) -> (Self, Task<Message>) {
        (
            Self {
                logs: Vec::new(),
                yt_url: String::new()
            },
            Task::none(),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::YoutubeUrlChanged(str) => {
                self.yt_url = str.clone();
                Task::none()
            }
            Message::DownloadYt => {
                let download_result: Result<String, String> = ytdownload::download();

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
            }
        }
    }

    fn view(&self) -> Column<Message> {
        let title = text("YouTube Playlist Downloader").size(30);
    
        /*let save_directory = text_input("Save Directory", &self.directory)
            //.on_input(Message::DirectoryChanged)
            .padding(10)
            .size(20);*/

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
        //.on_press(Message::CopyLogsPressed)
        .padding(10);

        let clear_logs_button = button("Clear Logs")
        .style(red_button_style)
        //.on_press(Message::ClearLogsPressed)
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
            //save_directory,
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