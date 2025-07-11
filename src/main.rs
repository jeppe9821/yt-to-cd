mod core;

mod app;
use std::path::Path;

use app::ApplicationState;
use app::App;

use iced::window;

mod binary_inc;

use std::process::Command as ProcessCommand;

const APPLICATION_TITLE: &str = "Youtube to CD Downloader";

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
    let window_settings: window::Settings = get_window_settings();
    
    iced::application(APPLICATION_TITLE, 
        ApplicationState::update, 
        ApplicationState::view)
        .window(window_settings)
        .run_with(|| ApplicationState::new(()))
}
