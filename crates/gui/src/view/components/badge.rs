use crate::message::Message;
use iced::widget::{Space, container, row, text};
use iced::{Alignment, Color, Element, Theme};

pub const COLOR_VALID: Color = Color::from_rgb(0.0, 0.8, 0.4);
pub const COLOR_INVALID: Color = Color::from_rgb(0.9, 0.25, 0.25);
pub const COLOR_PENDING: Color = Color::from_rgb(0.15, 0.15, 0.15);
pub const COLOR_READING: Color = Color::from_rgb(0.2, 0.6, 1.0);
pub const COLOR_WRITING: Color = Color::from_rgb(0.9, 0.6, 0.1);

pub fn legend<'a>(color: Color, label: String) -> Element<'a, Message> {
    row![
        container(Space::new())
            .width(12)
            .height(12)
            .style(move |_| {
                container::Style {
                    background: Some(color.into()),
                    border: iced::Border {
                        radius: 3.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            }),
        text(label).size(12).style(text::secondary)
    ]
    .spacing(6)
    .align_y(Alignment::Center)
    .into()
}

pub fn status(log: &str) -> Element<'static, Message> {
    let is_error = log.contains("❌");
    let is_success = log.contains("✅");
    let icon = if is_error {
        "⚠️"
    } else if is_success {
        "🛡️"
    } else {
        "ℹ️"
    };

    container(
        row![
            text(icon).size(16),
            text(log.to_string()).size(12).style(move |theme: &Theme| {
                let ext = theme.extended_palette();
                let color = if is_error {
                    ext.danger.strong.color
                } else if is_success {
                    ext.success.strong.color
                } else {
                    ext.primary.strong.color
                };
                text::Style { color: Some(color) }
            })
        ]
        .spacing(8)
        .align_y(Alignment::Center),
    )
    .padding([6, 10])
    .style(move |theme: &Theme| {
        let ext = theme.extended_palette();
        let color = if is_error {
            ext.danger.strong.color
        } else if is_success {
            ext.success.strong.color
        } else {
            ext.primary.strong.color
        };
        container::Style {
            background: Some(color.scale_alpha(0.08).into()),
            border: iced::Border {
                color: color.scale_alpha(0.2),
                width: 1.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        }
    })
    .into()
}
