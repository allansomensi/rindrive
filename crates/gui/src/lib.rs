use crate::app::App;

mod app;
mod message;
mod state;
mod view;
mod worker;

pub fn run() -> iced::Result {
    iced::application(App::new, App::update, App::view)
        .title(|_app: &App| "Rindrive".to_string())
        .theme(|_app: &App| iced::Theme::GruvboxDark)
        .subscription(|app: &App| app.subscription())
        .centered()
        .window_size((670.0, 550.0))
        .resizable(false)
        .antialiasing(true)
        .run()
}
