use crate::app::App;
use crate::message::Message;
use crate::view::{canvas, components::badge, sections::sidebar};
use iced::widget::{column, container, row};
use iced::{Alignment, Element, Length};
use rindrive_i18n::fl;

pub fn view<'a>(app: &'a App) -> Element<'a, Message> {
    let map_legend = row![
        badge::legend(badge::COLOR_READING, fl!("legend-read")),
        badge::legend(badge::COLOR_WRITING, fl!("legend-write")),
        badge::legend(badge::COLOR_VALID, fl!("legend-valid")),
        badge::legend(badge::COLOR_INVALID, fl!("legend-bad")),
    ]
    .spacing(15);

    let map_section = column![
        container(map_legend)
            .padding([10, 20])
            .align_x(Alignment::Center)
            .width(Length::Fill),
        canvas::view(&app.block_map, &app.map_cache, 0.0)
    ]
    .width(Length::Fill);

    let main_content_row = row![map_section].width(Length::Fill).height(Length::Fill);

    row![sidebar::view(app), main_content_row]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
