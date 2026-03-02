pub mod canvas;
pub mod components;
pub mod screens;
pub mod sections;

use crate::{app::App, message::Message, state::AppState};
use iced::Element;

pub fn view<'a>(app: &'a App) -> Element<'a, Message> {
    if app.state == AppState::Finished {
        screens::finished::view(app)
    } else {
        screens::audit::view(app)
    }
}
