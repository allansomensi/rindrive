use crate::app::App;
use crate::message::Message;
use crate::view::{canvas, components::badge, sections::report};
use iced::widget::{Space, button, column, container, row, text};
use iced::{Alignment, Element, Length, Theme};
use rindrive_i18n::fl;

pub fn view<'a>(app: &'a App) -> Element<'a, Message> {
    let has_errors = app.block_map.contains(&2);

    let (title_icon, title_text) = if has_errors {
        ("⚠️", fl!("report-status-fake"))
    } else {
        ("✅", fl!("report-status-genuine"))
    };

    let header = container(
        row![
            text(title_icon).size(28),
            text(title_text).size(20).style(move |theme: &Theme| {
                let ext = theme.extended_palette();
                let color = if has_errors {
                    ext.danger.strong.color
                } else {
                    ext.success.strong.color
                };
                text::Style { color: Some(color) }
            }),
        ]
        .spacing(15)
        .align_y(Alignment::Center),
    )
    .padding([15, 20])
    .width(Length::Fill);

    let mut hw_content: Element<_> = Space::new().into();

    if let Some(info) = &app.usb_info {
        hw_content = report::view(app, info);
    }

    let map_legend = row![
        badge::legend(badge::COLOR_VALID, fl!("legend-valid")),
        badge::legend(badge::COLOR_INVALID, fl!("legend-bad")),
    ]
    .spacing(15);

    let map_section = column![
        container(map_legend)
            .padding([0, 10])
            .align_x(Alignment::Center)
            .width(Length::Fill),
        canvas::view(&app.block_map, &app.map_cache, 6.0)
    ]
    .width(Length::Fill);

    let content_row = row![
        container(hw_content).width(Length::Fixed(400.0)),
        container(map_section).width(Length::Fill)
    ]
    .spacing(15)
    .width(Length::Fill)
    .height(Length::Fill);

    let btn_back = button(text(fl!("btn-new-audit")).size(15).center())
        .style(button::primary)
        .padding([10, 30])
        .on_press(Message::UnselectDrive);

    let main_col = column![
        header,
        content_row,
        Space::new().height(10),
        container(btn_back).width(Length::Fill)
    ]
    .padding([10, 20]);

    container(main_col)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|theme: &Theme| container::Style {
            background: Some(theme.extended_palette().background.base.color.into()),
            ..Default::default()
        })
        .into()
}
