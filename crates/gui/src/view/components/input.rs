use crate::message::Message;
use iced::widget::{column, text, text_input};
use iced::{Element, Length};

pub fn group<'a>(
    label: String,
    value: &str,
    on_change: fn(String) -> Message,
    enabled: bool,
) -> Element<'a, Message> {
    let mut input = text_input("", value)
        .padding(8)
        .size(14)
        .style(|theme, status| {
            let mut style = text_input::default(theme, status);
            style.border.radius = 4.0.into();
            style
        });

    if enabled {
        input = input.on_input(on_change);
    }

    column![text(label).size(12).style(text::secondary), input]
        .spacing(4)
        .width(Length::Fill)
        .into()
}
