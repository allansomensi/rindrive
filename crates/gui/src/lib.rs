use crate::app::App;
use iced::Theme;

mod app;
mod message;
mod state;
mod view;
mod worker;

pub fn run() -> iced::Result {
    iced::application(App::new, App::update, App::view)
        .title(|_app: &App| "Rindrive".to_string())
        .theme(|_app: &App| Theme::GruvboxDark)
        .subscription(|app: &App| app.subscription())
        .centered()
        .window_size((498.0, 500.0))
        .resizable(false)
        .antialiasing(true)
        .run()
}
